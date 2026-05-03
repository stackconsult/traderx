//! `ectoledger audit` — run a security audit with the cognitive loop.
//!
//! Extracts the ~300-line Audit arm from the monolithic `main.rs` match block
//! and adds a proper supervision tree using `CancellationToken` for cooperative
//! shutdown between the Axum observer server and the agent cognitive loop.

use crate::agent::{self, AgentError, AgentLoopConfig};
use crate::config;
use crate::guard::GuardExecutor;
use crate::guard_process::GuardProcess;
use crate::ledger::{self, AppendError};
use crate::pool;
use crate::sandbox;
use crate::schema::EventPayload;
use crate::server;
use crate::tripwire::Tripwire;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;

// ============================================================================
// MEM0 PHASE 2: Security Event Memory Integration
// ============================================================================

use uuid::Uuid;
use chrono::Utc;

/// Minimal mem0 imprint structure for audit.rs (mirrors journal.rs Mem0MemoryImprint)
/// Stored as JSON in the ledger event log for cross-session pattern learning.
#[derive(Debug, Clone, serde::Serialize)]
struct AuditMem0Imprint {
    memory_id: Uuid,
    memory_type: String,
    content: String,
    agent_role: Option<String>,
    category: Option<String>,
    confidence: Option<f64>,
    tags: Vec<String>,
    related_files: Vec<String>,
    session_id: Uuid,
    timestamp: String,
}

/// Create a mem0 imprint for the audit session.
fn create_audit_mem0_imprint(
    memory_type: &str,
    content: String,
    category: Option<&str>,
    tags: Vec<String>,
    session_id: Uuid,
) -> AuditMem0Imprint {
    AuditMem0Imprint {
        memory_id: Uuid::new_v4(),
        memory_type: memory_type.to_string(),
        content,
        agent_role: Some("audit".to_string()),
        category: category.map(|s| s.to_string()),
        confidence: Some(1.0),
        tags,
        related_files: vec![],
        session_id,
        timestamp: Utc::now().to_rfc3339(),
    }
}

/// Append mem0 imprint to ledger as a Thought event (best-effort).
/// Returns immediately — mem0 storage is non-critical to audit integrity.
async fn append_mem0_imprint(
    pool: &sqlx::PgPool,
    imprint: &AuditMem0Imprint,
    session_signing_key: Option<&crate::signing::SessionSigningKey>,
) {
    let content = match serde_json::to_string(imprint) {
        Ok(json) => format!("[MEM0_IMPRINT] {}", json),
        Err(e) => {
            tracing::warn!("Failed to serialize mem0 imprint: {}", e);
            return;
        }
    };

    if let Err(e) = crate::ledger::append_event(
        pool,
        EventPayload::Thought { content },
        Some(imprint.session_id),
        None,
        session_signing_key,
    ).await {
        tracing::warn!("Failed to append mem0 imprint to ledger: {}", e);
    }
}

/// Arguments for the `audit` subcommand, extracted from the CLI parser.
pub struct AuditArgs {
    pub prompt: String,
    pub policy: Option<PathBuf>,
    pub no_guard: bool,
    pub no_guard_confirmed: bool,
    pub interactive: bool,
    /// Read key-encryption password from this file descriptor (Unix only).
    pub key_password_fd: Option<i32>,
    /// Automatic anchoring interval in seconds (0 = disabled, TM-1e).
    pub auto_anchor_interval: u64,
    /// Target chain for auto-anchoring (`bitcoin` or `ethereum`).
    pub auto_anchor_chain: String,
}

/// Run a full security audit: start the Observer dashboard, spawn the guard,
/// and execute the cognitive loop with proper supervision.
///
/// # Supervision model
///
/// Both the Axum HTTP server and the cognitive loop are `tokio::spawn`-ed.
/// A shared `CancellationToken` coordinates shutdown:
///
/// - **Ctrl-C** → token cancelled → both tasks observe and shut down cooperatively
/// - **Server crash** → token cancelled → cognitive loop stops, session marked "failed"
/// - **Cognitive loop error** → token cancelled → server shuts down gracefully
/// - **Cognitive loop panic** → `JoinHandle` returns `Err` → token cancelled, session "panicked"
pub async fn run(
    pool: sqlx::PgPool,
    metrics: Arc<crate::metrics::Metrics>,
    args: AuditArgs,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let AuditArgs {
        prompt,
        policy,
        no_guard,
        no_guard_confirmed,
        interactive,
        key_password_fd,
        auto_anchor_interval,
        auto_anchor_chain,
    } = args;

    // Log auto-anchoring configuration if enabled.
    if auto_anchor_interval > 0 {
        tracing::info!(
            "Auto-anchoring enabled: every {}s to {} (TM-1e)",
            auto_anchor_interval,
            auto_anchor_chain,
        );
    }

    if no_guard && !no_guard_confirmed {
        return Err(
            "⚠️  WARNING: You specified --no-guard. The Guard provides a separate process \
             and model to validate actions.\n   Running without the Guard reduces security. \
             If you really want to proceed, run with:\n   \
             cargo run -- audit \"<your goal>\" --no-guard --no-guard-confirmed"
                .into(),
        );
    }

    // ── LLM backend ───────────────────────────────────────────────────────────
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()?;
    let llm_backend = crate::llm::backend_from_env(&client)?;
    llm_backend.ensure_ready(&client).await?;

    // ── Policy engine ─────────────────────────────────────────────────────────
    let (policy_engine, policy_hash) = if let Some(ref policy_path) = policy {
        let content = std::fs::read(policy_path).map_err(|e| {
            tracing::error!("Failed to read policy file: {}", e);
            e
        })?;
        let hash = crate::policy::policy_hash_bytes(&content);
        let engine = crate::policy::load_policy_engine(policy_path).map_err(|e| {
            tracing::error!("Failed to load policy: {}", e);
            std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
        })?;
        (Some(engine), Some(hash))
    } else {
        (None, None)
    };

    // One-time startup recovery: clean up zombies from prior server runs.
    // Must run before session creation to avoid TOCTOU with recovered sessions.
    if let Err(e) = crate::wakeup::recover_zombie_sessions(&pool).await {
        tracing::warn!("audit startup: failed to recover zombie sessions: {e}");
    }
    if let Err(e) = crate::wakeup::recover_incomplete_actions(&pool).await {
        tracing::warn!("audit startup: failed to recover incomplete actions: {e}");
    }

    // ── Session creation ──────────────────────────────────────────────────────
    let (session, session_signing_key) = ledger::create_session(
        &pool,
        &prompt,
        llm_backend.backend_name(),
        llm_backend.model_name(),
        policy_hash.as_deref(),
    )
    .await?;
    metrics.inc_sessions_created();
    let session_id = session.id;

    // Persist the signing key so a crash doesn't invalidate the audit trail.
    let key_dir = config::session_key_dir();
    let signing_password = if let Some(fd) = key_password_fd {
        crate::signing::read_password_from_fd(fd)
    } else {
        crate::signing::prompt_or_env_password(
            "Set a password to protect this session's signing key (leave blank to skip): ",
        )
    };
    if let Some(ref pw) = signing_password
        && let Err(e) =
            crate::signing::save_session_key(&key_dir, session_id, &session_signing_key, pw)
    {
        tracing::warn!(
            "Could not persist signing key: {}. Key will be lost on crash.",
            e
        );
    }

    let session_signing_key = Arc::new(session_signing_key);

    // ── Initial thought event ─────────────────────────────────────────────────
    let goal_thought = if let Some(ref h) = policy_hash {
        format!("Audit goal: {}. Policy hash: {}", prompt, h)
    } else {
        format!("Audit goal: {}", prompt)
    };
    ledger::append_event(
        &pool,
        EventPayload::Thought {
            content: goal_thought,
        },
        Some(session_id),
        Some(prompt.as_str()),
        Some(session_signing_key.as_ref()),
    )
    .await?;
    metrics.inc_events_appended();

    tracing::info!(
        "LLM ready ({} / {}). Starting cognitive loop.",
        llm_backend.backend_name(),
        llm_backend.model_name()
    );

    // ── Cloud credentials ─────────────────────────────────────────────────────
    let cloud_creds = crate::cloud_creds::load_cloud_creds().map(Arc::new);
    if let Some(ref c) = cloud_creds {
        tracing::info!(
            "Cloud credentials loaded: {} (provider: {})",
            c.name,
            c.provider
        );
    }

    // ── Approval state ────────────────────────────────────────────────────────
    let approval_state = Arc::new(crate::approvals::ApprovalState::new());

    // ── Observer HTTP server ──────────────────────────────────────────────────
    let pool_observer = pool.clone();
    let metrics_observer = metrics.clone();
    let approval_state_server = Arc::clone(&approval_state);
    let audit_bind_host = config::bind_host();
    let audit_bind_port = config::bind_port();
    let audit_bind_addr = format!("{}:{}", audit_bind_host, audit_bind_port);
    let audit_listener = TcpListener::bind(&audit_bind_addr).await?;
    let audit_display_host = if audit_bind_host == "0.0.0.0" {
        "localhost".to_string()
    } else {
        audit_bind_host.clone()
    };
    tracing::info!(
        "Observer dashboard: http://{}:{}",
        audit_display_host,
        audit_bind_port
    );

    // ── Supervision: shared cancellation token ────────────────────────────────
    let cancel = CancellationToken::new();

    // Use a oneshot channel to detect early server failure.
    let (server_ready_tx, server_ready_rx) = tokio::sync::oneshot::channel::<()>();
    let server_cancel = cancel.clone();
    let server_handle = tokio::spawn(async move {
        let _ = server_ready_tx.send(());
        let (made_router, _task_tracker) = server::router_with_approval_state(
            pool::DatabasePool::Postgres(pool_observer),
            metrics_observer,
            approval_state_server,
            server_cancel.clone(),
            None,
        );
        let server = axum::serve(
            audit_listener,
            made_router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(server_cancel.cancelled_owned());

        if let Err(e) = server.await {
            tracing::error!("Observer server terminated with error: {}", e);
        }
    });

    // Wait briefly for the server task to confirm it entered the serve loop.
    let mut server_handle = std::pin::pin!(server_handle);
    tokio::select! {
        _ = server_ready_rx => { /* server entered serve loop — proceed */ }
        result = &mut server_handle => {
            cancel.cancel();
            match result {
                Ok(()) => return Err("Observer server exited immediately — cannot proceed with audit.".into()),
                Err(e) => return Err(format!("Observer server task panicked: {}", e).into()),
            }
        }
    }

    // ── Tripwire + Guard ──────────────────────────────────────────────────────
    let tw_cfg = config::load_tripwire_config();
    let allowed_paths: Vec<PathBuf> = tw_cfg.allowed_paths.iter().map(PathBuf::from).collect();
    let tripwire = Tripwire::new(
        allowed_paths,
        tw_cfg.allowed_domains,
        tw_cfg.banned_command_patterns,
        tw_cfg.min_justification_length,
        tw_cfg.require_https,
    );

    let guard: Option<Box<dyn GuardExecutor>> = if no_guard && no_guard_confirmed {
        tracing::warn!("Running without Guard (--no-guard --no-guard-confirmed).");
        None
    } else {
        if let Err(e) = config::ensure_guard_config() {
            cancel.cancel();
            return Err(format!("Configuration error: {}", e).into());
        }
        match GuardProcess::spawn() {
            Ok(g) => Some(Box::new(g)),
            Err(e) => {
                cancel.cancel();
                return Err(format!("Guard process failed to start: {}", e).into());
            }
        }
    };

    // ── Webhook egress worker ─────────────────────────────────────────────────
    let (egress_sender, egress_handle) =
        crate::webhook::spawn_egress_worker(config::webhook_config(), pool.clone());

    // Keep the original sender for explicit cleanup; pass a clone into the
    // agent config so the channel is guaranteed to close even if the cognitive
    // loop future is cancelled mid-flight.
    let egress_tx_cleanup = egress_sender.clone();
    let egress_tx = Some(egress_sender);

    // ── Agent configuration ───────────────────────────────────────────────────
    let agent_config = AgentLoopConfig {
        llm: llm_backend,
        tripwire: &tripwire,
        max_steps: Some(config::max_steps()),
        session_id: Some(session_id),
        session_goal: prompt.clone(),
        guard,
        policy: policy_engine.as_ref(),
        session_signing_key: Some(session_signing_key),
        metrics: Some(metrics),
        egress_tx,
        cloud_creds,
        interactive,
        approval_state: Some(approval_state),
        firecracker_config: sandbox::FirecrackerConfig::from_env_opt(),
        docker_config: sandbox::DockerConfig::from_env_opt(),
        key_rotation_interval_steps: config::key_rotation_interval_steps(),
        compensation: Some(crate::compensation::CompensationPlanner::load()),
        enclave: None,
        enclave_attestation: None,
        cancel: Some(cancel.clone()),
    };

    // ── Main supervision loop ─────────────────────────────────────────────────
    //
    // Both the cognitive loop and the server are supervised. Any failure in
    // either task triggers cooperative cancellation of the other.

    let agent_pool = crate::pool::DatabasePool::Postgres(pool.clone());
    let (aborted, session_finished) = tokio::select! {
        result = agent::run_cognitive_loop(&agent_pool, &client, agent_config) => {
            match &result {
                Ok(()) => {
                    if let Err(e) = ledger::finish_session(&pool, session_id, "completed").await {
                        tracing::warn!("Failed to mark session {} as completed: {}", session_id, e);
                    }
                    // MEM0 PHASE 2: Store session outcome — completed
                    let imprint = create_audit_mem0_imprint(
                        "session_outcome",
                        format!("Session {} completed. Prompt: {}. Policy hash: {:?}.", session_id, prompt, policy_hash),
                        Some("audit_session"),
                        vec!["audit".into(), "completed".into()],
                        session_id,
                    );
                    append_mem0_imprint(&pool, &imprint, session_signing_key.as_deref()).await;
                    tracing::info!("Cognitive loop finished.");
                }
                Err(AgentError::Append(AppendError::GoalMismatch)) => {
                    if let Err(e) = ledger::append_event(
                        &pool,
                        EventPayload::Thought {
                            content: "Security: session goal mismatch (possible redirect); aborting.".to_string(),
                        },
                        Some(session_id),
                        None,
                        None,
                    ).await {
                        tracing::warn!("Failed to append goal-mismatch thought: {}", e);
                    }
                    if let Err(e) = ledger::finish_session(&pool, session_id, "aborted").await {
                        tracing::warn!("Failed to mark session {} as aborted: {}", session_id, e);
                    }
                    // MEM0 PHASE 2: Store security event — goal mismatch
                    let imprint = create_audit_mem0_imprint(
                        "security_event",
                        format!(
                            "Goal mismatch detected for session {}. Prompt: {}. Policy hash: {:?}. Session aborted.",
                            session_id, prompt, policy_hash
                        ),
                        Some("goal_mismatch"),
                        vec!["audit".into(), "security".into(), "goal_mismatch".into(), "aborted".into()],
                        session_id,
                    );
                    append_mem0_imprint(&pool, &imprint, session_signing_key.as_deref()).await;
                    cancel.cancel();
                    return Err("Session aborted: goal mismatch.".into());
                }
                Err(AgentError::Append(AppendError::UnverifiedEvidence(msg))) => {
                    if let Err(e) = ledger::append_event(
                        &pool,
                        EventPayload::Thought {
                            content: format!("Findings verification failed: {}; commit rejected.", msg),
                        },
                        Some(session_id),
                        None,
                        None,
                    ).await {
                        tracing::warn!("Failed to append verification-failure thought: {}", e);
                    }
                    if let Err(e) = ledger::finish_session(&pool, session_id, "failed").await {
                        tracing::warn!("Failed to mark session {} as failed: {}", session_id, e);
                    }
                    // MEM0 PHASE 2: Store security event — unverified evidence
                    let imprint = create_audit_mem0_imprint(
                        "security_event",
                        format!(
                            "Unverified evidence for session {}. Message: {}. Prompt: {}. Policy hash: {:?}. Session failed.",
                            session_id, msg, prompt, policy_hash
                        ),
                        Some("unverified_evidence"),
                        vec!["audit".into(), "security".into(), "unverified_evidence".into(), "failed".into()],
                        session_id,
                    );
                    append_mem0_imprint(&pool, &imprint, session_signing_key.as_deref()).await;
                    cancel.cancel();
                    return Err(format!("Session failed: {}", msg).into());
                }
                Err(AgentError::TripwireAbort(reason)) => {
                    if let Err(e) = ledger::finish_session(&pool, session_id, "aborted").await {
                        tracing::warn!("Failed to mark session {} as aborted: {}", session_id, e);
                    }
                    // MEM0 PHASE 2: Store security event — tripwire abort
                    let imprint = create_audit_mem0_imprint(
                        "security_event",
                        format!(
                            "Tripwire triggered for session {}. Reason: {}. Prompt: {}. Policy hash: {:?}. Session aborted.",
                            session_id, reason, prompt, policy_hash
                        ),
                        Some("tripwire_abort"),
                        vec!["audit".into(), "security".into(), "tripwire".into(), "aborted".into()],
                        session_id,
                    );
                    append_mem0_imprint(&pool, &imprint, session_signing_key.as_deref()).await;
                    cancel.cancel();
                    return Err(format!("Session aborted (tripwire): {}", reason).into());
                }
                Err(AgentError::Cancelled) => {
                    if let Err(e) = ledger::finish_session(&pool, session_id, "aborted").await {
                        tracing::warn!("Failed to mark session {} as aborted: {}", session_id, e);
                    }
                    // MEM0 PHASE 2: Store security event — cancelled
                    let imprint = create_audit_mem0_imprint(
                        "security_event",
                        format!(
                            "Session {} cancelled by user or system. Prompt: {}. Policy hash: {:?}. Session aborted.",
                            session_id, prompt, policy_hash
                        ),
                        Some("cancelled"),
                        vec!["audit".into(), "security".into(), "cancelled".into(), "aborted".into()],
                        session_id,
                    );
                    append_mem0_imprint(&pool, &imprint, session_signing_key.as_deref()).await;
                    cancel.cancel();
                }
                Err(_) => {
                    if let Err(e) = ledger::finish_session(&pool, session_id, "failed").await {
                        tracing::warn!("Failed to mark session {} as failed: {}", session_id, e);
                    }
                    cancel.cancel();
                    result?;
                }
            }
            // Signal the server to shut down gracefully now that the audit is done.
            cancel.cancel();
            (false, false)
        }
        result = &mut server_handle => {
            // The dashboard server exited or panicked while the cognitive
            // loop was running — abort so the user isn't left headless.
            match result {
                Ok(()) => tracing::error!("Observer server exited unexpectedly during audit."),
                Err(e) => tracing::error!("Observer server panicked during audit: {}", e),
            }
            if let Err(e) = ledger::finish_session(&pool, session_id, "failed").await {
                tracing::warn!("Failed to mark session {} as failed: {}", session_id, e);
            }
            // MEM0 PHASE 2: Store session outcome — server failure
            let imprint = create_audit_mem0_imprint(
                "session_outcome",
                format!(
                    "Observer server failed for session {}. Prompt: {}. Policy hash: {:?}.",
                    session_id, prompt, policy_hash
                ),
                Some("server_failure"),
                vec!["audit".into(), "server".into(), "failed".into()],
                session_id,
            );
            append_mem0_imprint(&pool, &imprint, session_signing_key.as_deref()).await;
            cancel.cancel();
            (true, true)
        }
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Shutdown signal received; cancelling tasks…");
            cancel.cancel();
            // MEM0 PHASE 2: Store session outcome — signal abort
            let imprint = create_audit_mem0_imprint(
                "session_outcome",
                format!(
                    "Session {} aborted by shutdown signal. Prompt: {}. Policy hash: {:?}.",
                    session_id, prompt, policy_hash
                ),
                Some("signal_abort"),
                vec!["audit".into(), "signal".into(), "aborted".into()],
                session_id,
            );
            append_mem0_imprint(&pool, &imprint, session_signing_key.as_deref()).await;
            (true, false)
        }
    };

    // ── Post-audit cleanup ────────────────────────────────────────────────────
    // Drop our copy of the egress sender so the egress worker can observe
    // channel closure and shut down, even if the cognitive loop future was
    // cancelled before it had a chance to drop its copy.
    drop(egress_tx_cleanup);
    if let Err(e) = egress_handle.await {
        tracing::error!("Webhook egress worker panicked: {e}");
    }
    if aborted {
        if let Some((seq, _)) = ledger::get_latest(&pool).await?
            && let Err(e) = crate::snapshot::snapshot_at_sequence(&pool, seq).await
        {
            tracing::warn!("Failed to snapshot on abort: {}", e);
        }
        if !session_finished
            && let Err(e) = ledger::finish_session(&pool, session_id, "aborted").await
        {
            tracing::warn!(
                "Failed to mark session {} as aborted on signal: {}",
                session_id,
                e
            );
        }
        tracing::info!("Shutdown signal received; session aborted.");
    }
    Ok(())
}

// ============================================================================
// MEM0 PHASE 2: Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_create_audit_mem0_imprint() {
        let imprint = create_audit_mem0_imprint(
            "session_outcome",
            "Session completed. Prompt: test".to_string(),
            Some("audit_session"),
            vec!["audit".into(), "completed".into()],
            Uuid::new_v4(),
        );

        assert_eq!(imprint.memory_type, "session_outcome");
        assert_eq!(imprint.content, "Session completed. Prompt: test");
        assert_eq!(imprint.category, Some("audit_session".to_string()));
        assert_eq!(imprint.tags, vec!["audit", "completed"]);
        assert_eq!(imprint.agent_role, Some("audit".to_string()));
        assert_eq!(imprint.confidence, Some(1.0));
        assert!(imprint.memory_id != Uuid::nil());
        assert!(!imprint.timestamp.is_empty());
    }

    #[test]
    fn test_audit_mem0_imprint_serialization() {
        let imprint = create_audit_mem0_imprint(
            "security_event",
            "Goal mismatch detected".to_string(),
            Some("goal_mismatch"),
            vec!["audit".into(), "security".into()],
            Uuid::new_v4(),
        );

        let json = serde_json::to_string(&imprint).unwrap();
        assert!(json.contains("security_event"));
        assert!(json.contains("Goal mismatch detected"));
        assert!(json.contains("goal_mismatch"));
        assert!(json.contains("audit"));
    }

    #[test]
    fn test_audit_mem0_imprint_all_event_types() {
        let session_id = Uuid::new_v4();

        let types = vec![
            ("session_outcome", "audit_session", vec!["completed"]),
            ("security_event", "goal_mismatch", vec!["security", "aborted"]),
            ("security_event", "unverified_evidence", vec!["security", "failed"]),
            ("security_event", "tripwire_abort", vec!["tripwire", "aborted"]),
            ("security_event", "cancelled", vec!["cancelled", "aborted"]),
            ("session_outcome", "server_failure", vec!["server", "failed"]),
            ("session_outcome", "signal_abort", vec!["signal", "aborted"]),
        ];

        for (mem_type, category, tags) in types {
            let imprint = create_audit_mem0_imprint(
                mem_type,
                format!("Test content for {} / {}", mem_type, category),
                Some(category),
                tags.iter().map(|s| s.to_string()).collect(),
                session_id,
            );
            assert_eq!(imprint.memory_type, mem_type);
            assert_eq!(imprint.category, Some(category.to_string()));
            assert_eq!(imprint.session_id, session_id);
        }
    }
}

/// Run audit using a `DatabasePool` (supports both Postgres and SQLite).
///
/// For now the SQLite path returns a "not implemented" error; the Postgres
/// path delegates to [`run`].
pub async fn run_with_pool(
    db: pool::DatabasePool,
    metrics: Arc<crate::metrics::Metrics>,
    args: AuditArgs,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match db {
        pool::DatabasePool::Postgres(pg) => run(pg, metrics, args).await,
        pool::DatabasePool::Sqlite(_) => {
            Err("audit is not yet fully implemented for SQLite mode.".into())
        }
    }
}

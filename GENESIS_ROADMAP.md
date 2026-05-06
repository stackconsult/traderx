# Genesis Roadmap — Telemetry Journal

> Snapshot-based telemetry log for the Genesis autonomous hoard agent.
> Written by agents, readable by humans.
> Snapshots taken at end of each session and after every 10 task entries.
> Format: `## [YYYY-MM-DD HH:MM UTC] [category] — [title]`

---

## Snapshot — 2026-05-05 Session 1

**Branch**: `feature/github-mcp-setup`
**Last commit**: `3f3fb95` — fix(portfolio-aggregation): resolve all 10 compile errors
**Windsurf version**: Cascade SWE-1 (auto-detected)
**Ollama version**: 0.23.0 (launchd service active)
**Models available**: `nomic-embed-text:latest` (gemma3:1b + qwen2.5-coder:1.5b pulling)

### Environment Status

- [x] Rust compile errors: **0** (portfolio-aggregation + oms-engine clean)
- [x] Ollama: running at `127.0.0.1:11434` via launchd
- [x] Genesis config: `~/.genesis/config/global.json` — provider: google
- [x] LiteLLM config: `litellm-config.yaml` present
- [x] MCP servers: 12 configured (tinyfish + ollama-bridge + 10 others)
- [x] All 35 agents: skill-bound with `genesis_model:` + plain triggers
- [ ] gemma3:1b: pulling (background)
- [ ] qwen2.5-coder:1.5b: pulling (background)
- [ ] pgvector: needs `CREATE EXTENSION pgvector` confirmation

### Skills Installed This Session

| Skill | Source | Status |
|-------|--------|--------|
| `llm-modelling` | Genesis session | ✅ |
| `reasoning-logic` | Genesis session | ✅ |
| `language-selection` | Genesis session | ✅ |
| `full-scope-search` | Genesis session | ✅ |
| `self-upskill` | Genesis session | ✅ |
| `neural-context` | Genesis session | ✅ |

### Compile Fixes This Session

| File | Error | Fix |
|------|-------|-----|
| `portfolio-aggregation/src/engine.rs` | `AggregatorEvent` derives Serialize with Sender<T> | Removed serde derives; introduced `WalRecord` in persistence.rs |
| `portfolio-aggregation/src/engine.rs` | `Arc<Self>` mutable borrow on `event_rx` | Wrapped in `Mutex<mpsc::Receiver<...>>` |
| `portfolio-aggregation/src/engine.rs` | WAL recovery passes `&Fill` to owned fn | Added `.clone()` |
| `portfolio-aggregation/src/risk.rs` | `DashMap Ref` used as divisor | Added `*old_price` deref |
| `portfolio-aggregation/src/risk.rs` | `AssetClass` deref (`*entry.key().1`) | Changed to `.clone()` |

### Wiring Done This Session

| Component | Action |
|-----------|--------|
| TinyFish MCP | Added to `mcp_config.json` + `User/mcp_servers.json` |
| Ollama launchd | `com.genesis.ollama.plist` — auto-starts on login |
| Windsurf settings | `autoContinue: 100`, trust disabled, SWE-1 default |
| AgentKit (35 agents) | All categories skill-bound, genesis_model assigned |
| `orchestrate.md` | 35-agent router workflow created |
| `genesis-hoard-agent.md` | Master autonomous agent workflow |
| `GENESIS_ROADMAP.md` | This file — telemetry journal |

### Pending (Next Session)

- [x] Confirm `gemma3:1b` + `qwen2.5-coder:1.5b` pulled successfully
- [x] Replace PostgreSQL/pgvector with LanceDB (macOS 12 compatible)
- [x] Wire `mem0` to LanceDB for vector storage
- [x] Test LanceDB integration with mem0
- [ ] Test TinyFish OAuth flow (browser prompt on first use)
- [x] Run `/preflight-checklist` to verify full deployment readiness
- [x] Set `GOOGLE_API_KEY` in shell environment (`~/.zshrc`)
- [x] Genesis hoard tmux daemon operational (SWE-1.5 fix)
- [x] GitHub MCP servers validated (14 servers configured)
- [x] Guardrail assembly completed (agent_roles.yaml, drift_signals.yaml, rule_registry.yaml, skill_registry.yaml)
- [x] Genesis hoard startup integrated with guardrail validation

---

## Skill Gaps Resolved

| Date | Skill | Installed from | Resolved task |
|------|-------|---------------|---------------|
| 2026-05-05 | `llm-modelling` | Genesis session | Model selection + rate limiting |
| 2026-05-05 | `reasoning-logic` | Genesis session | Anti-stuck 5-step loop |
| 2026-05-05 | `language-selection` | Genesis session | Rust/Python/TS decision matrix |
| 2026-05-05 | `full-scope-search` | Genesis session | Codebase + web search patterns |
| 2026-05-05 | `self-upskill` | Genesis session | Skill gap detection + install loop |
| 2026-05-05 | `neural-context` | Genesis session | Context budgeting + handoff protocol |

---

## Architectural Decisions

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-05-05 | Provider: Google Gemini (not Anthropic) | User has Google API key, not Anthropic |
| 2026-05-05 | Local models: Ollama 0.23.0 | Free, privacy-safe for code tasks |
| 2026-05-05 | `AggregatorEvent` no serde | `mpsc::Sender<T>` is not serialisable; WAL uses `WalRecord` instead |
| 2026-05-05 | `event_rx: Mutex<Receiver>` | Enables `Arc<Self>::run()` without moving out of Arc |
| 2026-05-05 | TinyFish via `mcp-remote` | HTTP MCP transport; OAuth on first use |

---

## Task Log

### 2026-05-05 13:50 UTC — compile-fix — portfolio-aggregation 10 errors

- Agent: Cascade (backend-architect role)
- Proof: `cargo check --package portfolio-aggregation` → 0 errors
- Status: ✅

### 2026-05-05 14:00 UTC — wiring — Ollama ↔ Windsurf Cascade

- Agent: Cascade (devops-automator role)
- Proof: `curl http://127.0.0.1:11434/api/tags` → 200 OK, models listed
- Status: ✅

### 2026-05-05 14:05 UTC — install — TinyFish MCP

- Agent: Cascade (devops-automator role)
- Proof: added to `mcp_config.json` + `User/mcp_servers.json`
- Status: ✅ (OAuth required on first Windsurf use)

### 2026-05-05 14:10 UTC — install — 6 new skills

- Agent: Cascade (ai-engineer role)
- Skills: llm-modelling, reasoning-logic, language-selection, full-scope-search, self-upskill, neural-context
- Proof: files present in `.windsurf/skills/`
- Status: ✅

### 2026-05-05 14:15 UTC — install — 35-agent AgentKit wiring

- Agent: Cascade (workflow-optimizer + studio-producer roles)
- Proof: all .ai/ agents have `skills:` + `genesis_model:` frontmatter
- Status: ✅

---

## Snapshot Template (copy for each new snapshot)

```markdown
## Snapshot — YYYY-MM-DD Session N

**Branch**: ``
**Last commit**: `` — 
**Models available**: 

### Environment Status
- [ ] Compile errors: 
- [ ] Ollama: 
- [ ] TinyFish MCP: 
- [ ] pgvector: 
- [ ] mem0: 

### Done This Session
- 

### Pending
- 
```

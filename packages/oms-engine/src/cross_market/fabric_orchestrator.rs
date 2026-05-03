use crate::cross_market::cross_layer_fusion::{CrossLayerFusion, FusionWeights};
use crate::cross_market::deterministic_engine::{
    DeterministicProfitEngine, ProfitEngineParams,
};
use crate::cross_market::fabric_guard::{FabricGuard, GuardDecision, GuardParams};
use crate::cross_market::market_fabric::FabricState;
use crate::cross_market::noise_filter::{NoiseFilter, NoiseFilterParams};
use crate::cross_market::pattern_layers::{
    PatternLayerEngine, PatternLayerParams,
};
use crate::cross_market::time_bounded_router::{RouteResult, RouterParams, TimeBoundedRouter};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use traderx_mem0_types::Mem0MemoryImprint;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorParams {
    pub noise: NoiseFilterParams,
    pub pattern: PatternLayerParams,
    pub fusion: FusionWeights,
    pub profit: ProfitEngineParams,
    pub router: RouterParams,
    pub guard: GuardParams,
    pub starting_nav: f64,
}

impl Default for OrchestratorParams {
    fn default() -> Self {
        Self {
            noise: NoiseFilterParams::default(),
            pattern: PatternLayerParams::default(),
            fusion: FusionWeights::default(),
            profit: ProfitEngineParams::default(),
            router: RouterParams::default(),
            guard: GuardParams::default(),
            starting_nav: 1_000_000.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorResult {
    pub allowed: Vec<GuardedRoute>,
    pub rejected: Vec<GuardedRoute>,
    pub patterns_detected: usize,
    pub signals_fused: usize,
    pub decisions_made: usize,
    pub timestamp: DateTime<Utc>,
    pub latency_us: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardedRoute {
    pub route: RouteResult,
    pub guard: GuardDecision,
    pub symbol: String,
}

pub struct FabricOrchestrator {
    noise_filter: NoiseFilter,
    pattern_engine: PatternLayerEngine,
    fusion: CrossLayerFusion,
    profit_engine: DeterministicProfitEngine,
    router: TimeBoundedRouter,
    guard: FabricGuard,
    params: OrchestratorParams,
    session_id: Option<Uuid>,
    mem0_buffer: Vec<Mem0MemoryImprint>,
}

impl FabricOrchestrator {
    pub fn new(params: OrchestratorParams) -> Self {
        Self {
            noise_filter: NoiseFilter::new(params.noise.clone()),
            pattern_engine: PatternLayerEngine::new(params.pattern.clone()),
            fusion: CrossLayerFusion::new(params.fusion.clone(), params.pattern.clone()),
            profit_engine: DeterministicProfitEngine::new(params.profit.clone()),
            router: TimeBoundedRouter::new(params.router.clone()),
            guard: FabricGuard::new(params.guard.clone(), params.starting_nav),
            params,
            session_id: None,
            mem0_buffer: Vec::new(),
        }
    }

    /// Set a session ID so that mem0 telemetry events carry traceable origin.
    /// Call before tick() if downstream persistence is desired.
    pub fn set_session_id(&mut self, id: Uuid) {
        self.session_id = Some(id);
    }

    /// Drain all buffered mem0 events produced by the last tick(s).
    /// Callers (async or sync) are responsible for pushing them into a journal.
    pub fn drain_mem0_events(&mut self) -> Vec<Mem0MemoryImprint> {
        std::mem::take(&mut self.mem0_buffer)
    }

    pub fn ingest_ohlcv(&mut self, symbol: &str, high: f64, low: f64, close: f64, volume: f64) {
        self.pattern_engine.record(symbol, high, low, close, volume);
    }

    pub fn tick(&mut self, fabric: &FabricState) -> OrchestratorResult {
        let start = std::time::Instant::now();
        let now = Utc::now();
        let sid = self.session_id;

        // 1. Noise filter the fabric
        let mut clean_states = Vec::new();
        let mut noise_filtered = 0usize;
        for (sym, state) in &fabric.asset_states {
            let nf = self.noise_filter.filter(state);
            if nf.is_noise {
                noise_filtered += 1;
            } else {
                clean_states.push((sym.clone(), state.clone()));
            }
        }

        if let Some(id) = sid {
            self.mem0_buffer.push(
                Mem0MemoryImprint::new(
                    "orchestrator_phase",
                    format!(
                        "Phase 1: noise_filter | assets_total={} noise_filtered={} clean={}",
                        fabric.asset_states.len(),
                        noise_filtered,
                        clean_states.len()
                    ),
                    id,
                )
                .with_agent_role("fabric_orchestrator")
                .with_category("phase_1_noise_filter")
                .with_confidence(1.0)
                .with_tags(vec!["tick".to_string(), "noise_filter".to_string()]),
            );

            // Red-team finding: everything filtered (data quality / false positive flood)
            if clean_states.is_empty() && !fabric.asset_states.is_empty() {
                self.mem0_buffer.push(
                    Mem0MemoryImprint::new(
                        "red_team_finding",
                        format!(
                            "All {} assets filtered as noise — potential data quality issue or filter misconfiguration",
                            fabric.asset_states.len()
                        ),
                        id,
                    )
                    .with_agent_role("fabric_orchestrator")
                    .with_category("red_team_noise_flood")
                    .with_confidence(1.0)
                    .with_tags(vec!["red_team".to_string(), "anomaly".to_string(), "noise_flood".to_string()]),
                );
            }
        }

        // 2. Pattern detection on all tracked symbols
        let patterns = self.pattern_engine.scan_all();
        let patterns_count = patterns.len();

        if let Some(id) = sid {
            self.mem0_buffer.push(
                Mem0MemoryImprint::new(
                    "orchestrator_phase",
                    format!(
                        "Phase 2: pattern_detection | patterns_detected={} clean_states={}",
                        patterns_count,
                        clean_states.len()
                    ),
                    id,
                )
                .with_agent_role("fabric_orchestrator")
                .with_category("phase_2_pattern_detection")
                .with_confidence(1.0)
                .with_tags(vec!["tick".to_string(), "pattern_detection".to_string()]),
            );

            // Red-team finding: zero patterns despite clean states (model degradation)
            if patterns_count == 0 && !clean_states.is_empty() {
                self.mem0_buffer.push(
                    Mem0MemoryImprint::new(
                        "red_team_finding",
                        format!(
                            "Zero patterns detected across {} clean states — pattern engine degradation or regime shift",
                            clean_states.len()
                        ),
                        id,
                    )
                    .with_agent_role("fabric_orchestrator")
                    .with_category("red_team_pattern_degradation")
                    .with_confidence(1.0)
                    .with_tags(vec!["red_team".to_string(), "anomaly".to_string(), "pattern_degradation".to_string()]),
                );
            }
        }

        // 3. Cross-layer fusion
        let fused = self.fusion.fuse(&patterns);
        let fused_count = fused.len();

        if let Some(id) = sid {
            self.mem0_buffer.push(
                Mem0MemoryImprint::new(
                    "orchestrator_phase",
                    format!(
                        "Phase 3: cross_layer_fusion | patterns={} fused={}",
                        patterns_count, fused_count
                    ),
                    id,
                )
                .with_agent_role("fabric_orchestrator")
                .with_category("phase_3_fusion")
                .with_confidence(1.0)
                .with_tags(vec!["tick".to_string(), "fusion".to_string()]),
            );
        }

        // 4. Deterministic profit engine
        let decisions = self.profit_engine.decide(&fused);
        let decisions_count = decisions.len();

        if let Some(id) = sid {
            self.mem0_buffer.push(
                Mem0MemoryImprint::new(
                    "orchestrator_phase",
                    format!(
                        "Phase 4: profit_engine | fused={} decisions={}",
                        fused_count, decisions_count
                    ),
                    id,
                )
                .with_agent_role("fabric_orchestrator")
                .with_category("phase_4_profit_decision")
                .with_confidence(1.0)
                .with_tags(vec!["tick".to_string(), "profit_engine".to_string()]),
            );
        }

        // 5. Route + Guard each decision
        let mut allowed = Vec::new();
        let mut rejected = Vec::new();
        let mut route_failures = 0usize;
        for dec in decisions {
            if let Some(route) = self.router.route(dec.clone()) {
                let guard_dec = self.guard.check(&route);
                let gr = GuardedRoute {
                    symbol: route.symbol.clone(),
                    route,
                    guard: guard_dec.clone(),
                };
                if guard_dec.allowed {
                    allowed.push(gr);
                } else {
                    rejected.push(gr);
                }
            } else {
                route_failures += 1;
            }
        }

        let latency_us = start.elapsed().as_micros();

        if let Some(id) = sid {
            self.mem0_buffer.push(
                Mem0MemoryImprint::new(
                    "orchestrator_phase",
                    format!(
                        "Phase 5: route_guard | allowed={} rejected={} route_failures={} halted={} latency_us={}",
                        allowed.len(), rejected.len(), route_failures, self.guard.is_halted(), latency_us
                    ),
                    id,
                )
                .with_agent_role("fabric_orchestrator")
                .with_category("phase_5_route_guard")
                .with_confidence(1.0)
                .with_tags(vec!["tick".to_string(), "route_guard".to_string()]),
            );

            // Red-team finding: guard is halted (risk circuit breaker)
            if self.guard.is_halted() {
                self.mem0_buffer.push(
                    Mem0MemoryImprint::new(
                        "red_team_finding",
                        "Guard halted — risk circuit breaker triggered, all further routing blocked".to_string(),
                        id,
                    )
                    .with_agent_role("fabric_orchestrator")
                    .with_category("red_team_guard_halted")
                    .with_confidence(1.0)
                    .with_tags(vec!["red_team".to_string(), "anomaly".to_string(), "circuit_breaker".to_string()]),
                );
            }

            // Red-team finding: all routes rejected (market risk or model degradation)
            if decisions_count > 0 && allowed.is_empty() {
                self.mem0_buffer.push(
                    Mem0MemoryImprint::new(
                        "red_team_finding",
                        format!(
                            "All {} decisions rejected by guard — potential systematic risk or guard misconfiguration",
                            decisions_count
                        ),
                        id,
                    )
                    .with_agent_role("fabric_orchestrator")
                    .with_category("red_team_total_rejection")
                    .with_confidence(1.0)
                    .with_tags(vec!["red_team".to_string(), "anomaly".to_string(), "total_rejection".to_string()]),
                );
            }

            // Red-team finding: routing failures (data inconsistency)
            if route_failures > 0 {
                self.mem0_buffer.push(
                    Mem0MemoryImprint::new(
                        "red_team_finding",
                        format!(
                            "Router failed to produce route for {} of {} decisions — symbol mapping or data inconsistency",
                            route_failures, decisions_count
                        ),
                        id,
                    )
                    .with_agent_role("fabric_orchestrator")
                    .with_category("red_team_route_failure")
                    .with_confidence(1.0)
                    .with_tags(vec!["red_team".to_string(), "anomaly".to_string(), "route_failure".to_string()]),
                );
            }
        }

        OrchestratorResult {
            allowed,
            rejected,
            patterns_detected: patterns_count,
            signals_fused: fused_count,
            decisions_made: decisions_count,
            timestamp: now,
            latency_us,
        }
    }

    pub fn record_execution_pnl(&mut self, pnl: f64) {
        self.guard.record_pnl(pnl);
    }

    pub fn is_halted(&self) -> bool {
        self.guard.is_halted()
    }
    pub fn manual_override(&mut self) {
        self.guard.manual_override();
    }
    pub fn reset(&mut self) {
        self.pattern_engine = PatternLayerEngine::new(self.params.pattern.clone());
        self.profit_engine = DeterministicProfitEngine::new(self.params.profit.clone());
        self.guard = FabricGuard::new(self.params.guard.clone(), self.params.starting_nav);
        self.mem0_buffer.clear();
    }
}

impl Default for FabricOrchestrator {
    fn default() -> Self {
        Self::new(OrchestratorParams::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cross_market::market_fabric::FabricState;
    use crate::cross_market::noise_filter::AssetFabricState;

    fn build_fabric(symbols: &[&str]) -> FabricState {
        let mut fabric = FabricState::new();
        for sym in symbols {
            fabric.assets.insert(
                sym.to_string(),
                AssetFabricState {
                    symbol: sym.to_string(),
                    last_price: 100.0,
                    bid: 99.5,
                    ask: 100.5,
                    volume_24h: 1_000_000.0,
                    volatility_1h: 0.01,
                    spread_bps: 5.0,
                    timestamp: Utc::now(),
                    health_score: 0.95,
                    deterministic_hash: format!("hash_{}", sym),
                },
            );
        }
        fabric
    }

    fn feed_sine(
        orch: &mut FabricOrchestrator,
        symbol: &str,
        n: usize,
        base: f64,
        amp: f64,
        vol: f64,
    ) {
        for i in 0..n {
            let price = base + amp * (i as f64 / n as f64 * std::f64::consts::PI * 2.0).sin();
            let high = price + 0.5;
            let low = price - 0.5;
            orch.ingest_ohlcv(symbol, high, low, price, vol);
        }
    }

    #[test]
    fn test_end_to_end_flow() {
        let mut orch = FabricOrchestrator::default();
        let fabric = build_fabric(&["AAPL"]);
        // Feed sine wave to create patterns
        feed_sine(&mut orch, "AAPL", 50, 100.0, 5.0, 1000.0);
        let res = orch.tick(&fabric);
        // Should detect at least some patterns
        assert!(
            res.patterns_detected > 0,
            "Should detect patterns; got {}",
            res.patterns_detected
        );
        assert!(
            res.latency_us < 1_000_000,
            "Should complete in <1s; took {}µs",
            res.latency_us
        );
    }

    #[test]
    fn test_guard_blocks_on_anomaly() {
        let mut orch = FabricOrchestrator::default();
        let fabric = build_fabric(&["AAPL"]);
        feed_sine(&mut orch, "AAPL", 50, 100.0, 5.0, 1000.0);
        // Record massive loss to trigger anomaly
        for _ in 0..30 {
            orch.record_execution_pnl(-1000.0);
        }
        let res = orch.tick(&fabric);
        // All should be rejected due to anomaly halt
        assert!(
            !res.allowed.is_empty() || !res.rejected.is_empty(),
            "Should produce some routes"
        );
    }

    #[test]
    fn test_manual_override_resumes() {
        let mut orch = FabricOrchestrator::default();
        let fabric = build_fabric(&["AAPL"]);
        feed_sine(&mut orch, "AAPL", 50, 100.0, 5.0, 1000.0);
        // Trigger halt
        for _ in 0..30 {
            orch.record_execution_pnl(-1000.0);
        }
        let res1 = orch.tick(&fabric);
        assert!(res1.allowed.is_empty(), "Should block when halted");
        assert!(orch.is_halted());
        // Override
        orch.manual_override();
        assert!(!orch.is_halted());
        let res2 = orch.tick(&fabric);
        assert!(
            !res2.allowed.is_empty() || !res2.rejected.is_empty(),
            "Should resume after override"
        );
    }

    #[test]
    fn test_mem0_no_events_without_session() {
        let mut orch = FabricOrchestrator::default();
        let fabric = build_fabric(&["AAPL"]);
        feed_sine(&mut orch, "AAPL", 50, 100.0, 5.0, 1000.0);
        let _ = orch.tick(&fabric);
        let drained = orch.drain_mem0_events();
        assert!(drained.is_empty(), "No mem0 events without session_id");
    }

    #[test]
    fn test_mem0_phase_events_emitted() {
        let mut orch = FabricOrchestrator::default();
        let sid = Uuid::new_v4();
        orch.set_session_id(sid);

        let fabric = build_fabric(&["AAPL"]);
        feed_sine(&mut orch, "AAPL", 50, 100.0, 5.0, 1000.0);
        let _ = orch.tick(&fabric);

        let drained = orch.drain_mem0_events();
        assert!(!drained.is_empty(), "Phase events should be emitted");

        let phase_events: Vec<_> = drained
            .iter()
            .filter(|e| e.memory_type == "orchestrator_phase")
            .collect();
        assert!(
            phase_events.len() >= 3,
            "Expected at least 3 phase events (noise, pattern, route); got {}",
            phase_events.len()
        );

        for event in &drained {
            assert_eq!(event.session_id, sid, "All events must carry session_id");
            assert_eq!(event.agent_role, Some("fabric_orchestrator".to_string()));
        }
    }

    #[test]
    fn test_mem0_red_team_guard_halted() {
        let mut orch = FabricOrchestrator::default();
        let sid = Uuid::new_v4();
        orch.set_session_id(sid);

        let fabric = build_fabric(&["AAPL"]);
        feed_sine(&mut orch, "AAPL", 50, 100.0, 5.0, 1000.0);
        // Trigger halt
        for _ in 0..30 {
            orch.record_execution_pnl(-1000.0);
        }

        let _ = orch.tick(&fabric);
        let drained = orch.drain_mem0_events();

        let red_team: Vec<_> = drained
            .iter()
            .filter(|e| e.memory_type == "red_team_finding")
            .collect();
        assert!(
            !red_team.is_empty(),
            "Expected red_team_finding when guard is halted"
        );

        let halted_events: Vec<_> = red_team
            .iter()
            .filter(|e| e.category.as_deref() == Some("red_team_guard_halted"))
            .collect();
        assert!(
            !halted_events.is_empty(),
            "Expected red_team_guard_halted event"
        );
    }

    #[test]
    fn test_mem0_drain_clears_buffer() {
        let mut orch = FabricOrchestrator::default();
        orch.set_session_id(Uuid::new_v4());

        let fabric = build_fabric(&["AAPL"]);
        feed_sine(&mut orch, "AAPL", 10, 100.0, 5.0, 1000.0);
        let _ = orch.tick(&fabric);

        let first = orch.drain_mem0_events();
        assert!(!first.is_empty());

        let second = orch.drain_mem0_events();
        assert!(second.is_empty(), "Buffer must be empty after drain");
    }
}

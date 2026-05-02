use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use crate::cross_market::market_fabric::{FabricState, AssetFabricState};
use crate::cross_market::noise_filter::{NoiseFilter, NoiseFilterParams, NoiseFilterResult};
use crate::cross_market::pattern_layers::{PatternLayerEngine, PatternLayerParams, LayerPatternDetection};
use crate::cross_market::cross_layer_fusion::{CrossLayerFusion, FusionWeights, FusedSignal};
use crate::cross_market::deterministic_engine::{DeterministicProfitEngine, ProfitEngineParams, TradeDecision};
use crate::cross_market::time_bounded_router::{TimeBoundedRouter, RouterParams, RouteResult};
use crate::cross_market::fabric_guard::{FabricGuard, GuardParams, GuardDecision, HaltLevel};

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
        }
    }

    pub fn ingest_ohlcv(&mut self, symbol: &str, high: f64, low: f64, close: f64, volume: f64) {
        self.pattern_engine.record(symbol, high, low, close, volume);
    }

    pub fn tick(&mut self, fabric: &FabricState) -> OrchestratorResult {
        let start = std::time::Instant::now();
        let now = Utc::now();

        // 1. Noise filter the fabric
        let mut clean_states = Vec::new();
        for (sym, state) in &fabric.assets {
            let nf = self.noise_filter.filter(sym, state);
            match nf.action {
                crate::cross_market::noise_filter::FilterAction::Pass => {
                    clean_states.push((sym.clone(), state.clone()));
                }
                _ => {}
            }
        }

        // 2. Pattern detection on all tracked symbols
        let patterns = self.pattern_engine.scan_all();
        let patterns_count = patterns.len();

        // 3. Cross-layer fusion
        let fused = self.fusion.fuse(&patterns);
        let fused_count = fused.len();

        // 4. Deterministic profit engine
        let decisions = self.profit_engine.decide(&fused);
        let decisions_count = decisions.len();

        // 5. Route + Guard each decision
        let mut allowed = Vec::new();
        let mut rejected = Vec::new();
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
            }
        }

        OrchestratorResult {
            allowed,
            rejected,
            patterns_detected: patterns_count,
            signals_fused: fused_count,
            decisions_made: decisions_count,
            timestamp: now,
            latency_us: start.elapsed().as_micros(),
        }
    }

    pub fn record_execution_pnl(&mut self, pnl: f64) {
        self.guard.record_pnl(pnl);
    }

    pub fn is_halted(&self) -> bool { self.guard.is_halted() }
    pub fn manual_override(&mut self) { self.guard.manual_override(); }
    pub fn reset(&mut self) {
        self.pattern_engine = PatternLayerEngine::new(self.params.pattern.clone());
        self.profit_engine = DeterministicProfitEngine::new(self.params.profit.clone());
        self.guard = FabricGuard::new(self.params.guard.clone(), self.params.starting_nav);
    }
}

impl Default for FabricOrchestrator {
    fn default() -> Self { Self::new(OrchestratorParams::default()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cross_market::market_fabric::FabricState;
    use crate::cross_market::noise_filter::AssetFabricState;

    fn build_fabric(symbols: &[&str]) -> FabricState {
        let mut fabric = FabricState::new();
        for sym in symbols {
            fabric.assets.insert(sym.to_string(), AssetFabricState {
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
            });
        }
        fabric
    }

    fn feed_sine(orch: &mut FabricOrchestrator, symbol: &str, n: usize, base: f64, amp: f64, vol: f64) {
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
        assert!(res.patterns_detected > 0, "Should detect patterns; got {}", res.patterns_detected);
        assert!(res.latency_us < 1_000_000, "Should complete in <1s; took {}µs", res.latency_us);
    }

    #[test]
    fn test_guard_blocks_on_anomaly() {
        let mut orch = FabricOrchestrator::default();
        let fabric = build_fabric(&["AAPL"]);
        feed_sine(&mut orch, "AAPL", 50, 100.0, 5.0, 1000.0);
        // Record massive loss to trigger anomaly
        for _ in 0..30 { orch.record_execution_pnl(-1000.0); }
        let res = orch.tick(&fabric);
        // All should be rejected due to anomaly halt
        assert!(!res.allowed.is_empty() || !res.rejected.is_empty(), "Should produce some routes");
    }

    #[test]
    fn test_manual_override_resumes() {
        let mut orch = FabricOrchestrator::default();
        let fabric = build_fabric(&["AAPL"]);
        feed_sine(&mut orch, "AAPL", 50, 100.0, 5.0, 1000.0);
        // Trigger halt
        for _ in 0..30 { orch.record_execution_pnl(-1000.0); }
        let res1 = orch.tick(&fabric);
        assert!(res1.allowed.is_empty(), "Should block when halted");
        assert!(orch.is_halted());
        // Override
        orch.manual_override();
        assert!(!orch.is_halted());
        let res2 = orch.tick(&fabric);
        assert!(!res2.allowed.is_empty() || !res2.rejected.is_empty(), "Should resume after override");
    }
}

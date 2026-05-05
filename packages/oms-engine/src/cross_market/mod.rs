pub mod bam_integration;
pub mod correlation_matrix;
pub mod cross_layer_fusion;
pub mod deterministic_engine;
pub mod fabric_guard;
pub mod fabric_orchestrator;
pub mod funnel_index;
pub mod liquidity_tracker;
pub mod market_fabric;
pub mod multi_market_grid;
pub mod noise_filter;
pub mod pattern_detector;
pub mod pattern_layers;
pub mod pattern_matcher;
pub mod regime_detection;
pub mod ripple_sync;
pub mod schema_registry;
pub mod signal_fusion;
pub mod time_bounded_router;
pub mod time_decay;
pub mod volatility_surface;
pub mod weight_engine;

pub use bam_integration::{
    AsymmetryPattern, BamCrossMarketIntegration, BamDomain, BamLayer, BamSignal,
};
pub use correlation_matrix::{CorrelationAdjustment, CorrelationParams, CrossCorrelationMatrix};
pub use cross_layer_fusion::{CrossLayerFusion, FusedSignal, FusionWeights};
pub use deterministic_engine::{DeterministicProfitEngine, ProfitEngineParams, TradeDecision};
pub use fabric_guard::{FabricGuard, GuardDecision, GuardParams, HaltLevel};
pub use fabric_orchestrator::{
    FabricOrchestrator, GuardedRoute, OrchestratorParams, OrchestratorResult,
};
pub use funnel_index::{
    BloomFilter16, CategoryIndex, ComparableMatch, FlashContainer, FunnelIndex, PatternImprint,
    PatternSignature,
};
pub use liquidity_tracker::{LiquidityAdjustment, LiquidityParams, LiquidityTracker};
pub use market_fabric::{AssetFabricState, FabricState, MarketFabric};
pub use multi_market_grid::{
    BamCell, BamGrid, MultiMarketBamGrids, PortfolioFabricAllocator, RebalanceOrder, GRID_CELLS,
    GRID_LEVELS, GRID_TIME_BUCKETS,
};
pub use noise_filter::{
    FilterAction, FilterStats, NoiseFilter, NoiseFilterParams, NoiseFilterResult, NoiseType,
};
pub use pattern_detector::{DetectedPattern, PatternDetector, PatternDetectorParams, PatternType};
pub use pattern_layers::{
    BottomLayerPattern, CrossLayerPattern, HorizontalLayerPattern, IndicativeLayerPattern,
    LayerPattern, LayerPatternDetection, MatchingLayerPattern, MiddleLayerPattern,
    PatternLayerEngine, PatternLayerParams, SqueezeLayerPattern, TopLayerPattern,
    VerticalLayerPattern,
};
pub use pattern_matcher::{CompiledCypherPattern, CypherPatternMatcher, PatternMatch};
pub use regime_detection::{
    MarketRegime, MarketState, RegimeBias, RegimeDetectionParams, RegimeDetectionResult,
    RegimeDetector,
};
pub use ripple_sync::{PriceTick, RipplePattern, RippleSyncEngine, RippleSyncParams, RippleType};
pub use schema_registry::{
    BamSchemaRegistry, CorrelationSchema, MarketClass, MarketSchema, TradingHours,
};
pub use signal_fusion::{
    AttributionMap, AttributionParams, AttributionTracker, BayesianBelief, BayesianUpdater,
    BayesianUpdaterParams, ConfidenceModel, ConfidenceParams, ConfidenceScore, DeterministicHasher,
    FusionParams, FusionResult, HashParams, HashResult, SignalFusionEngine,
};
pub use time_bounded_router::{PathType, RouteResult, RouterParams, TimeBoundedRouter};
pub use time_decay::{DecayAdjustment, DecayParams, TimeDecayModel};
pub use volatility_surface::{VolatilityAdjustment, VolatilitySurface, VolatilitySurfaceParams};
pub use weight_engine::{WeightEngine, WeightEngineParams, WeightVector};

pub mod regime_detection;
pub mod volatility_surface;
pub mod correlation_matrix;
pub mod liquidity_tracker;
pub mod time_decay;
pub mod weight_engine;

pub use regime_detection::{
    RegimeDetector, MarketRegime, RegimeBias, RegimeDetectionParams,
    MarketState, RegimeDetectionResult
};
pub use volatility_surface::{
    VolatilitySurface, VolatilitySurfaceParams, VolatilityAdjustment
};
pub use correlation_matrix::{
    CrossCorrelationMatrix, CorrelationParams, CorrelationAdjustment
};
pub use liquidity_tracker::{
    LiquidityTracker, LiquidityParams, LiquidityAdjustment
};
pub use time_decay::{
    TimeDecayModel, DecayParams, DecayAdjustment
};
pub use weight_engine::{
    WeightEngine, WeightVector, WeightEngineParams
};

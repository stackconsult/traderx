// Fabric module
// Phase A: Base Model Foundation - Pattern matching and base model
// Phase B: Advanced Model - Advanced model inference, divergence monitor, correlation updater

pub mod advanced_model;
pub mod base_model;
pub mod correlation_updater;
pub mod divergence_monitor;
pub mod funnel_index;
pub mod pattern_matcher;
pub mod ripple_compounder;

pub use advanced_model::{AdvancedModelConfig, AdvancedModelPipeline, InferenceResult};
pub use base_model::{BaseModel, BaseModelConfig};
pub use correlation_updater::{CorrelationEntry, CorrelationUpdater};
pub use divergence_monitor::{DivergenceConfig, DivergenceEvent, DivergenceMonitor};
pub use funnel_index::{FlashFunnel, FunnelIndex};
pub use pattern_matcher::{CompiledPattern, PatternMatcher};
pub use ripple_compounder::{RippleCompounder, RippleConfig};

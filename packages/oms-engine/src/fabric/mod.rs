// Fabric module
// Phase A: Base Model Foundation - Pattern matching and base model

pub mod base_model;
pub mod funnel_index;
pub mod pattern_matcher;
pub mod ripple_compounder;

pub use base_model::{BaseModel, BaseModelConfig};
pub use funnel_index::{FlashFunnel, FunnelIndex};
pub use pattern_matcher::{CompiledPattern, PatternMatcher};
pub use ripple_compounder::{RippleCompounder, RippleConfig};

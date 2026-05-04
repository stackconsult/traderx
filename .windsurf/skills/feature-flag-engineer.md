# Feature Flag Engineer Skill

## Trigger
Need to conditionally compile modules, reduce build scope, enable/disable features

## Action

### 1. Design Feature Flag Strategy
```toml
# Root Cargo.toml
[features]
default = ["core"]
core = ["oms-engine", "traderx-mem0-types", "portfolio-aggregation"]
hft = ["hft-system", "dealing-desk/ebpf-router"]
ai = ["ai-agents", "learnship"]
backtest = ["hft", "ai"]  # Backtest needs both
production = ["core", "hft"]
development = ["core", "ai", "backtest"]
```

### 2. Implement Conditional Compilation
```rust
// In modules that need feature gates
#[cfg(feature = "hft")]
pub mod hft_execution;

#[cfg(feature = "ai")]
pub mod ai_agents;

#[cfg(feature = "backtest")]
pub mod backtest_engine;

#[cfg(all(feature = "hft", feature = "ai"))]
pub mod hft_ai_integration;
```

### 3. Update Module Imports
```rust
// Use conditional imports
#[cfg(feature = "hft")]
use crate::hft_execution::HftExecutor;

#[cfg(feature = "ai")]
use crate::ai_agents::AiAgentManager;
```

### 4. Configure Build Profiles
```toml
# Profile-specific features
[profile.dev]
features = ["development"]

[profile.release]
features = ["production"]

[profile.test]
features = ["core", "backtest"]
```

### 5. Test Feature Combinations
```bash
# Test core only
cargo check --no-default-features --features core

# Test HFT
cargo check --no-default-features --features hft

# Test AI
cargo check --no-default-features --features ai

# Test full development
cargo check --features development
```

## Verification
- Core builds without features ✅
- HFT builds independently ✅
- AI builds independently ✅
- Combined features work ✅
- Build time reduced ✅

## Prevention Skills
- feature-flag-validator.md
- build-configuration-manager.md
- conditional-compilation-auditor.md

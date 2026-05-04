# Module Splitter Skill

**Unified Workflow Team:** Refactoring Specialist  
**Follows:** `.windsurf/workflows/unified-team-execution.md` — DIAGNOSE → PLAN → EXECUTE → VERIFY → COMMIT → HANDOFF

## Trigger

Monolithic modules >10KB, single responsibility violations, build performance issues

## Action

### 1. Identify Monolithic Modules

```bash
# Find large modules
find packages/oms-engine/src -name "*.rs" -exec wc -l {} + | sort -n | tail -10

# Check module complexity
cargo check --package oms-engine 2>&1 | grep "warning: large module"
```

### 2. Analyze Module Structure

- Identify distinct responsibilities
- Map dependencies between functions
- Find natural split points
- Preserve public API

### 3. Split Strategy

```rust
// Before: engineering_orchestra.rs (40KB)
pub mod engineering_orchestra;

// After: Split into focused modules
pub mod engineering_orchestra {
    pub mod execution_engine;     // Core execution logic
    pub mod resource_manager;     // Resource allocation
    pub mod performance_monitor;  // Metrics and monitoring
    pub mod error_handler;        // Error recovery
}
```

### 4. Maintain API Compatibility

```rust
// Re-export to maintain existing API
pub use execution_engine::ExecutionEngine;
pub use resource_manager::ResourceManager;
pub use performance_monitor::PerformanceMonitor;
pub use error_handler::ErrorHandler;

// Keep original struct if needed
pub struct EngineeringOrchestra {
    execution_engine: ExecutionEngine,
    resource_manager: ResourceManager,
    performance_monitor: PerformanceMonitor,
    error_handler: ErrorHandler,
}
```

### 5. Update Imports

```rust
// Update all imports to use new modules
use crate::engineering_orchestra::execution_engine::ExecutionEngine;
use crate::engineering_orchestra::resource_manager::ResourceManager;
```

### 6. Validate Split

```bash
# Ensure everything still compiles
cargo check --package oms-engine

# Check module sizes are reasonable
find packages/oms-engine/src/engineering_orchestra -name "*.rs" -exec wc -l {} +
```

## Verification

- All modules <10KB ✅
- Single responsibility principle ✅
- API compatibility maintained ✅
- No circular dependencies ✅
- Build time improved ✅

## Prevention Skills

- module-size-monitor.md
- dependency-analyzer.md
- api-compatibility-checker.md

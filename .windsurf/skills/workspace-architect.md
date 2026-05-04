# Workspace Architect Skill

**Unified Workflow Team:** Structure Designer  
**Follows:** `.windsurf/workflows/unified-team-execution.md` — DIAGNOSE → PLAN → EXECUTE → VERIFY → COMMIT → HANDOFF

## Trigger

Workspace organization issues, monolithic modules, build performance problems

## Action

### 1. Analyze Current Workspace Structure

```bash
# Check workspace members
cargo metadata --format-version 1 | jq '.workspace_members'
cargo tree --duplicates

# Analyze module sizes
find packages/oms-engine/src -name "*.rs" -exec wc -l {} + | sort -n | tail -10
```

### 2. Design Workspace Reorganization

#### Core Workspace (Always Built)

- oms-engine (core trading logic)
- traderx-mem0-types
- portfolio-aggregation

#### HFT Workspace (Optional)

- hft-system/*
- dealing-desk/ebpf-router

#### AI Workspace (Optional)

- ai-agents
- learnship

### 3. Implement Feature Flags

```toml
# Root Cargo.toml
[features]
default = ["core"]
core = []
hft = ["hft-system", "dealing-desk"]
ai = ["ai-agents", "learnship"]
backtest = ["hft"]  # Backtest needs HFT
```

### 4. Split Monolithic Modules

- Identify modules >10KB
- Split by responsibility
- Maintain API compatibility
- Update imports

### 5. Optimize Build Performance

- Configure workspace-specific profiles
- Set up sccache for distributed builds
- Add cargo-nextest for faster testing
- Configure cargo-watch for development

## Verification

- Build time < 10 seconds ✅
- Workspace structure logical ✅
- Feature flags working ✅
- No circular dependencies ✅

## Prevention Skills

- module-size-monitor.md
- dependency-analyzer.md
- build-profiler.md
- feature-flag-manager.md

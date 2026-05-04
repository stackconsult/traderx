# Build Optimization Team Workflow

## Description
Specialized team workflow for comprehensive build optimization, workspace reorganization, and performance improvements.

## Team Composition

### 1. Workspace Architect
**Role**: Design and implement workspace reorganization
- Analyze current workspace structure
- Design core/hft/ai workspace separation
- Implement feature flags
- Manage workspace dependencies

### 2. Feature Flag Engineer
**Role**: Implement conditional compilation
- Design feature flag strategy
- Add feature gates to modules
- Configure build profiles
- Test feature combinations

### 3. Module Splitter
**Role**: Break down monolithic modules
- Identify modules >10KB
- Split by single responsibility
- Maintain API compatibility
- Update imports and dependencies

### 4. Tooling Specialist
**Role**: Optimize development and build tools
- Install cargo-nextest and cargo-watch
- Configure sccache for distributed builds
- Set up pre-commit hooks
- Benchmark build performance

### 5. Build System Engineer
**Role**: Overall build optimization
- Apply profile optimizations
- Manage dependencies
- Monitor build metrics
- Coordinate team efforts

## Execution Flow

### Phase 1: Analysis and Planning
```bash
# Workspace Architect: Analyze current structure
find packages/oms-engine/src -name "*.rs" -exec wc -l {} + | sort -n | tail -10
cargo metadata --format-version 1 | jq '.workspace_members'

# Build System Engineer: Benchmark current performance
hyperfine --warmup 3 'cargo check --package oms-engine'
cargo check --package oms-engine 2>&1 | grep "^warning" | wc -l
```

### Phase 2: Workspace Reorganization
- Workspace Architect designs core/hft/ai workspaces
- Feature Flag Engineer implements feature gates
- Module Splitter identifies monolithic modules
- Tooling Specialist installs optimization tools

### Phase 3: Module Optimization
- Module Splitter breaks down large modules
- Feature Flag Engineer adds conditional compilation
- Build System Engineer applies profile optimizations
- Workspace Architect validates structure

### Phase 4: Tooling and Performance
- Tooling Specialist configures cargo-nextest/watch
- Build System Engineer sets up sccache
- Feature Flag Engineer tests feature combinations
- Workspace Architect validates final structure

### Phase 5: Verification and Monitoring
- All specialists validate their areas
- Build System Engineer benchmarks final performance
- Team documents improvements
- Set up ongoing monitoring

## Current Build Plan Analysis

### Issues Identified:
1. **Monolithic modules** (engineering_orchestra.rs ~40KB)
2. **No feature flags** (everything compiled always)
3. **Slow build times** (~40s for cargo check)
4. **Excessive warnings** (150 warnings)
5. **No workspace separation** (HFT/AI excluded but not organized)

### Optimization Targets:
- Build time: 40s → 10s (4x improvement)
- Warnings: 150 → 25 (6x improvement)
- Module size: <10KB per module
- Workspace: 3 focused workspaces

## Success Metrics
- Build time < 10 seconds ✅
- Warnings < 25 ✅
- All modules <10KB ✅
- Feature flags working ✅
- Tooling installed ✅

## Prevention Strategies
- Module size monitoring
- Warning budget enforcement
- Build time alerts
- Feature flag validation
- Automated tooling setup

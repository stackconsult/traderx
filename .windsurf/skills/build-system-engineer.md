# Build System Engineer Skill

## Trigger
Build system issues, compilation errors, performance problems

## Action

### 1. Diagnose Build Issues
```bash
# Check compilation status
cargo check --package oms-engine 2>&1 | grep "^error" | wc -l
cargo check --package oms-engine 2>&1 | grep "^warning" | wc -l

# Check build time
time cargo check --package oms-engine

# Check disk space
df -h /Users/kirtissiemens/CascadeProjects
du -sh target/
```

### 2. Apply Optimizations

#### Profile Configuration
```toml
[profile.dev]
incremental = true
codegen-units = 256

[profile.dev-optimized]
inherits = "dev"
opt-level = 1
codegen-units = 16
```

#### Dependency Management
```bash
# Audit dependencies
cargo tree --duplicates
cargo outdated

# Fix unused imports
cargo fix --package oms-engine --allow-dirty
```

#### Cache Management
```bash
# Clean if disk full
cargo clean

# Use sccache for distributed builds
export RUSTC_WRAPPER=sccache
```

### 3. Monitor Performance
- Track build times
- Monitor warning counts
- Measure disk usage
- Check incremental compilation effectiveness

### 4. Prevent Recurrence
- Add build checks to CI/CD
- Configure automated cleanup
- Set up build time alerts
- Document optimization patterns

## Verification
- Build time < 20 seconds ✅
- Warnings < 50 ✅
- Disk usage < 2GB ✅
- Incremental builds working ✅

## Prevention Skills
- profile-optimization.md
- dependency-management.md
- cache-strategy.md
- build-monitoring.md

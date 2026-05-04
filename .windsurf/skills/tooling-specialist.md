# Tooling Specialist Skill

## Trigger
Need for faster builds, better testing, development workflow improvements

## Action

### 1. Install Essential Tools
```bash
# Install faster test runner
cargo install cargo-nextest

# Install file watcher for development
cargo install cargo-watch

# Install distributed compilation cache
cargo install sccache

# Install build benchmarking
cargo install hyperfine
```

### 2. Configure Development Tools
```bash
# Set up sccache for distributed builds
export RUSTC_WRAPPER=sccache
export SCCACHE_CACHE_SIZE="10G"
export SCCACHE_DIR="/tmp/sccache"

# Configure cargo-watch for development
cargo watch -x 'check --package oms-engine'
cargo watch -x 'test --package oms-engine'
cargo watch -x 'run --bin main'
```

### 3. Optimize Build Profiles
```toml
# Add to Cargo.toml
[profile.dev]
incremental = true
codegen-units = 256
debug = true

[profile.dev-optimized]
inherits = "dev"
opt-level = 1
codegen-units = 16

[profile.test]
inherits = "dev"
opt-level = 1

[profile.bench]
debug = true
```

### 4. Set Up Pre-commit Hooks
```bash
# .git/hooks/pre-commit
#!/bin/bash
cargo check --package oms-engine
cargo fmt --package oms-engine
cargo clippy --package oms-engine
```

### 5. Benchmark Build Performance
```bash
# Benchmark current build time
hyperfine --warmup 3 'cargo check --package oms-engine'

# Compare with optimizations
hyperfine --warmup 3 'cargo check --package oms-engine --profile dev-optimized'

# Test parallel compilation
hyperfine --warmup 3 'cargo check --package oms-engine -j $(nproc)'
```

### 6. Configure CI/CD Optimizations
```yaml
# .github/workflows/build.yml
- name: Cache cargo registry
  uses: actions/cache@v3
  with:
    path: ~/.cargo/registry
    key: cargo-registry-${{ runner.os }}-${{ hashFiles('**/Cargo.lock') }}

- name: Cache cargo index
  uses: actions/cache@v3
  with:
    path: ~/.cargo/git
    key: cargo-index-${{ runner.os }}-${{ hashFiles('**/Cargo.lock') }}
```

## Verification
- cargo-nextest runs tests faster ✅
- cargo-watch detects changes ✅
- sccache improves compilation speed ✅
- Build time < 10 seconds ✅
- Pre-commit hooks work ✅

## Prevention Skills
- build-profiler.md
- ci-optimizer.md
- dev-workflow-automator.md

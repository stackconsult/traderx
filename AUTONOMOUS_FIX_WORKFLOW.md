# Autonomous Failure Fix Workflow

## Learning-Based Failure Resolution

### **Philosophy**: Every failure teaches. Document, fix, prevent.

---

## 🔍 Step 1: DETECT

**Commands to detect failures**:
```bash
# Check PR checks status
gh pr checks 2

# Check latest Actions runs
gh run list --limit 5

# Check specific run
gh run view <run-id>

# Local verification
cargo check --all
cargo test --no-run
cargo clippy -- -D warnings
cargo audit
```

**What to look for**:
- ❌ Red X marks on checks
- "failure" in run status
- Exit code 1 in jobs
- Vulnerability reports
- Compilation errors
- Test failures

---

## 🧠 Step 2: ANALYZE

### **Security Audit Failure**:
```
Pattern: RUSTSEC-XXXX-XXXX
Source: cargo audit
Fix: Add explicit patched dependency
```

### **Compilation Error**:
```
Pattern: error[E0XXX]: ...
Source: cargo check/build
Fix: Fix code or add missing dependency
```

### **Test Failure**:
```
Pattern: test XXX ... FAILED
Source: cargo test
Fix: Fix test or implementation
```

### **Clippy Warning**:
```
Pattern: warning: ...
Source: cargo clippy
Fix: Apply suggestion or allow lint
```

---

## 🛠️ Step 3: FIX

### **Security Vulnerability Fix**:
```toml
# Add to Cargo.toml
[dependencies]
# Security fix: RUSTSEC-XXXX-XXXX
vulnerable-crate = ">=patched.version"
```

### **Test Fix**:
```rust
// Add missing import
use crate::missing_module;

// Or add dev-dependency
// Cargo.toml: missing-crate = "x.y"
```

### **Clippy Fix**:
```rust
// Before (warning)
let x = y.clone();

// After (fixed)
let x = y;
```

---

## ✅ Step 4: VERIFY

```bash
# Re-run failing check locally
cargo audit
cargo check --all
cargo test --package <package>
cargo clippy -- -D warnings

# If all pass locally, push and monitor
```

---

## 📝 Step 5: COMMIT

```bash
# Clear commit message
git add <files>
git commit -m "fix: <failure description>

- <specific change 1>
- <specific change 2>

<reference to advisory/error>"

git push origin <branch>
```

---

## 🎓 Step 6: LEARN

### **Document in Mistake Journal**:
```markdown
## Failure: <brief description>
**Date**: YYYY-MM-DD
**Cause**: <root cause>
**Fix**: <solution>
**Prevention**: <how to avoid>
```

### **Add to Skills**:
```markdown
## Skill: Fix <failure type>
**Trigger**: <detection pattern>
**Fix**: <solution steps>
**Verification**: <check commands>
```

---

## 🔄 Autonomous Execution

### **Self-Healing Loop**:
```python
while True:
    # 1. Detect
    failures = check_pr_status()
    
    if not failures:
        print("All checks passing ✅")
        break
    
    # 2. For each failure
    for failure in failures:
        # 3. Analyze
        cause = analyze_failure(failure)
        
        # 4. Fix
        fix = generate_fix(cause)
        apply_fix(fix)
        
        # 5. Verify
        if verify_fix(failure):
            commit_fix(failure, fix)
            learn(cause, fix)
        else:
            escalate(failure)
```

---

## 📊 Current Status Check

**Executing detection commands...**

Results will be analyzed and fixes applied automatically.

---

**Learning mode: ACTIVE**  
**Autonomous fix: ENABLED**  
**Target: Zero failing checks**

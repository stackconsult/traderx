# Compilation Errors Found - Master Hardening Phase 1

**Date**: 2026-04-15 21:18 UTC-6  
**Status**: AUDIT COMPLETE - FIXING NOW  

---

## 🚨 ERRORS IDENTIFIED

### **Error 1: UnixListener Not Available on Windows**
```rust
File: packages/oms-engine/src/signal_router.rs:19:5
Error: no `UnixListener` in `net`
Help: a similar name exists: `TcpListener`
```

**Root Cause**: `tokio::net::UnixListener` is Unix-specific, not available on Windows  
**Fix**: Use conditional compilation or replace with TcpListener for cross-platform

**Engineering Solution**:
```rust
#[cfg(unix)]
use tokio::net::UnixListener;
#[cfg(not(unix))]
use tokio::net::TcpListener as UnixListener; // Or handle differently
```

---

### **Error 2: Moved Value in WAL Recovery**
```rust
File: packages/portfolio-aggregation/src/engine.rs:358:60
Error: borrow of moved value: `events`
Note: `into_iter` takes ownership
```

**Root Cause**: `events` is moved in `for event in events` loop, then used again  
**Fix**: Iterate by reference: `for event in &events`

**Engineering Solution**:
```rust
// Change line 351
for event in &events {  // Add & to borrow instead of move
```

---

### **Error 3 & 4: AeronRsError Not Found**
```rust
File: packages/oms-engine/src/aeron_journal.rs:163:31
File: packages/oms-engine/src/aeron_journal.rs:166:31
Error: could not find `AeronRsError` in `aeron_rs`
```

**Root Cause**: Error type name changed or doesn't exist in aeron_rs crate  
**Fix**: Check aeron_rs documentation for correct error type

**Engineering Solution**:
```rust
// Check what error types aeron_rs actually exports
// Use correct error type name
```

---

### **Error 5: Deserialize Not Implemented**
```rust
File: packages/portfolio-aggregation/src/persistence.rs:71:42
Error: trait bound `AggregatorEvent: serde::Deserialize` not satisfied
Note: consider adding `#[derive(serde::Deserialize)]`
```

**Root Cause**: `AggregatorEvent` enum missing Deserialize derive  
**Fix**: Add derive macro

**Engineering Solution**:
```rust
// In engine.rs line 90
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]  // Add Deserialize
pub enum AggregatorEvent {
```

---

## 🔧 SYSTEMATIC FIX PLAN

### **Priority Order** (Critical First):

1. **AggregatorEvent Deserialize** (Error 5) - Affects persistence
2. **Moved value fix** (Error 2) - Affects WAL recovery  
3. **AeronRsError** (Errors 3-4) - Affects journal logging
4. **UnixListener** (Error 1) - Platform compatibility

### **Fix Workflow**:

```
For Each Error:
  1. Read file at error location
  2. Understand root cause
  3. Engineer minimal fix
  4. Test compilation
  5. Commit with clear message
  6. Push to origin
  7. Monitor Actions
  8. Update JOURNAL.md
```

---

## ✅ COMPLETION CRITERIA

### **Compilation**:
- [ ] cargo check passes (0 errors)
- [ ] cargo clippy passes (0 warnings, 0 errors)
- [ ] cargo test passes (90%+ tests)

### **Security**:
- [ ] cargo audit installed and running
- [ ] No RUSTSEC vulnerabilities
- [ ] Branch protection enabled

### **Documentation**:
- [ ] All fixes documented
- [ ] JOURNAL.md updated
- [ ] Proof artifacts created

---

**Next**: Execute fixes in priority order, starting with AggregatorEvent Deserialize  
**ETA**: 20-30 minutes for all fixes  
**Method**: Immutable commits, full documentation  

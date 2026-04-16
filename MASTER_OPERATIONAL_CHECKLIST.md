# MASTER OPERATIONAL CHECKLIST

**Status**: ACTIVE - UPDATED CONTINUOUSLY  
**Last Updated**: 2026-04-15 21:40 UTC-6  
**Rule**: UPDATE THIS FILE BEFORE AND AFTER EVERY ACTION  

---

## 🚨 ABSOLUTE LAWS (Never Violate)

### Law 1: GitHub is Truth
- Local state is ephemeral
- GitHub online repo is the ONLY source of truth
- **VERIFY**: `python scripts/github_sync_check.py` before EVERY action

### Law 2: Workflow Compliance
- Check `/preflight-checklist` before session start
- Check `/session-start` for initialization
- Check relevant workflow before every action type

### Law 3: Continuous Journaling
- Update `JOURNAL.md` after every significant action
- Document decisions, fixes, learnings
- Include commit hashes, error details, recovery steps

### Law 4: Immutable Commits
- Commit after every meaningful change
- Push immediately (never > 5 minutes)
- Use clear commit messages with context

### Law 5: No Vibe Coding
- Every line must be justified
- Reference spec/workflow/skill before coding
- Test after every change

---

## 📋 PRE-ACTION CHECKLIST (Execute Before EVERY Action)

### Step 1: GitHub Sync Verification (5 seconds)
```bash
python scripts/github_sync_check.py
```
**Status**: [ ] Verified  
**If Fails**: STOP → Fix sync → Re-verify → Continue

### Step 2: Context Awareness (10 seconds)
- [ ] Review `JOURNAL.md` last 3 entries
- [ ] Check `MASTER_HARDENING_EXECUTIVE_REPORT.md` for current status
- [ ] Confirm current phase (Phase 1/2/3/4/5)

### Step 3: Workflow Reference (15 seconds)
**Identify action type and check relevant workflow**:

| Action Type | Required Workflow | Location |
|-------------|-------------------|----------|
| Session Start | `/session-start` | `.windsurf/workflows/session-start.md` |
| Pre-Flight | `/preflight-checklist` | `.windsurf/workflows/preflight-checklist.md` |
| Git Operations | `github-sync-safeguard` | `.windsurf/workflows/github-sync-safeguard.md` |
| Code Changes | `production-guard` | `.windsurf/workflows/production-guard.md` |
| Testing | `quality-guardian` | `.windsurf/workflows/quality-guardian.md` |
| GitHub MCP | `github-mcp-integration` | `.windsurf/workflows/github-mcp-integration.md` |
| Self-Healing | `github-first-self-healing` | `.windsurf/workflows/github-first-self-healing.md` |
| Upskilling | `autonomous-upskilling` | `.windsurf/workflows/autonomous-upskilling.md` |
| Meta-Cognitive | `meta-cognitive-improvement` | `.windsurf/workflows/meta-cognitive-improvement.md` |
| Repository Audit | `repository-audit` | `.windsurf/workflows/repository-audit.md` |
| Validation | `validation-gate` | `.windsurf/workflows/validation-gate.md` |
| Task Refinement | `task-refinement` | `.windsurf/workflows/task-refinement.md` |
| Hardening | `master-hardening-engineering` | `.windsurf/workflows/master-hardening-engineering.md` |

**Status**: [ ] Workflow referenced  
**Skill Applied**: _______________

### Step 4: Safety Gates (5 seconds)
- [ ] No rebase in progress (`git status`)
- [ ] On correct branch (`git branch --show-current`)
- [ ] No uncommitted critical work

### Step 5: Action Specification (10 seconds)
**Document what you're about to do**:
- **Action**: _____________________________________
- **Purpose**: ___________________________________
- **Expected Outcome**: ___________________________
- **Rollback Plan**: _____________________________

---

## 🔧 ACTION-SPECIFIC WORKFLOWS

### For Git Operations
**Required**: `github-sync-safeguard.md`

Checklist:
- [ ] Pre-flight sync check passed
- [ ] No rebase/merge in progress
- [ ] On valid branch
- [ ] Post-push verification within 1 minute

### For Code Changes (Compilation Fixes)
**Required**: `production-guard.md` + `quality-guardian.md`

Checklist:
- [ ] Identify error category (Send/Type/Impl/Warning)
- [ ] Check systematic fix tracker
- [ ] Make minimal focused change
- [ ] Run `cargo check` after change
- [ ] Run `cargo clippy` for quality
- [ ] Commit with clear message
- [ ] Push immediately

### For Documentation
**Required**: `meta-cognitive-improvement.md`

Checklist:
- [ ] Document context
- [ ] Record decisions
- [ ] Update relevant tracking files
- [ ] Cross-reference commits

### For GitHub API Operations
**Required**: `github-mcp-integration.md`

Checklist:
- [ ] Verify GitHub MCP configured
- [ ] Check token validity
- [ ] Verify API rate limits
- [ ] Log all API calls

### For Emergency Recovery
**Required**: `github-first-self-healing.md`

Checklist:
- [ ] Diagnose root cause
- [ ] Document failure mode
- [ ] Apply recovery procedure
- [ ] Verify restoration
- [ ] Update safeguards

---

## ✅ POST-ACTION CHECKLIST (Execute After EVERY Action)

### Step 1: Verification (10 seconds)
- [ ] Action completed successfully
- [ ] Expected outcome achieved
- [ ] No errors generated

### Step 2: GitHub Sync (CRITICAL - 10 seconds)
```bash
git add -A
git commit -m "<type>: clear description"
git push origin main
python scripts/github_sync_check.py
```
**Status**: [ ] Sync verified  
**Commit Hash**: _______________

### Step 3: Documentation Update (15 seconds)
**Update ALL relevant files**:
- [ ] `JOURNAL.md` - Add entry with context
- [ ] `MASTER_OPERATIONAL_CHECKLIST.md` - Update this file
- [ ] `SYSTEMATIC_FIX_TRACKER.md` - If fixing errors
- [ ] `PRODUCTION_HARDENING_STATUS.md` - If hardening work

### Step 4: Skill Learning Capture (10 seconds)
- [ ] What worked? ______________________________
- [ ] What didn't? _____________________________
- [ ] New pattern discovered? _________________
- [ ] Upskill opportunity? ______________________

### Step 5: Next Action Planning (5 seconds)
**Identify next step**:
- [ ] Continue current batch
- [ ] Switch to new batch
- [ ] Emergency intervention needed
- [ ] Session complete

---

## 📊 CURRENT SESSION STATUS

### Environment
- **IDE**: Windsurf ✅
- **MCP Config**: ✅ Present
- **GitHub Token**: ✅ Set
- **Branch**: main ✅
- **Sync Status**: ✅ Verified

### Workflows Verified Present
- [x] `agent-execution-engine.md`
- [x] `autonomous-upskilling.md`
- [x] `github-first-self-healing.md`
- [x] `github-mcp-integration.md`
- [x] `github-sync-safeguard.md` ✅ CRITICAL
- [x] `master-hardening-engineering.md`
- [x] `meta-cognitive-improvement.md`
- [x] `preflight-checklist.md` ✅ CRITICAL
- [x] `production-guard.md`
- [x] `quality-guardian.md`
- [x] `repository-audit.md`
- [x] `session-start.md` ✅ CRITICAL
- [x] `validation-gate.md`

### Current Phase
**Master Hardening - Phase 2: Type System Fixes**
- Batch 1: ✅ Complete (5 errors fixed)
- Batch 2: ✅ Complete (Send trait - architecture rewrite)
- Batch 3: ⏳ IN PROGRESS (~40 type mismatch errors)
- Batch 4: ⏳ PENDING (~30 missing implementations)
- Batch 5: ⏳ PENDING (51 warnings)

### Last Actions Log
1. ✅ Fixed GitHub sync issue (rebase blocking)
2. ✅ Created safeguard system (github_sync_check.py)

---

## NEXT IMMEDIATE ACTIONS

### Priority 1: Re-apply Lost Compilation Fixes  COMPLETE
**Workflow**: `production-guard.md`  
**Status**: 5/5 files complete - BATCH 1 DONE

**Files Fixed**:
1.  `disruptor.rs` - Replace rtrb with tokio mpsc (COMMITTED)
2.  `Cargo.toml` - Remove rtrb dependency (COMMITTED)
3.  `aeron_journal.rs` - Error handling (FIXED & COMMITTED)
4.  `signal_router.rs` - UnixListener conditional compilation (FIXED & COMMITTED)
5.  `engine.rs` - AggregatorEvent derive (FIXED & COMMITTED)

**Last Commit**: `8734347` - "fix: complete Batch 1 - all 5 critical compilation errors"
**Next**: Continue with Batch 2 (Send trait issues) - DISRUPTOR ALREADY DONE

### Priority 2: Continue Master Hardening Phase 2
**Workflow**: `master-hardening-engineering.md`  
**Status**: Batch 1 , Batch 2 , Batch 3 

**Batch 3: Type Mismatch Fixes**
- Estimated: ~40 errors remaining
- Target: Add missing trait derives (Eq, Hash, etc.)
- Target: Fix Decimal/f64 conversions
- Method: Systematic batch fixes

**Pre-Action** (for Batch 3):
- [x] Run sync check 
- [ ] Read master-hardening-engineering workflow
- [ ] Document action
- [ ] Execute fix
- [ ] Verify with cargo check
- [ ] Commit & push
- [ ] Update checklist
- [ ] Document action in checklist

**Post-Action**:
- [ ] Run cargo check
- [ ] Commit & push
- [ ] Update JOURNAL.md
- [ ] Update this checklist

### Priority 2: Continue Batch 3 (Type Mismatches)
**Workflow**: `master-hardening-engineering.md`  
**Estimated**: 30 minutes, ~40 errors  
**Method**: Systematic batch fixes

### Priority 3: Documentation Updates
**Workflow**: `meta-cognitive-improvement.md`  
- Update all status files
- Record learning
- Update progress metrics

---

## 📁 ACTIVE DOCUMENTS REFERENCE

| Document | Purpose | Last Update |
|----------|---------|-------------|
| `JOURNAL.md` | Session log | 2026-04-15 21:40 |
| `MASTER_HARDENING_EXECUTIVE_REPORT.md` | Status report | 2026-04-15 21:30 |
| `PRODUCTION_HARDENING_STATUS.md` | Progress tracking | 2026-04-15 21:35 |
| `SYSTEMATIC_FIX_TRACKER.md` | Error batches | 2026-04-15 21:35 |
| `MASTER_OPERATIONAL_CHECKLIST.md` | This file | 2026-04-15 21:40 |
| `AGENT_WORK_AUDIT_AND_UPSKILLING.md` | Learning log | 2026-04-15 21:20 |

---

## 🔄 CONTINUOUS UPDATE RULE

**BEFORE every action**:
1. Read this checklist
2. Check off pre-action steps
3. Specify action
4. Reference workflow

**AFTER every action**:
1. Check off post-action steps
2. Update this file with results
3. Update JOURNAL.md
4. Verify GitHub sync

**EVERY 5 minutes**:
1. Run `python scripts/github_sync_check.py`
2. Verify no accumulated unpushed work
3. Update progress in this file

---

## ✅ CHECKLIST INTEGRATION WITH COMMANDS

### Before EVERY terminal command:
```bash
# 1. Sync check
python scripts/github_sync_check.py

# 2. Read this checklist (mentally)
# 3. Reference relevant workflow
# 4. Execute command
# 5. Update checklist
```

### Before EVERY file edit:
```bash
# 1. Sync check
python scripts/github_sync_check.py

# 2. Read relevant workflow section
# 3. Document change purpose
# 4. Make minimal edit
# 5. Test if applicable
# 6. Commit & push
# 7. Update checklist & journal
```

### Before EVERY workflow invocation:
```bash
# 1. Sync check
python scripts/github_sync_check.py

# 2. Read workflow file completely
# 3. Verify prerequisites
# 4. Execute workflow
# 5. Document outcomes
# 6. Update this checklist
```

---

## 🎓 SKILL REFERENCE

### Skills Directory: `.windsurf/skills/`

**Agentic Learning** (`agentic-learning/`):
- Research methodology
- Learning capture
- Pattern recognition

**Impeccable** (`impeccable/`):
- Error recovery
- Quality assurance
- Production practices

### When to Use Which Skill:

| Situation | Skill | Action |
|-----------|-------|--------|
| Unclear how to proceed | `agentic-learning/RESEARCH.md` | Research pattern |
| Error encountered | `impeccable/ERROR_RECOVERY.md` | Apply recovery |
| Quality concerns | `impeccable/QUALITY_ASSURANCE.md` | Run checks |
| Documentation needed | `agentic-learning/DOCUMENTATION.md` | Capture learning |

---

**STATUS**: ✅ CHECKLIST ACTIVE  
**NEXT**: Re-apply lost compilation fixes using this checklist  
**GUARDIAN**: github_sync_check.py + this checklist + JOURNAL.md  

**I WILL UPDATE THIS FILE BEFORE AND AFTER EVERY ACTION.**

---
description: Binary validation checklist before ANY work begins. All items must pass (✅) or workflow cannot proceed
---

# preflight-checklist

**ABSOLUTE REQUIREMENT**: This checklist MUST be completed at session start.

**Purpose**: Ensure zero ambiguity, zero missing guardrails, and absolute readiness.

---

## Section A: Environment Readiness

| # | Check | Command/Method | Pass Criteria |
|---|-------|---------------|---------------|
| A1 | Windsurf IDE Active | Visual confirmation | IDE is running |
| A2 | MCP Config Present | `Test-Path ~/.windsurf/mcp_config.json` | Returns True |
| A3 | GitHub Token Set | `$env:GITHUB_TOKEN -ne $null` | Returns True |
| A4 | GitHub Token Valid | API test via curl | Returns 200 OK |
| A5 | In Correct Branch | `git branch --show-current` | Returns `feature/github-mcp-setup` |
| A6 | Git Clean | `git status --short` | Returns empty |

**Section A Status**: [ ] ALL PASS [ ] HAS FAILURES

---

## Section B: Skills & Workflows Readiness

| # | Check | Location | Pass Criteria |
|---|-------|----------|---------------|
| B1 | Session Start Workflow | `.windsurf/workflows/session-start.md` | File exists |
| B2 | Sync Upstream Skills | `.windsurf/workflows/sync-upstream-skills.md` | File exists |
| B3 | Preflight Checklist | `.windsurf/workflows/preflight-checklist.md` | File exists |
| B4 | Agent Handoff Skill | `.windsurf/skills/agent-handoff.md` | File exists |
| B5 | Audit Compliance Skill | `.windsurf/skills/audit-compliance.md` | File exists |
| B6 | HSTR Orchestrator | `.windsurf/skills/hstr-orchestrator.md` | File exists |
| B7 | Agentic Learning Dir | `.windsurf/skills/agentic-learning/` | Dir exists with SKILL.md |
| B8 | Impeccable Skills Dir | `.windsurf/skills/impeccable/` | Dir exists with 21 sub-skills |

**Section B Status**: [ ] ALL PASS [ ] HAS FAILURES

**If B7 or B8 Fail**: Execute `/sync-upstream-skills` immediately.

---

## Section C: Previous Session Analysis

| # | Check | Method | Pass Criteria |
|---|-------|--------|---------------|
| C1 | Last JOURNAL Entry | Read `JOURNAL.md` | Entry exists within last session |
| C2 | Previous Work Graded | Grade: A/B/C/D/F | Grade assigned and documented |
| C3 | Proof Artifacts Present | Check `proofs/` directory | Relevant proofs from last work exist |
| C4 | No Uncommitted Critical Work | `git diff HEAD` | No critical uncommitted changes |

**Section C Status**: [ ] ALL PASS [ ] HAS FAILURES

---

## Section D: Repository Sync Status

| # | Check | Source | Action if Outdated |
|---|-------|--------|-------------------|
| D1 | agentic-learning Latest | `FavioVazquez/agentic-learning` | Execute `/sync-upstream-skills` |
| D2 | impeccable Latest | `pbakaus/impeccable` | Execute `/sync-upstream-skills` |
| D3 | Local vs Origin Match | `git log HEAD...origin/branch` | Push or pull as needed |
| D4 | No Merge Conflicts | `git merge-tree` | Resolve if present |

**Section D Status**: [ ] ALL PASS [ ] SYNC NEEDED [ ] HAS FAILURES

---

## Section E: Roadmap Clarity

| # | Check | Document | Must Be Clear |
|---|-------|----------|---------------|
| E1 | Current Phase | `MILESTONES.md` | Which milestone is active |
| E2 | Current Step | `IMPLEMENTATION_PLAN.md` | What phase/step is in progress |
| E3 | Next 3 Deliverables | Derived from plan | Explicitly listed |
| E4 | Blockers Identified | JOURNAL or issues | Known issues documented |
| E5 | Dependencies Ready | Check dependency status | All deps available/installed |

**Section E Status**: [ ] ALL CLEAR [ ] NEEDS CLARIFICATION

---

## Section F: Production Readiness

| # | Check | Standard | Validation |
|---|-------|----------|------------|
| F1 | No Pseudo-Code Policy | AGENTS.branch.mcp.md Law 2 | Acknowledged and understood |
| F2 | Testing During Build | Write tests with code | Confirmed as practice |
| F3 | Recursive Up-Engineering | Continuous improvement | Acknowledged as practice |
| F4 | Proof Artifacts Required | Every commit | Confirmed as practice |
| F5 | Multi-Persona Review | Before PR | Confirmed as practice |

**Section F Status**: [ ] ALL ACKNOWLEDGED [ ] NEEDS REVIEW

---

## Execution Script

```powershell
# Run this to execute all checks automatically

Write-Host "╔════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║          PREFLIGHT CHECKLIST EXECUTION                    ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan

$results = @{}

# Section A
Write-Host "`n📋 SECTION A: Environment Readiness" -ForegroundColor Yellow
$results.A1 = $true  # Visual confirmation
$results.A2 = Test-Path "$env:USERPROFILE\.windsurf\mcp_config.json"
$results.A3 = ![string]::IsNullOrEmpty($env:GITHUB_TOKEN)
$results.A4 = $false  # Requires API test
$results.A5 = (git branch --show-current) -eq "feature/github-mcp-setup"
$results.A6 = [string]::IsNullOrEmpty((git status --short))

foreach ($check in $results.GetEnumerator() | Where-Object {$_.Key -like "A*"}) {
    $status = if ($check.Value) { "✅ PASS" } else { "❌ FAIL" }
    Write-Host "  $($check.Key): $status"
}

# Section B
Write-Host "`n📋 SECTION B: Skills & Workflows" -ForegroundColor Yellow
$bChecks = @(
    @{ Name = "B1"; Path = ".windsurf/workflows/session-start.md" },
    @{ Name = "B2"; Path = ".windsurf/workflows/sync-upstream-skills.md" },
    @{ Name = "B3"; Path = ".windsurf/workflows/preflight-checklist.md" },
    @{ Name = "B4"; Path = ".windsurf/skills/agent-handoff.md" },
    @{ Name = "B5"; Path = ".windsurf/skills/audit-compliance.md" },
    @{ Name = "B6"; Path = ".windsurf/skills/hstr-orchestrator.md" }
)

foreach ($check in $bChecks) {
    $exists = Test-Path $check.Path
    $results[$check.Name] = $exists
    $status = if ($exists) { "✅ PASS" } else { "❌ FAIL" }
    Write-Host "  $($check.Name) ($($check.Path)): $status"
}

# Section C
Write-Host "`n📋 SECTION C: Previous Session Analysis" -ForegroundColor Yellow
$results.C1 = Test-Path "JOURNAL.md"
$results.C2 = $false  # Manual check required
$results.C3 = (Test-Path "proofs/") -and ((Get-ChildItem "proofs/").Count -gt 0)
$results.C4 = [string]::IsNullOrEmpty((git status --short))

Write-Host "  C1 (JOURNAL exists): $(if($results.C1){"✅"}else{"❌"})"
Write-Host "  C2 (Work graded): ⚠️ MANUAL CHECK REQUIRED"
Write-Host "  C3 (Proofs present): $(if($results.C3){"✅"}else{"❌"})"
Write-Host "  C4 (No uncommitted): $(if($results.C4){"✅"}else{"❌"})"

# Summary
Write-Host "`n═══════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "SUMMARY:" -ForegroundColor Cyan
$passed = ($results.Values | Where-Object { $_ -eq $true }).Count
$total = $results.Count
Write-Host "Passed: $passed / $total checks" -ForegroundColor $(if($passed -eq $total){"Green"}else{"Yellow"})

if ($passed -eq $total) {
    Write-Host "`n✅ ALL CHECKS PASSED - READY TO BUILD" -ForegroundColor Green
} else {
    Write-Host "`n⚠️  SOME CHECKS FAILED - REVIEW AND FIX" -ForegroundColor Yellow
}
```

---

## Binary Go/No-Go Decision

```
╔══════════════════════════════════════════════════════════════════╗
║              GO / NO-GO DECISION MATRIX                          ║
╠══════════════════════════════════════════════════════════════════╣
║                                                                  ║
║  ALL SECTIONS PASS →  ✅ GO - Proceed with production work       ║
║                                                                  ║
║  Section A Fail  →  ❌ NO-GO - Fix environment first             ║
║  Section B Fail  →  ❌ NO-GO - Run /sync-upstream-skills         ║
║  Section C Fail  →  ⚠️  CONDITIONAL - Commit or stash work       ║
║  Section D Fail  →  ⚠️  CONDITIONAL - Sync repositories            ║
║  Section E Fail  →  ❌ NO-GO - Clarify roadmap first             ║
║  Section F Fail  →  ❌ NO-GO - Review AGENTS.branch.mcp.md        ║
║                                                                  ║
╚══════════════════════════════════════════════════════════════════╝
```

---

## Post-Check Actions

### If ALL PASS
1. Log success in JOURNAL.md
2. Begin work with full guardrails active
3. Maintain production standards throughout

### If ANY FAIL
1. STOP - Do not proceed with new work
2. Address failures in order: A → B → C → D → E → F
3. Re-run checklist until ALL PASS
4. Only then begin work

---

## Integration with Session Start

This checklist is **automatically invoked** by `/session-start` workflow.

**Do not run independently** unless debugging a specific check.

---

## Success Criteria

Preflight is successful when:
- ✅ All automated checks pass
- ✅ Manual checks acknowledged
- ✅ Binary GO decision reached
- ✅ Ready for production work confirmed

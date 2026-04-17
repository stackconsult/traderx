---
description: Comprehensive repository audit workflow - automates branch analysis, gap identification, merge strategy determination, and production readiness assessment
---

# /repository-audit

**MANDATORY FOR**: Repository reviews, branch comparisons, merge preparation, production readiness checks

**Purpose**: Automate systematic repository analysis to identify gaps, assess code quality, and determine merge readiness through agentic production engineering practices.

---

## When to Execute

Execute this workflow when:
- [ ] Reviewing repository state before major changes
- [ ] Comparing branches for merge readiness
- [ ] Auditing code quality across all branches
- [ ] Preparing production readiness assessments
- [ ] After agent updates to verify changes
- [ ] Before merging feature branches to main

---

## Phase 1: Repository Structure Discovery (5 minutes)

### 1.1 Branch Inventory
```powershell
# List all branches (local and remote)
git branch -a

# Check branch relationships
git log --oneline --all --graph -20

# Identify unmerged branches
git branch -a --no-merged main
```

**Validation Gate**: Must identify ALL branches including:
- Local branches
- Remote tracking branches
- Feature branches
- Fix branches
- Stale/abandoned branches

### 1.2 Remote Configuration
```powershell
# Verify remote URLs
git remote -v

# Check fetch/push permissions
git remote show origin
```

### 1.3 Commit History Analysis
```powershell
# Analyze recent commits on all branches
git log --oneline --all -30

# Identify merge commits vs direct commits
git log --oneline --all --merges -10
git log --oneline --all --no-merges -10
```

---

## Phase 2: Per-Branch Deep Analysis (10 minutes per branch)

### 2.1 File Inventory
```powershell
# Count files by type
find . -type f -name "*.rs" | wc -l
find . -type f -name "*.py" | wc -l
find . -type f -name "*.md" | wc -l
find . -type f -name "*.toml" | wc -l
find . -type f -name "*.yml" -o -name "*.yaml" | wc -l

# Check for test files
find . -path "*/tests/*" -type f | wc -l
find . -name "*test*.py" -type f | wc -l
find . -name "*test*.rs" -type f | wc -l
```

### 2.2 Code Volume Metrics
```powershell
# Count lines of code (Rust)
find . -name "*.rs" -type f -exec cat {} \; | wc -l

# Count lines of code (Python)
find . -name "*.py" -type f -exec cat {} \; | wc -l

# Check package sizes
find packages -maxdepth 1 -type d | while read dir; do
    echo "$dir: $(find $dir -type f | wc -l) files"
done
```

### 2.3 Dependency Analysis
```powershell
# Check Cargo.toml workspace members
grep -A 30 "^\[workspace\]" Cargo.toml | grep "members"

# Verify all workspace members have Cargo.toml
for member in $(grep -o '"packages/[^"]*"' Cargo.toml | tr -d '"'); do
    if [ ! -f "$member/Cargo.toml" ]; then
        echo "❌ Missing: $member/Cargo.toml"
    fi
done

# Check Python requirements
find . -name "requirements.txt" -type f
```

### 2.4 Documentation State
```powershell
# Check for required documentation
$required_docs = @("README.md", "ARCHITECTURE.md", "MILESTONES.md", "AGENTS.md", "JOURNAL.md")
foreach ($doc in $required_docs) {
    if (Test-Path $doc) {
        $lines = (Get-Content $doc | Measure-Object).Count
        Write-Host "✅ $doc : $lines lines"
    } else {
        Write-Host "❌ $doc : MISSING"
    }
}
```

---

## Phase 3: Gap Identification (10 minutes)

### 3.1 Testing Gaps
```powershell
# Check for test coverage gaps
$packages = Get-ChildItem packages -Directory
foreach ($pkg in $packages) {
    $src_files = (Get-ChildItem $pkg.FullName -Recurse -Filter "*.py" -ErrorAction SilentlyContinue | Measure-Object).Count
    $test_files = (Get-ChildItem $pkg.FullName -Recurse -Filter "*test*" -ErrorAction SilentlyContinue | Measure-Object).Count
    
    if ($src_files -gt 0 -and $test_files -eq 0) {
        Write-Host "❌ $($pkg.Name): No tests ($src_files source files)"
    }
}
```

### 3.2 CI/CD Gaps
```powershell
# Check GitHub Actions
if (Test-Path ".github/workflows") {
    $workflows = Get-ChildItem .github/workflows -Filter "*.yml"
    Write-Host "Found $($workflows.Count) workflows:"
    foreach ($wf in $workflows) {
        Write-Host "  - $($wf.Name)"
    }
} else {
    Write-Host "❌ No .github/workflows directory"
}

# Check for pre-commit
if (Test-Path ".pre-commit-config.yaml") {
    Write-Host "✅ Pre-commit config present"
} else {
    Write-Host "❌ No pre-commit configuration"
}
```

### 3.3 Security Gaps
```powershell
# Check for hardcoded secrets
grep -r "password.*=" --include="*.yml" --include="*.yaml" --include="*.py" --include="*.rs" .
grep -r "api_key.*=" --include="*.py" --include="*.rs" .
grep -r "token.*=" --include="*.py" --include="*.rs" .

# Check .env handling
if (Test-Path ".env") {
    Write-Host "⚠️  .env file exists (should be gitignored)"
}
if (Test-Path ".env.example") {
    Write-Host "✅ .env.example present"
}
```

### 3.4 Documentation Gaps
```powershell
# Check for API documentation
find . -name "*.rs" -type f -exec grep -l "///" {} \; | wc -l

# Check for README in each package
find packages -maxdepth 1 -type d | while read dir; do
    if [ ! -f "$dir/README.md" ]; then
        echo "❌ Missing README: $dir"
    fi
done
```

---

## Phase 4: Branch Comparison (10 minutes)

### 4.1 Identify Changed Files
```powershell
# Compare branches
git diff main..HEAD --stat
git diff main..HEAD --name-only

# Check for deleted files
git diff main..HEAD --name-status | grep "^D"

# Check for new files
git diff main..HEAD --name-status | grep "^A"

# Check for renamed files
git diff main..HEAD --name-status | grep "^R"
```

### 4.2 Analyze Commit Quality
```powershell
# Check commit messages
git log main..HEAD --oneline

# Identify large commits
git log main..HEAD --stat | grep -A 5 "^[a-f0-9]" | head -50

# Check for merge commits
git log main..HEAD --merges --oneline
```

### 4.3 Cross-Branch File Differences
```powershell
# Files unique to current branch
git ls-tree -r HEAD --name-only | sort > current_files.txt
git ls-tree -r main --name-only | sort > main_files.txt
comm -23 current_files.txt main_files.txt > new_in_current.txt
comm -13 current_files.txt main_files.txt > removed_from_current.txt

Write-Host "Files unique to current branch:"
Get-Content new_in_current.txt

Write-Host "Files removed from current branch:"
Get-Content removed_from_current.txt
```

---

## Phase 5: Production Readiness Assessment (10 minutes)

### 5.1 Build Verification
```powershell
# Verify Rust compilation
cargo check --workspace 2>&1 | head -20
cargo test --workspace --no-run 2>&1 | head -20

# Check for compilation errors
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Compilation errors detected"
} else {
    Write-Host "✅ Compilation successful"
}
```

### 5.2 Test Execution
```powershell
# Run available tests
cargo test --workspace 2>&1 | tail -30

# Check test results
if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ All tests passed"
} else {
    Write-Host "❌ Test failures detected"
}
```

### 5.3 Documentation Completeness
```powershell
# Verify critical docs exist and are non-empty
$critical_docs = @(
    "README.md",
    "AGENTS.md",
    "docs/MCP_BRANCH_GOVERNANCE.md",
    ".windsurf/AGENTS.internal.md"
)

foreach ($doc in $critical_docs) {
    if (Test-Path $doc) {
        $size = (Get-Item $doc).Length
        if ($size -gt 100) {
            Write-Host "✅ $doc : $size bytes"
        } else {
            Write-Host "⚠️  $doc : Very short ($size bytes)"
        }
    } else {
        Write-Host "❌ $doc : MISSING"
    }
}
```

---

## Phase 6: Gap Classification & Prioritization (5 minutes)

### 6.1 Critical Gap Checklist
```markdown
## CRITICAL GAPS (Must Fix Before Production)

- [ ] Compilation errors exist
- [ ] No unit tests for critical components
- [ ] Hardcoded secrets/credentials
- [ ] No CI/CD pipeline configured
- [ ] Security vulnerabilities unaddressed
- [ ] Breaking changes not documented
- [ ] Missing required documentation
```

### 6.2 High Priority Gap Checklist
```markdown
## HIGH PRIORITY GAPS

- [ ] Incomplete test coverage (< 80%)
- [ ] No performance benchmarks
- [ ] Python/Rust linting not enforced
- [ ] Dependency vulnerabilities not scanned
- [ ] API documentation incomplete
- [ ] No integration tests
- [ ] Branch protection not configured
```

### 6.3 Scoring Matrix
```powershell
# Calculate production readiness score
$critical_gaps = 0
$high_gaps = 0
$medium_gaps = 0

# Score calculation
$total_score = 100
$total_score -= ($critical_gaps * 20)
$total_score -= ($high_gaps * 10)
$total_score -= ($medium_gaps * 5)

Write-Host "Production Readiness Score: $total_score%"

if ($total_score -ge 80) {
    Write-Host "✅ READY FOR PRODUCTION"
} elseif ($total_score -ge 60) {
    Write-Host "⚠️  CONDITIONAL - Address critical gaps first"
} else {
    Write-Host "❌ NOT READY - Significant work required"
}
```

---

## Phase 7: Merge Strategy Recommendation (5 minutes)

### 7.1 Conflict Analysis
```powershell
# Identify potential merge conflicts
git merge-tree $(git merge-base main HEAD) main HEAD 2>&1 | head -50

# Check for divergent file modifications
git diff main...HEAD --name-only
```

### 7.2 Recommendation Matrix
```markdown
## Merge Readiness Assessment

### Criteria:
- [ ] All compilation errors resolved
- [ ] Test coverage meets minimum (80%)
- [ ] Security scan passes
- [ ] Documentation complete
- [ ] No breaking changes OR breaking changes documented
- [ ] Code review completed
- [ ] Performance benchmarks acceptable

### Recommendation:
- **READY**: All criteria met
- **READY WITH NOTES**: Minor issues, documented
- **NOT READY**: Critical gaps exist
```

### 7.3 Pre-Merge Checklist
```powershell
# Final validation before merge
git status
git log --oneline -5

# Verify no uncommitted changes
if (git status --porcelain) {
    Write-Host "❌ Uncommitted changes present"
} else {
    Write-Host "✅ Working tree clean"
}

# Verify branch is up to date
git fetch origin
git status
```

---

## Phase 8: Report Generation (5 minutes)

### 8.1 Generate Audit Report
Create file: `AUDIT_[BRANCH]_[DATE].md`

```markdown
# Repository Audit Report

**Branch**: [current branch]
**Audit Date**: [date]
**Auditor**: Cascade
**Base Branch**: main

## Summary

### Branch State
- **Total Files**: [count]
- **Total Lines**: [count]
- **Test Files**: [count]
- **Documentation Files**: [count]

### Production Readiness
- **Score**: [X]%
- **Status**: [Ready/Conditional/Not Ready]
- **Critical Gaps**: [count]
- **High Priority Gaps**: [count]

### Merge Recommendation
- **Status**: [Ready/Not Ready]
- **Conditions**: [list]
- **Estimated Effort**: [hours]

## Detailed Findings

[Include all gaps identified]

## Recommended Actions

[Include prioritized action list]

## Proof Artifacts

- [ ] Compilation proof
- [ ] Test results proof
- [ ] Security scan proof
- [ ] Documentation proof
```

### 8.2 Update JOURNAL.md
```markdown
## [Date] - Repository Audit Completed

**Branch**: [branch]
**Purpose**: [audit reason]
**Outcome**: [summary]

**Key Findings**:
- [List top 3-5 findings]

**Actions Taken**:
- [List actions]

**Next Steps**:
- [List next steps]
```

---

## Quality Gates (Binary Validation)

### Gate 1: Branch Discovery Complete
- [ ] All local branches identified
- [ ] All remote branches identified
- [ ] Branch relationships mapped

### Gate 2: File Analysis Complete
- [ ] File counts by type documented
- [ ] Code volume metrics calculated
- [ ] Dependencies verified

### Gate 3: Gap Identification Complete
- [ ] Testing gaps documented
- [ ] CI/CD gaps documented
- [ ] Security gaps documented
- [ ] Documentation gaps documented

### Gate 4: Production Readiness Assessed
- [ ] Build verification passed
- [ ] Test execution completed
- [ ] Score calculated

### Gate 5: Merge Strategy Determined
- [ ] Conflicts analyzed
- [ ] Recommendation made
- [ ] Pre-merge checklist complete

### Gate 6: Report Generated
- [ ] Audit report created
- [ ] Journal updated
- [ ] Proof artifacts generated

---

## Execution Command

```powershell
# Execute full repository audit
/repository-audit --branch=current --base=main --output=AUDIT_REPORT.md
```

## Success Criteria

**Grade A**: All 6 gates pass, score ≥ 80%, 0 critical gaps  
**Grade B**: 5-6 gates pass, score ≥ 60%, ≤ 2 critical gaps  
**Grade C**: 3-4 gates pass, score ≥ 40%, ≤ 5 critical gaps  
**Grade D**: < 3 gates pass OR score < 40%  
**Grade F**: Major failures, cannot proceed

---

**ABSOLUTE REQUIREMENT**: All repository audits MUST use this workflow.  
**NO EXCEPTIONS**: Manual ad-hoc analysis is FORBIDDEN.  
**PROOF REQUIRED**: Audit report must be committed to `audits/` directory.

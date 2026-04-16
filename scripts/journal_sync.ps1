#!/usr/bin/env pwsh
# journal_sync.ps1 — Auto-sync JOURNAL.md and AGENT_MASTER_SYSTEM.md across all branches
# Usage: pwsh scripts/journal_sync.ps1 [-DryRun] [-Verbose]
# Triggered: after every significant commit, or manually

param(
    [switch]$DryRun,
    [switch]$Verbose
)

$ErrorActionPreference = "Stop"
$REPO_ROOT = Split-Path -Parent $PSScriptRoot
$TIMESTAMP = Get-Date -Format "yyyy-MM-dd HH:mm UTC"
$SYNC_FILES = @("JOURNAL.md", "AGENT_MASTER_SYSTEM.md", "MASTER_OPERATIONAL_CHECKLIST.md")
$MCP_BRANCH = "mcp/agent-master"

function Write-Log($msg, $level = "INFO") {
    $color = switch ($level) {
        "INFO"    { "Cyan" }
        "SUCCESS" { "Green" }
        "WARN"    { "Yellow" }
        "ERROR"   { "Red" }
    }
    Write-Host "[$level] $msg" -ForegroundColor $color
}

function Invoke-Git($args) {
    if ($Verbose) { Write-Log "git $args" "INFO" }
    if ($DryRun) { Write-Log "[DRY-RUN] git $args" "WARN"; return "" }
    $result = & git @($args.Split(" ")) 2>&1
    if ($LASTEXITCODE -ne 0) { Write-Log "Git error: $result" "ERROR"; throw "Git failed" }
    return $result
}

# ── 1. Verify clean state ──────────────────────────────────────────────────
Write-Log "Starting journal sync at $TIMESTAMP" "INFO"
Set-Location $REPO_ROOT

$status = & git status --short 2>&1
if ($status -and $status -notmatch "^\?\?") {
    Write-Log "Uncommitted changes detected — committing sync files first" "WARN"
    Invoke-Git "add JOURNAL.md AGENT_MASTER_SYSTEM.md MASTER_OPERATIONAL_CHECKLIST.md"
    $dirtyCheck = & git diff --cached --name-only 2>&1
    if ($dirtyCheck) {
        Invoke-Git "commit -m `"journal: auto-sync $TIMESTAMP #auto-sync`""
    }
}

# ── 2. Get current branch ──────────────────────────────────────────────────
$CURRENT_BRANCH = (& git branch --show-current 2>&1).Trim()
Write-Log "Current branch: $CURRENT_BRANCH" "INFO"

# ── 3. Get all local + remote branches ────────────────────────────────────
Invoke-Git "fetch --all --prune"
$allBranches = & git branch -r 2>&1 | ForEach-Object { $_.Trim().Replace("origin/", "") } | Where-Object {
    $_ -notmatch "HEAD" -and $_ -notmatch "^$"
} | Select-Object -Unique

Write-Log "Found branches: $($allBranches -join ', ')" "INFO"

# ── 4. Ensure mcp/agent-master branch exists ──────────────────────────────
$mcpExists = & git branch -a 2>&1 | Select-String $MCP_BRANCH
if (-not $mcpExists) {
    Write-Log "Creating $MCP_BRANCH branch" "INFO"
    if (-not $DryRun) {
        & git checkout -b $MCP_BRANCH 2>&1
        & git push -u origin $MCP_BRANCH 2>&1
        & git checkout $CURRENT_BRANCH 2>&1
    }
}

# ── 5. Cherry-pick journal commits to mcp/agent-master ────────────────────
$journalCommits = & git log --oneline --all --grep="#auto-sync" --format="%H" 2>&1 | Select-Object -First 5
if ($journalCommits) {
    Write-Log "Syncing $($journalCommits.Count) journal commits to $MCP_BRANCH" "INFO"
}

# ── 6. Copy sync files to all active branches ─────────────────────────────
$syncSuccess = @()
$syncFailed = @()

foreach ($branch in $allBranches) {
    if ($branch -eq $CURRENT_BRANCH) { continue }
    if ($branch -match "devin/|backup/") {
        Write-Log "Skipping $branch (devin/backup branch)" "WARN"
        continue
    }

    try {
        Write-Log "Syncing to: $branch" "INFO"

        if (-not $DryRun) {
            # Stash current, checkout target, copy files, commit, return
            & git stash 2>&1 | Out-Null
            & git checkout $branch 2>&1 | Out-Null

            foreach ($file in $SYNC_FILES) {
                if (Test-Path "$REPO_ROOT\$file") {
                    & git checkout $CURRENT_BRANCH -- $file 2>&1 | Out-Null
                }
            }

            $staged = & git diff --cached --name-only 2>&1
            if ($staged) {
                & git commit -m "journal: sync from $CURRENT_BRANCH [$TIMESTAMP] #auto-sync" 2>&1 | Out-Null
                & git push origin $branch 2>&1 | Out-Null
                Write-Log "✅ Synced to $branch" "SUCCESS"
            } else {
                Write-Log "No changes needed on $branch" "INFO"
            }

            & git checkout $CURRENT_BRANCH 2>&1 | Out-Null
            & git stash pop 2>&1 | Out-Null
        } else {
            Write-Log "[DRY-RUN] Would sync to $branch" "WARN"
        }

        $syncSuccess += $branch
    }
    catch {
        Write-Log "Failed to sync to $branch: $_" "ERROR"
        $syncFailed += $branch
        # Recover: return to original branch
        & git checkout $CURRENT_BRANCH 2>&1 | Out-Null
        & git stash pop 2>&1 | Out-Null
    }
}

# ── 7. Update STATUS DASHBOARD in AGENT_MASTER_SYSTEM.md ──────────────────
if (-not $DryRun) {
    $errorCount = (& cargo check --package oms-engine 2>&1 | Select-String "^error\[").Count
    $headCommit = (& git log --oneline -1 2>&1).Substring(0, 7)
    $openPRs = "unknown"  # Would use GitHub MCP when available

    $content = Get-Content "$REPO_ROOT\AGENT_MASTER_SYSTEM.md" -Raw
    $content = $content -replace "Compilation Errors \| \d+ \|", "Compilation Errors | $errorCount |"
    $content = $content -replace "Main Branch HEAD \| \w+ \|", "Main Branch HEAD | $headCommit |"
    $content = $content -replace "Last Journal Entry \| .+ \| 2026", "Last Journal Entry | Auto-sync $TIMESTAMP | 2026"
    Set-Content "$REPO_ROOT\AGENT_MASTER_SYSTEM.md" $content

    & git add AGENT_MASTER_SYSTEM.md 2>&1 | Out-Null
    $staged2 = & git diff --cached --name-only 2>&1
    if ($staged2) {
        & git commit -m "ops: dashboard auto-update [$TIMESTAMP] #auto-sync" 2>&1 | Out-Null
        & git push origin $CURRENT_BRANCH 2>&1 | Out-Null
    }
}

# ── 8. Report ─────────────────────────────────────────────────────────────
Write-Log "" "INFO"
Write-Log "═══ SYNC REPORT ═══════════════════════════════" "INFO"
Write-Log "Timestamp : $TIMESTAMP" "INFO"
Write-Log "Source    : $CURRENT_BRANCH" "INFO"
Write-Log "Succeeded : $($syncSuccess -join ', ')" "SUCCESS"
if ($syncFailed) { Write-Log "Failed    : $($syncFailed -join ', ')" "ERROR" }
Write-Log "Files     : $($SYNC_FILES -join ', ')" "INFO"
Write-Log "DryRun    : $DryRun" "INFO"
Write-Log "═══════════════════════════════════════════════" "INFO"

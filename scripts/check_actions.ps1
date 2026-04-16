# PowerShell script to validate GitHub Actions status
# Usage: $env:GITHUB_TOKEN='token'; .\scripts\check_actions.ps1

$ErrorActionPreference = "Stop"

$Token = $env:GITHUB_TOKEN
if (-not $Token) {
    Write-Error "GITHUB_TOKEN environment variable not set"
    exit 1
}

$Owner = "stackconsult"
$Repo = "traderx"
$Branch = "fix/oms-engine-compilation-errors"

Write-Host "Checking Actions for $Branch..." -ForegroundColor Cyan

$Headers = @{
    "Authorization" = "token $Token"
    "Accept" = "application/vnd.github.v3+json"
}

# Build URL with proper escaping
$BaseUrl = "https://api.github.com/repos/$Owner/$Repo/actions/runs"
$Url = "$BaseUrl`?branch=$Branch&per_page=1"

try {
    $Response = Invoke-RestMethod -Uri $Url -Headers $Headers
    
    if ($Response.workflow_runs.Count -eq 0) {
        Write-Host "No Actions runs found" -ForegroundColor Yellow
        exit 1
    }
    
    $Run = $Response.workflow_runs[0]
    
    Write-Host ""
    Write-Host "Latest Run:" -ForegroundColor Cyan
    Write-Host "  ID: $($Run.id)"
    Write-Host "  Status: $($Run.status)" -NoNewline
    
    if ($Run.status -eq "completed") {
        if ($Run.conclusion -eq "success") {
            Write-Host " ✅" -ForegroundColor Green
        } else {
            Write-Host " ❌ ($($Run.conclusion))" -ForegroundColor Red
        }
    } else {
        Write-Host " ⏳" -ForegroundColor Yellow
    }
    
    Write-Host "  URL: $($Run.html_url)"
    Write-Host ""
    
    if ($Run.conclusion -eq "success") {
        Write-Host "✅ VALIDATION PASSED - Ready to merge" -ForegroundColor Green
        exit 0
    } elseif ($Run.conclusion -eq "failure") {
        Write-Host "❌ VALIDATION FAILED - Fixes required" -ForegroundColor Red
        exit 1
    } else {
        Write-Host "⏳ VALIDATION PENDING - Check again later" -ForegroundColor Yellow
        exit 2
    }
} catch {
    Write-Error "Failed to query GitHub API: $_"
    exit 1
}

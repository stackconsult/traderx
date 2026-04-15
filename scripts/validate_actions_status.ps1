# PowerShell script to validate GitHub Actions status
# Part of GitHub-First Self-Healing Architecture

param(
    [string]$Branch = "fix/oms-engine-compilation-errors",
    [string]$Owner = "stackconsult",
    [string]$Repo = "traderx",
    [int]$TimeoutMinutes = 15
)

# Get token from environment or parameter
$Token = $env:GITHUB_TOKEN
if (-not $Token) {
    Write-Host "❌ ERROR: GITHUB_TOKEN not set" -ForegroundColor Red
    Write-Host "Set it with: `$env:GITHUB_TOKEN='your_token'"
    exit 1
}

Write-Host "=== GitHub Actions Validation ===" -ForegroundColor Cyan
Write-Host "Repository: $Owner/$Repo"
Write-Host "Branch: $Branch"
Write-Host "Time: $(Get-Date)"
Write-Host ""

# Disable SSL verification for this request (if needed)
[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.SecurityProtocolType]::Tls12

# Get latest workflow run
Write-Host "🔍 Checking latest Actions run..." -ForegroundColor Yellow

try {
    $Headers = @{
        Authorization = "token $Token"
        Accept = "application/vnd.github.v3+json"
    }
    
    $Uri = "https://api.github.com/repos/$Owner/$Repo/actions/runs?branch=$Branch&per_page=1"`nInvoke-RestMethod -Uri `$Uri -Headers `$Headers -Method GET
    $Response = Invoke-RestMethod -Uri $Uri -Headers $Headers -Method GET
    
    if ($Response.workflow_runs.Count -eq 0) {
        Write-Host "❌ No Actions runs found for branch $Branch" -ForegroundColor Red
        Write-Host "Push may not have triggered Actions yet."
        exit 1
    }
    
    $Run = $Response.workflow_runs[0]
    $RunId = $Run.id
    $Status = $Run.status
    $Conclusion = $Run.conclusion
    $RunUrl = $Run.html_url
    
    Write-Host "Run ID: $RunId"
    Write-Host "Status: $Status"
    Write-Host "Conclusion: $Conclusion"
    Write-Host "URL: $RunUrl"
    Write-Host ""
    
    # Wait if still running
    if ($Status -eq "in_progress" -or $Status -eq "queued") {
        Write-Host "⏳ Actions still running..." -ForegroundColor Yellow
        Write-Host "Monitoring for up to $TimeoutMinutes minutes..."
        
        $MaxAttempts = $TimeoutMinutes * 2  # Check every 30 seconds
        for ($i = 1; $i -le $MaxAttempts; $i++) {
            Start-Sleep -Seconds 30
            
            $RunData = Invoke-RestMethod -Uri "https://api.github.com/repos/$Owner/$Repo/actions/runs/$RunId" -Headers $Headers
            $Status = $RunData.status
            $Conclusion = $RunData.conclusion
            
            Write-Host "  [$i/$MaxAttempts] Status: $Status | Conclusion: $Conclusion"
            
            if ($Status -eq "completed") {
                break
            }
        }
    }
    
    # Final check
    if ($Status -ne "completed") {
        Write-Host ""
        Write-Host "❌ TIMEOUT: Actions did not complete within $TimeoutMinutes minutes" -ForegroundColor Red
        Write-Host "Check manually: $RunUrl"
        exit 1
    }
    
    Write-Host ""
    Write-Host "=== Validation Result ===" -ForegroundColor Cyan
    
    if ($Conclusion -eq "success") {
        Write-Host "✅ SUCCESS: All Actions checks passed!" -ForegroundColor Green
        Write-Host "Branch: $Branch"
        Write-Host "Run ID: $RunId"
        Write-Host "URL: $RunUrl"
        Write-Host ""
        Write-Host "Ready for merge." -ForegroundColor Green
        
        # Create validation stamp
        $ValidationDir = ".validation"
        if (-not (Test-Path $ValidationDir)) {
            New-Item -ItemType Directory -Path $ValidationDir -Force | Out-Null
        }
        
        $StampContent = @"
Validation Time: $(Get-Date -Format "yyyy-MM-ddTHH:mm:ss")
Branch: $Branch
Run ID: $RunId
Conclusion: $Conclusion
Status: PASSED
Validated By: validate_actions_status.ps1
"@
        $StampContent | Out-File -FilePath "$ValidationDir\$Branch-passed.txt" -Encoding utf8
        
        exit 0
    }
    elseif ($Conclusion -eq "failure") {
        Write-Host "❌ FAILURE: Actions checks failed" -ForegroundColor Red
        Write-Host "Branch: $Branch"
        Write-Host "Run ID: $RunId"
        Write-Host "URL: $RunUrl"
        Write-Host ""
        Write-Host "View logs and apply fixes." -ForegroundColor Red
        exit 1
    }
    else {
        Write-Host "⚠️  UNKNOWN: Actions conclusion is '$Conclusion'" -ForegroundColor Yellow
        Write-Host "Branch: $Branch"
        Write-Host "Run ID: $RunId"
        Write-Host "URL: $RunUrl"
        exit 1
    }
}
catch {
    Write-Host "❌ ERROR: $_" -ForegroundColor Red
    exit 1
}

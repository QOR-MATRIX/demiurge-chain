# =============================================================================
# Demiurge Email Setup Script (Windows PowerShell)
# =============================================================================
# This script helps configure SMTP credentials for email verification
# Run: .\scripts\setup-email.ps1 -ResendApiKey "re_your_key"
# =============================================================================

param(
    [string]$ResendApiKey,
    [string]$FromEmail = "[retired contact]",
    [string]$FromName = "Demiurge-Cloud"
)
# Retired before any side effects; kept for source context only.
throw "Retired deployment helper. Use docs/architecture/HOSTING.md and ADR-130; no deployment was performed."



$ErrorActionPreference = "Stop"

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Demiurge Email Configuration Setup" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Check for API key
if (-not $ResendApiKey) {
    Write-Host "Enter your Resend API Key (starts with re_):" -ForegroundColor Yellow
    $ResendApiKey = Read-Host
}

if (-not $ResendApiKey -or -not $ResendApiKey.StartsWith("re_")) {
    Write-Host "ERROR: Invalid API key. Resend API keys start with 're_'" -ForegroundColor Red
    exit 1
}

# Create secrets content
$secretsContent = @"
# Demiurge Production Secrets
# Generated: $(Get-Date -Format "yyyy-MM-ddTHH:mm:ss")
# SECURITY: Never commit this file to git!

# SMTP Configuration (Resend)
SMTP_HOST=smtp.resend.com
SMTP_PORT=587
SMTP_USERNAME=resend
SMTP_PASSWORD=$ResendApiKey
SMTP_FROM_EMAIL=$FromEmail
SMTP_FROM_NAME=$FromName
BASE_URL=https://qorsync.dev
"@

# Save locally
$secretsPath = "config\production\.secrets"
$secretsContent | Out-File -FilePath $secretsPath -Encoding UTF8 -NoNewline
Write-Host "Created local secrets file: $secretsPath" -ForegroundColor Green

# Instructions for server deployment
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Next Steps - Deploy to Server" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "1. Copy secrets to server:" -ForegroundColor Yellow
Write-Host "   scp config\production\.secrets ubuntu@51.210.209.112:/data/Demiurge-Blockchain/config/production/" -ForegroundColor White
Write-Host ""
Write-Host "2. SSH to server and rebuild QOR Auth:" -ForegroundColor Yellow
Write-Host "   ssh ubuntu@51.210.209.112" -ForegroundColor White
Write-Host ""
Write-Host "3. On the server, run:" -ForegroundColor Yellow
Write-Host @"
   cd /data/Demiurge-Blockchain
   git pull origin main
   cd services/qor-auth
   cargo build --release
   sudo systemctl restart qor-auth
   sudo journalctl -u qor-auth -f
"@ -ForegroundColor White
Write-Host ""
Write-Host "The logs should show: 'Email service configured'" -ForegroundColor Green
Write-Host ""

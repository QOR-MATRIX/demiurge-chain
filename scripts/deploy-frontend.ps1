# Retired before any side effects; kept for source context only.
throw "Retired deployment helper. Use docs/architecture/HOSTING.md and ADR-130; no deployment was performed."

# Deploy Frontend (Hub) to Server
# Pulls latest code and rebuilds the Hub container

$SSH_HOST = "pleroma"  # SSH alias configured in ~/.ssh/config
$PROJECT_PATHS = @("/opt/demiurge-blockchain", "/data/Demiurge-Blockchain")

Write-Host "Deploying Frontend to Server..." -ForegroundColor Cyan
Write-Host "The flame burns eternal. The code serves the will." -ForegroundColor Yellow
Write-Host ""

# Find the project directory on the server
Write-Host "Finding project directory..." -ForegroundColor Yellow
$projectPath = $null
foreach ($path in $PROJECT_PATHS) {
    $checkResult = ssh $SSH_HOST "bash -c 'if [ -d `"$path`" ]; then echo exists; else echo notfound; fi'"
    if ($checkResult -match "exists") {
        $projectPath = $path
        Write-Host "Found project at: $projectPath" -ForegroundColor Green
        break
    }
}

if (-not $projectPath) {
    Write-Host "Project directory not found on server!" -ForegroundColor Red
    Write-Host "Checked paths:" -ForegroundColor Yellow
    foreach ($path in $PROJECT_PATHS) {
        Write-Host "  - $path" -ForegroundColor Gray
    }
    exit 1
}

# Step 1: Pull latest code
Write-Host ""
Write-Host "Pulling latest code from GitHub..." -ForegroundColor Green
$pullCmd = "cd $projectPath; git pull origin main"
$pullResult = ssh $SSH_HOST $pullCmd 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "Git pull failed. Continuing with existing code..." -ForegroundColor Yellow
    Write-Host $pullResult -ForegroundColor Gray
}

# Step 2: Rebuild Hub container
Write-Host ""
Write-Host "Rebuilding Hub container (this may take a few minutes)..." -ForegroundColor Green
$buildCmd = "cd $projectPath; docker compose -f docker/docker-compose.production.yml build hub"
ssh $SSH_HOST $buildCmd
if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}

# Step 3: Restart Hub service
Write-Host ""
Write-Host "Restarting Hub service..." -ForegroundColor Green
$restartCmd = "cd $projectPath; docker compose -f docker/docker-compose.production.yml up -d hub"
ssh $SSH_HOST $restartCmd
if ($LASTEXITCODE -ne 0) {
    Write-Host "Failed to restart Hub!" -ForegroundColor Red
    exit 1
}

# Step 4: Wait for service to be ready
Write-Host ""
Write-Host "Waiting for service to be ready..." -ForegroundColor Yellow
Start-Sleep -Seconds 10

# Step 5: Check service status
Write-Host ""
Write-Host "Checking service status..." -ForegroundColor Green
$statusCmd = "cd $projectPath; docker compose -f docker/docker-compose.production.yml ps hub"
ssh $SSH_HOST $statusCmd

# Step 6: Show recent logs
Write-Host ""
Write-Host "Recent Hub logs:" -ForegroundColor Cyan
ssh $SSH_HOST "docker logs demiurge-hub --tail 20"

Write-Host ""
Write-Host "Deployment complete!" -ForegroundColor Cyan
Write-Host ""
Write-Host "Frontend should be available at:" -ForegroundColor Yellow
Write-Host "  http://51.210.209.112:3000" -ForegroundColor White
Write-Host "  https://qorsync.dev (if Nginx is configured)" -ForegroundColor White
Write-Host ""
Write-Host "To view logs:" -ForegroundColor Gray
Write-Host "  ssh $SSH_HOST 'docker logs demiurge-hub -f'" -ForegroundColor DarkGray

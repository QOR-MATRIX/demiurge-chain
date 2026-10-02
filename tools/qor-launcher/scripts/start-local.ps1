<#
.SYNOPSIS
  Start everything the QOR Launcher talks to, on this computer: QOR ID with its
  Postgres and Redis, and a development chain node.

.DESCRIPTION
  The launcher's settings point at http://127.0.0.1:8080/api/v1 for QOR ID and
  ws://127.0.0.1:9944 for the chain. This starts both there.

  - Postgres and Redis run in Docker as `qor-local-pg` (port 54329, volume
    `qor-local-pgdata`, so accounts survive restarts) and `qor-local-redis`
    (port 56389). 5432 and 6379 are often taken, and Windows reserves some
    ranges near 55432, so neither is used.
  - QOR ID runs from services/qor-auth/target/release, built first if missing.
    Its JWT secrets are generated once into %LOCALAPPDATA%\qor-local\secrets.env,
    outside the repository, and never printed.
  - QOR ID sends no real email, whatever this computer's environment holds. The
    script takes RESEND_API_KEY, EMAIL_FROM, BASE_URL and RESEND_WEBHOOK_SECRET
    away from the process it starts, and points RESEND_API_URL at this machine
    (127.0.0.1:59925, where the end-to-end scripts' stand-in for Resend
    listens), so even a key supplied some other way cannot reach Resend. This is
    the default because the end-to-end scripts register addresses nobody holds,
    and sent for real they bounce and count against the sending domain.
      -EmailStandIn   email is on, and goes to that stand-in: what
                      services/qor-auth/scripts/e2e/email-via-resend.mjs needs.
      -SendRealEmail  the process inherits the environment's email settings and
                      sends real mail through Resend. Do not run the end-to-end
                      scripts against it; they refuse.
    Once QOR ID answers, the script asks it whether its mail leaves this
    machine, and stops it if the answer is yes without -SendRealEmail.
  - The chain runs `demiurge-node --dev --tmp`: a fresh chain on every start.
  - Logs go to %LOCALAPPDATA%\qor-local\logs.

.EXAMPLE
  powershell -File tools/qor-launcher/scripts/start-local.ps1
  powershell -File tools/qor-launcher/scripts/start-local.ps1 -NoChain
  powershell -File tools/qor-launcher/scripts/start-local.ps1 -NoChain -EmailStandIn
  powershell -File tools/qor-launcher/scripts/start-local.ps1 -SendRealEmail
  powershell -File tools/qor-launcher/scripts/start-local.ps1 -Stop
#>
param(
  [switch]$Stop,
  [switch]$NoChain,
  # Email on, to the stand-in for Resend on this machine. Nothing leaves it.
  [switch]$EmailStandIn,
  # QOR ID inherits the environment's email settings and sends real mail.
  [switch]$SendRealEmail
)
if ($EmailStandIn -and $SendRealEmail) {
  throw 'Choose one of -EmailStandIn and -SendRealEmail. Without either, QOR ID sends no email.'
}

# Native tools (docker) write routine notices to stderr, which Windows PowerShell
# 5 turns into errors; this script stops only on its own checks, by throwing.
$ErrorActionPreference = 'Continue'
$repo = (Resolve-Path "$PSScriptRoot\..\..\..").Path
$home_ = Join-Path $env:LOCALAPPDATA 'qor-local'
$logs = Join-Path $home_ 'logs'
New-Item -ItemType Directory -Force -Path $logs | Out-Null

$pidFile = Join-Path $home_ 'pids.txt'

function Stop-Local {
  if (Test-Path $pidFile) {
    foreach ($id in Get-Content $pidFile) {
      try { Stop-Process -Id ([int]$id) -Force -ErrorAction Stop } catch {}
    }
    Remove-Item $pidFile -Force
  }
  docker stop qor-local-pg qor-local-redis 2>$null | Out-Null
  Write-Host 'Stopped QOR ID, the chain node, Postgres and Redis.'
}

if ($Stop) { Stop-Local; return }

# --- Docker ---------------------------------------------------------------
$dockerDesktop = Join-Path $env:LOCALAPPDATA 'Programs\DockerDesktop\Docker Desktop.exe'
docker info 2>$null | Out-Null
if ($LASTEXITCODE -ne 0) {
  if (Test-Path $dockerDesktop) { Start-Process $dockerDesktop }
  Write-Host 'Waiting for Docker…'
  for ($i = 0; $i -lt 90; $i++) {
    Start-Sleep -Seconds 2
    docker info 2>$null | Out-Null
    if ($LASTEXITCODE -eq 0) { break }
  }
  if ($LASTEXITCODE -ne 0) { throw 'Docker did not start. Open Docker Desktop and run this again.' }
}

function Start-Container($name, $create) {
  $state = docker inspect -f '{{.State.Running}}' $name 2>$null
  if ($LASTEXITCODE -ne 0) { & $create }
  elseif ($state -ne 'true') { docker start $name | Out-Null }
}

Start-Container 'qor-local-pg' {
  docker run -d --name qor-local-pg -p 127.0.0.1:54329:5432 `
    -e POSTGRES_USER=qor -e POSTGRES_PASSWORD=qor-local -e POSTGRES_DB=qor_auth `
    -v qor-local-pgdata:/var/lib/postgresql/data postgres:16 | Out-Null
}
Start-Container 'qor-local-redis' {
  docker run -d --name qor-local-redis -p 127.0.0.1:56389:6379 redis:7 | Out-Null
}

Write-Host 'Waiting for Postgres…'
for ($i = 0; $i -lt 60; $i++) {
  docker exec qor-local-pg pg_isready -U qor -d qor_auth 2>$null | Out-Null
  if ($LASTEXITCODE -eq 0) { break }
  Start-Sleep -Seconds 1
}
if ($LASTEXITCODE -ne 0) { throw 'Postgres did not become ready.' }

# --- secrets, made once, kept outside the repository ----------------------
$secrets = Join-Path $home_ 'secrets.env'
if (-not (Test-Path $secrets)) {
  function New-Secret { -join ((1..48) | ForEach-Object { '{0:x2}' -f (Get-Random -Maximum 256) }) }
  @("JWT_ACCESS_SECRET=$(New-Secret)", "JWT_REFRESH_SECRET=$(New-Secret)") | Set-Content $secrets
}
$env_ = @{}
foreach ($line in Get-Content $secrets) {
  $k, $v = $line -split '=', 2
  $env_[$k] = $v
}

# --- QOR ID ---------------------------------------------------------------
$auth = Join-Path $repo 'services\qor-auth'
$authExe = Join-Path $auth 'target\release\qor-auth.exe'
if (-not (Test-Path $authExe)) {
  Write-Host 'Building QOR ID (first time only)…'
  Push-Location $auth
  try { cargo build --release } finally { Pop-Location }
}

$pids = @()
# Start-Process redirects to files at the operating-system level, so the
# server keeps writing its log after this script exits. The environment is
# set on this session for the child to inherit, then cleared.
$authEnv = @{
  RUN_ENV = 'development'
  QOR_AUTH__SERVER__PORT = '8080'
  DATABASE_URL = 'postgres://qor:qor-local@127.0.0.1:54329/qor_auth'
  REDIS_URL = 'redis://127.0.0.1:56389'
  JWT_ACCESS_SECRET = $env_['JWT_ACCESS_SECRET']
  JWT_REFRESH_SECRET = $env_['JWT_REFRESH_SECRET']
}

# Email. This computer's environment may hold a real Resend key, and the child
# inherits whatever this session has. Unless real mail was asked for by name,
# every email variable is taken out of the session while the child starts, and
# RESEND_API_URL is set to this machine. That last one is not redundant: QOR ID
# also reads a .env file, which cannot replace a variable that is already set,
# so a key arriving that way still has nowhere to send but here. Values are
# held only to put this session back as it was, and are never written out.
$standIn = 'http://127.0.0.1:59925'
$emailNames = 'RESEND_API_KEY', 'EMAIL_FROM', 'BASE_URL', 'RESEND_API_URL', 'RESEND_WEBHOOK_SECRET'
$emailWas = @{}
if (-not $SendRealEmail) {
  foreach ($k in $emailNames) {
    if (Test-Path "env:$k") { $emailWas[$k] = (Get-Item "env:$k").Value }
  }
  $authEnv['RESEND_API_URL'] = $standIn
  if ($EmailStandIn) {
    # The values services/qor-auth/scripts/e2e/email-via-resend.mjs expects. The
    # key is a placeholder that only the stand-in ever sees.
    $authEnv['RESEND_API_KEY'] = 're_e2e_key'
    $authEnv['EMAIL_FROM'] = 'Demiurge-Cloud <noreply@example.invalid>'
    $authEnv['BASE_URL'] = 'http://127.0.0.1:8080'
  }
}

$authLog = Join-Path $logs 'qor-auth.log'
try {
  foreach ($k in $emailWas.Keys) { Remove-Item "env:$k" -ErrorAction SilentlyContinue }
  foreach ($k in $authEnv.Keys) { Set-Item "env:$k" $authEnv[$k] }
  $authProc = Start-Process -FilePath $authExe -WorkingDirectory $auth -WindowStyle Hidden -PassThru `
    -RedirectStandardOutput $authLog -RedirectStandardError "$authLog.err"
} finally {
  foreach ($k in $authEnv.Keys) { Remove-Item "env:$k" -ErrorAction SilentlyContinue }
  foreach ($k in $emailWas.Keys) { Set-Item "env:$k" $emailWas[$k] }
}
$pids += $authProc.Id

# --- chain ----------------------------------------------------------------
if (-not $NoChain) {
  $node = Join-Path $repo 'chain\target\release\demiurge-node.exe'
  if (Test-Path $node) {
    $chainProc = Start-Process -FilePath $node -ArgumentList '--dev', '--tmp' `
      -RedirectStandardError (Join-Path $logs 'chain.log') `
      -RedirectStandardOutput (Join-Path $logs 'chain.out.log') -WindowStyle Hidden -PassThru
    $pids += $chainProc.Id
  } else {
    Write-Host "No chain node at $node. Build it in chain/ with cargo build --release."
  }
}
$pids | Set-Content $pidFile

# --- ready? ---------------------------------------------------------------
$health = $null
for ($i = 0; $i -lt 60; $i++) {
  try {
    $health = (Invoke-WebRequest -UseBasicParsing -Uri 'http://127.0.0.1:8080/health' -TimeoutSec 2).Content |
      ConvertFrom-Json
    Write-Host 'QOR ID is up at http://127.0.0.1:8080/api/v1'
    break
  } catch { Start-Sleep -Seconds 1 }
  if ($authProc.HasExited) { throw "QOR ID stopped. See $authLog.err" }
}

# --- does its mail leave this machine? -------------------------------------
# Asked of the running service rather than assumed from what was set above: the
# process that answers on 8080 is the one the end-to-end scripts will reach.
if ($health) {
  $leaves = $health.email_leaves_this_machine
  if ($SendRealEmail) {
    if ($leaves -eq $true) {
      Write-Host 'Email: REAL. QOR ID sends through Resend. The end-to-end scripts refuse to run against it.'
    } else {
      Write-Host 'Email: none. -SendRealEmail was given, but the environment does not configure email.'
    }
  } elseif ($leaves -eq $true) {
    Stop-Local
    throw 'QOR ID was about to send real email, though none was asked for, so everything was stopped. Something other than this script configures its email (a .env file in services/qor-auth, or another QOR ID already on port 8080).'
  } elseif ($null -eq $leaves) {
    Write-Host 'Email: unknown. This build of QOR ID is older than the check and does not say; it was started without email settings. Rebuild it (cargo build --release in services/qor-auth); until then the end-to-end scripts refuse to run against it.'
  } elseif ($EmailStandIn) {
    Write-Host "Email: to the stand-in at $standIn only. Nothing leaves this machine."
  } else {
    Write-Host 'Email: off. Nothing is sent. Use -EmailStandIn for the end-to-end email script.'
  }
}
if (-not $NoChain -and $chainProc) { Write-Host 'Chain node starting at ws://127.0.0.1:9944 (a fresh chain).' }
Write-Host "Logs: $logs. Stop everything with: start-local.ps1 -Stop"

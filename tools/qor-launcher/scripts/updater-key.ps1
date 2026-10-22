<#
.SYNOPSIS
    The update channel's signing key (ADR-111): made once, handed to a release build, shown for an offline backup.

.DESCRIPTION
    The key never enters the repository or a log. It lives in %APPDATA%\Demiurge\updater\ on the PC that builds
    releases:

        qor-launcher.key        the private key (minisign), encrypted with a random password
        qor-launcher.key.pub    the public key, which tauri.conf.json carries (plugins.updater.pubkey)
        password.dpapi          the password, protected by Windows for this user on this PC (DPAPI), so nobody types it

        pwsh tools/qor-launcher/scripts/updater-key.ps1 -Create     # once; refuses if a key is already there
        pwsh tools/qor-launcher/scripts/updater-key.ps1 -Public     # the public key, as tauri.conf.json carries it
        pwsh tools/qor-launcher/scripts/updater-key.ps1 -Password   # the password, for release.mjs to hand to the build
        pwsh tools/qor-launcher/scripts/updater-key.ps1 -Backup     # what to keep offline, and the password, once

.NOTES
    The Tauri CLI takes the password on its command line when it makes the key, so it is seen there for that moment,
    on this PC; builds read it from the environment. A lost key means testers reinstall by hand once; keep the backup.
#>
param([switch] $Create, [switch] $Public, [switch] $Password, [switch] $Backup)

$ErrorActionPreference = 'Stop'
$dir = Join-Path $env:APPDATA 'Demiurge\updater'
$key = Join-Path $dir 'qor-launcher.key'
$pub = "$key.pub"
$sealed = Join-Path $dir 'password.dpapi'
$launcher = Split-Path $PSScriptRoot -Parent

function Read-Password {
    if (-not (Test-Path $sealed)) { throw "No update key on this PC ($dir). Make one with -Create." }
    $secure = Get-Content $sealed | ConvertTo-SecureString
    return [Net.NetworkCredential]::new('', $secure).Password
}

if ($Create) {
    if ((Test-Path $key) -or (Test-Path $sealed)) { throw "An update key is already here ($dir); it is never replaced by this script." }
    New-Item -ItemType Directory -Force $dir | Out-Null
    $bytes = [byte[]]::new(32)
    [Security.Cryptography.RandomNumberGenerator]::Fill($bytes)
    $secret = [Convert]::ToBase64String($bytes)
    Push-Location $launcher
    try {
        npx --no-install tauri signer generate --ci -p $secret -w $key | Out-Null
        if ($LASTEXITCODE -ne 0) { throw "tauri signer generate failed ($LASTEXITCODE)." }
    } finally { Pop-Location }
    ConvertTo-SecureString $secret -AsPlainText -Force | ConvertFrom-SecureString | Set-Content $sealed
    Write-Host "Made the update key in $dir."
    Write-Host "Its public key, for tauri.conf.json's plugins.updater.pubkey:"
    Write-Output (Get-Content $pub -Raw).Trim()
    Write-Host "Back it up now: pwsh $PSCommandPath -Backup"
    exit 0
}

if ($Public) {
    if (-not (Test-Path $pub)) { throw "No update key on this PC ($dir). Make one with -Create." }
    Write-Output (Get-Content $pub -Raw).Trim()
    exit 0
}

if ($Password) {
    Write-Output (Read-Password)
    exit 0
}

if ($Backup) {
    $secret = Read-Password
    Write-Host "Keep these offline, together, somewhere only you can reach (a password manager or an encrypted drive):"
    Write-Host "  1. the file $key"
    Write-Host "  2. this password: $secret"
    Write-Host "password.dpapi opens only on this PC for this Windows user, so the password above is what restores the key elsewhere."
    exit 0
}

Write-Host "Say what to do: -Create, -Public, -Password or -Backup (Get-Help $PSCommandPath)."
exit 1

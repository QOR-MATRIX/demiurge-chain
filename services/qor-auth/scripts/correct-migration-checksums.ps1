<#
.SYNOPSIS
  Correct a QOR ID database whose migration checksums were recorded from CRLF files.

.DESCRIPTION
  sqlx records the SHA-384 of each migration file in `_sqlx_migrations.checksum`. Until
  `.gitattributes` pinned `services/qor-auth/migrations/*.sql` to LF, a Windows clone with
  `core.autocrlf=true` held those files with CRLF line endings, so a database migrated by a
  Windows build holds CRLF checksums. A build from LF files (every build now) refuses it:
  "migration 1 was previously applied but has been modified".

  This rewrites a row's checksum to the LF checksum ONLY where the row now holds the CRLF
  checksum of the same file. A row that already holds the LF checksum is left alone, so running
  it twice changes nothing the second time. A row that holds anything else is left alone and
  reported: that migration really was modified, and this script is not the answer to it.
  No migration is run, and no other table is read or written.

  The database is named by a connection URL in an environment variable (DATABASE_URL unless
  -UrlVariable names another). The URL is never printed, never written to a file and never put on
  a command line: its parts reach psql through the PG* environment variables of the child process.
  The SQL holds only version numbers and hexadecimal checksums computed from the files here.

  It needs `psql` on PATH, or Docker, in which case it runs psql from the postgres:16-alpine image.

  It refuses any host but this machine unless -AllowRemoteHost is given. A deployed database is
  corrected deliberately, by whoever is responsible for it, not in passing.

.EXAMPLE
  # The launcher's local database (tools/qor-launcher/scripts/start-local.ps1), while it runs:
  $env:DATABASE_URL = 'postgres://<user>:<password>@127.0.0.1:54329/qor_auth'
  powershell -File services/qor-auth/scripts/correct-migration-checksums.ps1 -DryRun
  powershell -File services/qor-auth/scripts/correct-migration-checksums.ps1
#>
param(
  # The NAME of the environment variable that holds the connection URL.
  [string]$UrlVariable = 'DATABASE_URL',
  # Report what each row holds, and change nothing.
  [switch]$DryRun,
  # Allow a host other than this machine.
  [switch]$AllowRemoteHost
)

$ErrorActionPreference = 'Stop'

# --- the connection, taken apart without being shown ------------------------
$url = [Environment]::GetEnvironmentVariable($UrlVariable)
if ([string]::IsNullOrWhiteSpace($url)) {
  throw "The environment variable $UrlVariable is not set. Set it to the database's connection URL."
}
try { $uri = [Uri]$url } catch { throw "The value of $UrlVariable is not a URL." }
if ($uri.Scheme -ne 'postgres' -and $uri.Scheme -ne 'postgresql') {
  throw "The value of $UrlVariable is not a postgres:// URL."
}
$dbHost = $uri.Host.Trim('[', ']')
$dbPort = if ($uri.Port -gt 0) { $uri.Port } else { 5432 }
$dbName = [Uri]::UnescapeDataString($uri.AbsolutePath.TrimStart('/'))
$dbUser, $dbPassword = $uri.UserInfo -split ':', 2
$dbUser = [Uri]::UnescapeDataString("$dbUser")
$dbPassword = [Uri]::UnescapeDataString("$dbPassword")
$sslMode = $null
foreach ($pair in $uri.Query.TrimStart('?') -split '&') {
  $k, $v = $pair -split '=', 2
  if ($k -eq 'sslmode') { $sslMode = [Uri]::UnescapeDataString("$v") }
}
if (-not $dbName) { throw "The URL in $UrlVariable names no database." }

$local = @('localhost', '127.0.0.1', '::1') -contains $dbHost
if (-not $local -and -not $AllowRemoteHost) {
  throw "The database in $UrlVariable is not on this machine. Pass -AllowRemoteHost if that is intended."
}

# --- the two checksums of every migration -----------------------------------
function Get-Sha384Hex([byte[]]$bytes) {
  $sha = [System.Security.Cryptography.SHA384]::Create()
  try { -join ($sha.ComputeHash($bytes) | ForEach-Object { $_.ToString('x2') }) } finally { $sha.Dispose() }
}

$migrations = Join-Path $PSScriptRoot '..\migrations'
$rows = @()
foreach ($file in Get-ChildItem -Path $migrations -Filter '*.sql' | Sort-Object Name) {
  if ($file.Name -notmatch '^(\d+)_') { continue }
  $version = [int64]$Matches[1]
  $bytes = [System.IO.File]::ReadAllBytes($file.FullName)
  # As git stores it (LF), and as autocrlf checked it out (every LF as CRLF).
  $lf = New-Object System.Collections.Generic.List[byte]
  $crlf = New-Object System.Collections.Generic.List[byte]
  for ($i = 0; $i -lt $bytes.Length; $i++) {
    $b = $bytes[$i]
    if ($b -eq 13 -and $i + 1 -lt $bytes.Length -and $bytes[$i + 1] -eq 10) { continue }
    $lf.Add($b)
    if ($b -eq 10) { $crlf.Add(13) }
    $crlf.Add($b)
  }
  if ($lf.Count -ne $bytes.Length) {
    Write-Warning "$($file.Name) has CRLF line endings on disk. A build from it would record the CRLF checksum again; restore it first (see .gitattributes)."
  }
  $rows += [pscustomobject]@{
    Version = $version
    Name    = $file.Name
    Lf      = Get-Sha384Hex $lf.ToArray()
    Crlf    = Get-Sha384Hex $crlf.ToArray()
  }
}
if ($rows.Count -eq 0) { throw "No migrations were found in $migrations." }

# --- the statement: version numbers and hexadecimal only ---------------------
$values = ($rows | ForEach-Object {
    "({0}, decode('{1}', 'hex'), decode('{2}', 'hex'))" -f $_.Version, $_.Lf, $_.Crlf
  }) -join ",`n  "
$update = if ($DryRun) { '-- dry run: nothing is changed' } else {
  'UPDATE _sqlx_migrations m SET checksum = e.lf FROM expected e WHERE m.version = e.version AND m.checksum = e.crlf;'
}
$sql = @"
BEGIN;
CREATE TEMP TABLE expected (version BIGINT PRIMARY KEY, lf BYTEA NOT NULL, crlf BYTEA NOT NULL) ON COMMIT DROP;
INSERT INTO expected (version, lf, crlf) VALUES
  $values;
SELECT e.version,
       CASE WHEN m.version IS NULL THEN 'absent'
            WHEN m.checksum = e.lf THEN 'lf'
            WHEN m.checksum = e.crlf THEN 'crlf'
            ELSE 'other' END
FROM expected e LEFT JOIN _sqlx_migrations m ON m.version = e.version
ORDER BY e.version;
$update
COMMIT;
"@

# --- psql, here or from Docker -----------------------------------------------
$psqlArgs = @('-X', '-q', '-A', '-t', '-F', '|', '-v', 'ON_ERROR_STOP=1')
$useDocker = -not (Get-Command psql -ErrorAction SilentlyContinue)
if ($useDocker -and -not (Get-Command docker -ErrorAction SilentlyContinue)) {
  throw 'Neither psql nor docker is on PATH. Install the PostgreSQL client, or start Docker.'
}
$childEnv = @{
  # From inside a container, this machine is host.docker.internal.
  PGHOST     = if ($useDocker -and $local) { 'host.docker.internal' } else { $dbHost }
  PGPORT     = "$dbPort"
  PGDATABASE = $dbName
  PGUSER     = $dbUser
  PGPASSWORD = $dbPassword
}
if ($sslMode) { $childEnv['PGSSLMODE'] = $sslMode }
$before = @{}
foreach ($k in $childEnv.Keys) { $before[$k] = [Environment]::GetEnvironmentVariable($k) }

$ErrorActionPreference = 'Continue'   # psql and docker write notices to stderr
try {
  foreach ($k in $childEnv.Keys) { Set-Item "env:$k" $childEnv[$k] }
  if ($useDocker) {
    # `-e NAME` passes the variable through by name; no value is on the command line.
    $pass = $childEnv.Keys | ForEach-Object { '-e'; $_ }
    $out = $sql | & docker run --rm -i @pass postgres:16-alpine psql @psqlArgs 2>&1
  } else {
    $out = $sql | & psql @psqlArgs 2>&1
  }
  $code = $LASTEXITCODE
} finally {
  foreach ($k in $childEnv.Keys) {
    if ($null -eq $before[$k]) { Remove-Item "env:$k" -ErrorAction SilentlyContinue }
    else { Set-Item "env:$k" $before[$k] }
  }
}
$ErrorActionPreference = 'Stop'
if ($code -ne 0) {
  # psql's own messages name the host, the role and the database, never the password.
  $out | ForEach-Object { Write-Host "$_" }
  throw 'psql failed; nothing was changed (the statement is one transaction).'
}

# --- what was found -----------------------------------------------------------
$state = @{}
foreach ($line in $out) {
  if ("$line" -match '^(\d+)\|(absent|lf|crlf|other)$') { $state[[int64]$Matches[1]] = $Matches[2] }
}
$counts = @{ absent = 0; lf = 0; crlf = 0; other = 0 }
foreach ($row in $rows) {
  $s = $state[$row.Version]
  if (-not $s) { throw "psql did not report on migration $($row.Version); nothing can be said about it." }
  $counts[$s]++
  $said = switch ($s) {
    'lf' { 'holds the LF checksum already; left alone' }
    'crlf' { if ($DryRun) { 'holds the CRLF checksum; would be corrected' } else { 'held the CRLF checksum; corrected to LF' } }
    'absent' { 'has not been applied to this database; nothing to correct' }
    'other' { 'holds a checksum that is neither; LEFT ALONE. This migration was really modified' }
  }
  Write-Host ("{0,4}  {1}: {2}" -f $row.Version, $row.Name, $said)
}
$verb = if ($DryRun) { 'to correct' } else { 'corrected' }
Write-Host ("{0} {1}, {2} already LF, {3} not applied, {4} left alone as modified." -f `
    $counts['crlf'], $verb, $counts['lf'], $counts['absent'], $counts['other'])
if ($counts['other'] -gt 0) { exit 2 }

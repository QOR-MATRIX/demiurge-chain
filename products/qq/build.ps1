<#
.SYNOPSIS
    Build QQ (products/qq) with the owner's Qt, MSVC 2022 and Ninja, and run its tests.

.DESCRIPTION
    Configures once, builds, then runs every Qt Test suite and prints its results. Exits non-zero if the build or any
    test fails. The build directory defaults to %LOCALAPPDATA%\qq-build, off the repository's drive.

        pwsh products/qq/build.ps1                 # build and test
        pwsh products/qq/build.ps1 -Run            # build, then open QQ Studio
        pwsh products/qq/build.ps1 -Qt C:\Qt\6.12.0\msvc2022_64 -Build D:\qq-build

.NOTES
    The tests render on the machine's GPU in real windows, so they need a desktop session (not a service account).
#>
param(
    [string] $Qt = 'C:\Qt\6.12.0\msvc2022_64',
    [string] $Build = (Join-Path $env:LOCALAPPDATA 'qq-build'),
    [string] $Config = 'Release',
    [switch] $Run
)

$ErrorActionPreference = 'Stop'
$source = $PSScriptRoot
$tools = Join-Path (Split-Path (Split-Path $Qt -Parent) -Parent) 'Tools'
$cmake = Join-Path $tools 'CMake_64\bin\cmake.exe'
$ninja = Join-Path $tools 'Ninja\ninja.exe'
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
$vs = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
$vcvars = Join-Path $vs 'VC\Auxiliary\Build\vcvars64.bat'
foreach ($needed in $cmake, $ninja, $vcvars) {
    if (-not (Test-Path $needed)) { throw "Not found: $needed" }
}

$configure = "`"$cmake`" -S `"$source`" -B `"$Build`" -G Ninja -DCMAKE_MAKE_PROGRAM=`"$ninja`" -DCMAKE_PREFIX_PATH=`"$Qt`" -DCMAKE_BUILD_TYPE=$Config"
$compile = "`"$cmake`" --build `"$Build`""
cmd /c "call `"$vcvars`" >nul 2>nul && $configure && $compile"
if ($LASTEXITCODE -ne 0) { throw "The build failed ($LASTEXITCODE)." }

$env:PATH = (Join-Path $Qt 'bin') + ';' + $env:PATH

if ($Run) {
    & (Join-Path $Build 'qq-studio.exe')
    exit $LASTEXITCODE
}

$failed = 0
foreach ($test in Get-ChildItem $Build -Filter 'tst_*.exe') {
    $log = Join-Path $Build ($test.BaseName + '.txt')
    & $test.FullName -o "$log,txt" | Out-Null
    $code = $LASTEXITCODE
    Get-Content $log | Where-Object { $_ -match '^(PASS|FAIL|QWARN|QDEBUG|Totals)|failure location' }
    if ($code -ne 0) { $failed += 1 }
}
if ($failed) { throw "$failed test suite(s) failed." }
Write-Host 'All QQ tests passed.'

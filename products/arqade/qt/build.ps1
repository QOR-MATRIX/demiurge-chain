<#
.SYNOPSIS
    Build ARQADE (products/arqade/qt) with the owner's Qt, MSVC 2022 and Ninja, and run its tests.

.DESCRIPTION
    Configures once, builds, then runs every Qt Test suite and prints its results. Exits non-zero if the build or any
    test fails. The build directory defaults to %LOCALAPPDATA%\arqade-build, off the repository's drive.

        pwsh products/arqade/qt/build.ps1              # build and test
        pwsh products/arqade/qt/build.ps1 -Run         # build, then open ARQADE in a window of its own
        pwsh products/arqade/qt/build.ps1 -Deploy      # build, test, then make the self-contained copy the launcher opens
        pwsh products/arqade/qt/build.ps1 -Shots dir   # and save what the interface test drew, to look at

.NOTES
    The interface test opens a real window and draws on the graphics card, so it needs a desktop session (not a service
    account). The launcher finds the deployed copy at %LOCALAPPDATA%\arqade\arqade.exe, or wherever ARQADE_APP points.
#>
param(
    [string] $Qt = 'C:\Qt\6.12.0\msvc2022_64',
    [string] $Build = (Join-Path $env:LOCALAPPDATA 'arqade-build'),
    [string] $Config = 'Release',
    [switch] $Run,
    [switch] $Deploy,
    [string] $DeployTo = (Join-Path $env:LOCALAPPDATA 'arqade'),
    [string] $Shots
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
    & (Join-Path $Build 'arqade.exe')
    exit $LASTEXITCODE
}

if ($Shots) { $env:ARQADE_SHOT = $Shots }

$failed = 0
foreach ($test in Get-ChildItem $Build -Filter 'tst_*.exe') {
    $log = Join-Path $Build ($test.BaseName + '.txt')
    & $test.FullName -o "$log,txt" | Out-Null
    $code = $LASTEXITCODE
    Get-Content $log | Where-Object { $_ -match '^(PASS|FAIL|SKIP|QWARN|Totals)|failure location' }
    if ($code -ne 0) { $failed += 1 }
}
if ($failed) { throw "$failed test suite(s) failed." }
Write-Host 'All ARQADE tests passed.'

# An ARQADE that runs on its own: the executable with the Qt libraries, plugins (the TLS backend its server needs among
# them) and QML modules (Qt Quick 3D, its helpers and particles) it uses beside it. The launcher opens it from here.
if ($Deploy) {
    if (Test-Path $DeployTo) { Remove-Item $DeployTo -Recurse -Force }
    New-Item -ItemType Directory -Force $DeployTo | Out-Null
    Copy-Item (Join-Path $Build 'arqade.exe') $DeployTo
    $windeployqt = Join-Path $Qt 'bin\windeployqt.exe'
    # Inside the compiler's environment, so the Microsoft C++ runtime comes along for a machine without Visual Studio.
    $deployArgs = "--release --no-translations --compiler-runtime --qmldir ""$(Join-Path $source 'qml')"" ""$(Join-Path $DeployTo 'arqade.exe')"""
    cmd /c "call `"$vcvars`" >nul 2>nul && `"$windeployqt`" $deployArgs" | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "windeployqt failed ($LASTEXITCODE)." }
    # The fonts' licence travels with them.
    Copy-Item (Join-Path $source 'fonts\*-OFL.txt') $DeployTo
    Write-Host "ARQADE deployed to $DeployTo"
}

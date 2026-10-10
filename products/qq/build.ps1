<#
.SYNOPSIS
    Build QQ (products/qq) with the owner's Qt, MSVC 2022 and Ninja, and run its tests.

.DESCRIPTION
    Configures once, builds, then runs every Qt Test suite and prints its results. Exits non-zero if the build or any
    test fails. The build directory defaults to %LOCALAPPDATA%\qq-build, off the repository's drive.

        pwsh products/qq/build.ps1                 # build and test
        pwsh products/qq/build.ps1 -Run            # build, then open QQ Studio
        pwsh products/qq/build.ps1 -Deploy         # build, test, then make a self-contained QQ Studio the launcher opens
        pwsh products/qq/build.ps1 -Web            # build the QQ Player for the browser, then measure it in one
        pwsh products/qq/build.ps1 -Qt C:\Qt\6.12.0\msvc2022_64 -Build D:\qq-build

.NOTES
    The tests render on the machine's GPU in real windows, so they need a desktop session (not a service account).
    -Web needs the Qt for WebAssembly kit (multithreaded) beside -Qt, the Emscripten SDK that kit names (5.0.5 for Qt
    6.12) in -Emsdk, Node.js and a Chromium-family browser.
#>
param(
    [string] $Qt = 'C:\Qt\6.12.0\msvc2022_64',
    [string] $Build = (Join-Path $env:LOCALAPPDATA 'qq-build'),
    [string] $Config = 'Release',
    [switch] $Run,
    [switch] $Deploy,
    [string] $DeployTo = (Join-Path $env:LOCALAPPDATA 'qq-studio'),
    [switch] $Web,
    [string] $WebBuild = (Join-Path $env:LOCALAPPDATA 'qq-wasm'),
    [string] $Emsdk = (Join-Path $env:LOCALAPPDATA 'emsdk')
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

# The QQ Player for the browser: Qt for WebAssembly's multithreaded kit (the one with Qt Multimedia, which spatial audio
# needs), with the Emscripten SDK that kit names; then played and measured in a headless browser (player/measure-web.mjs).
if ($Web) {
    $wasmCmake = Join-Path (Split-Path $Qt -Parent) 'wasm_multithread\bin\qt-cmake.bat'
    $emsdkEnv = Join-Path $Emsdk 'emsdk_env.bat'
    foreach ($needed in $wasmCmake, $emsdkEnv) {
        if (-not (Test-Path $needed)) { throw "Not found: $needed" }
    }
    $configureWeb = "`"$wasmCmake`" -S `"$source`" -B `"$WebBuild`" -G Ninja -DCMAKE_MAKE_PROGRAM=`"$ninja`" -DCMAKE_BUILD_TYPE=Release -DQT_HOST_PATH=`"$Qt`""
    cmd /c "call `"$emsdkEnv`" >nul 2>nul && call $configureWeb && `"$cmake`" --build `"$WebBuild`""
    if ($LASTEXITCODE -ne 0) { throw "The web build failed ($LASTEXITCODE)." }
    node (Join-Path $source 'player\measure-web.mjs') (Join-Path $WebBuild 'player') --frame (Join-Path $WebBuild 'first-frame.png')
    if ($LASTEXITCODE -ne 0) { throw 'The QQ Player did not play in the browser.' }
    exit 0
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

# A QQ Studio that runs on its own: the executable with the Qt libraries, plugins and QML modules it uses beside it.
# The QOR Launcher's "Open in QQ Studio" looks here (or wherever QQ_STUDIO points). qq-mcp, which an MCP client starts
# to reach the Studio's tools, goes beside it and uses the same Qt libraries.
if ($Deploy) {
    if (Test-Path $DeployTo) { Remove-Item $DeployTo -Recurse -Force }
    New-Item -ItemType Directory -Force $DeployTo | Out-Null
    Copy-Item (Join-Path $Build 'qq-studio.exe') $DeployTo
    Copy-Item (Join-Path $Build 'qq-mcp.exe') $DeployTo
    $windeployqt = Join-Path $Qt 'bin\windeployqt.exe'
    # Inside the compiler's environment, so the Microsoft C++ runtime comes along for a machine without Visual Studio.
    $deployArgs = "--release --no-translations --compiler-runtime --qmldir ""$(Join-Path $source 'runtime')"" --qmldir ""$(Join-Path $source 'studio')"" ""$(Join-Path $DeployTo 'qq-studio.exe')"""
    cmd /c "call `"$vcvars`" >nul 2>nul && `"$windeployqt`" $deployArgs" | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "windeployqt failed ($LASTEXITCODE)." }
    Write-Host "QQ Studio deployed to $DeployTo"
}

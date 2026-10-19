<#
.SYNOPSIS
    Build the QOR Launcher's Qt shell (tools/qor-launcher/shell) over the Rust core: the desktop build and its tests,
    or the Android APK.

.DESCRIPTION
        pwsh tools/qor-launcher/shell/build.ps1              # desktop (Windows) build, then its Qt Test suites
        pwsh tools/qor-launcher/shell/build.ps1 -Run         # desktop build, then open it
        pwsh tools/qor-launcher/shell/build.ps1 -Android     # the phone's APK, arm64-v8a, debug-signed for testing
        pwsh tools/qor-launcher/shell/build.ps1 -Android -Abi x86_64   # for the emulator on this PC

    It needs the owner's Qt 6.12 (with Qt for Android for -Android), MSVC 2022, Ninja, Rust, and, for -Android, what
    android-toolchain.ps1 installs. Even the Android build runs inside MSVC's environment: Rust's build scripts and
    macros are built for this PC before the core is built for the phone. Build folders default to
    %LOCALAPPDATA%\qor-shell-build and %LOCALAPPDATA%\qor-shell-android, off the repository's drive.
#>
param(
    [string] $Qt = 'C:\Qt\6.12.0',
    [string] $Build,
    [string] $Config = 'Release',
    [switch] $Android,
    [ValidateSet('arm64-v8a', 'x86_64')] [string] $Abi = 'arm64-v8a',
    [switch] $Run,
    [string] $Sdk = $(if ($env:ANDROID_SDK_ROOT) { $env:ANDROID_SDK_ROOT } else { Join-Path $env:LOCALAPPDATA 'Android\Sdk' }),
    [string] $Ndk = '27.2.12479018',
    [string] $Jdk = $(if ($env:JAVA_HOME) { $env:JAVA_HOME } else { Join-Path $env:LOCALAPPDATA 'Programs\jdk-21' })
)

$ErrorActionPreference = 'Stop'
$source = $PSScriptRoot
if (-not $Build) { $Build = Join-Path $env:LOCALAPPDATA $(if ($Android) { "qor-shell-android-$Abi" } else { 'qor-shell-build' }) }
$host64 = Join-Path $Qt 'msvc2022_64'
$tools = Join-Path (Split-Path $Qt -Parent) 'Tools'
$cmake = Join-Path $tools 'CMake_64\bin\cmake.exe'
$ninja = Join-Path $tools 'Ninja\ninja.exe'
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
$vs = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
$vcvars = Join-Path $vs 'VC\Auxiliary\Build\vcvars64.bat'
foreach ($needed in $cmake, $ninja, $vcvars) {
    if (-not (Test-Path $needed)) { throw "Not found: $needed" }
}

if ($Android) {
    $ndkRoot = Join-Path $Sdk "ndk\$Ndk"
    $qtAndroid = Join-Path $Qt ('android_' + $Abi.Replace('-', '_'))
    foreach ($needed in $ndkRoot, $qtAndroid, (Join-Path $Jdk 'bin\java.exe')) {
        if (-not (Test-Path $needed)) { throw "Not found: $needed (run android-toolchain.ps1)" }
    }
    $env:JAVA_HOME = $Jdk
    $env:ANDROID_SDK_ROOT = $Sdk
    $env:ANDROID_NDK_ROOT = $ndkRoot
    $qtCmake = Join-Path $qtAndroid 'bin\qt-cmake.bat'
    $configure = "`"$qtCmake`" -S `"$source`" -B `"$Build`" -G Ninja -DCMAKE_MAKE_PROGRAM=`"$ninja`" -DCMAKE_BUILD_TYPE=$Config " +
        "-DQT_HOST_PATH=`"$host64`" -DANDROID_SDK_ROOT=`"$Sdk`" -DANDROID_NDK_ROOT=`"$ndkRoot`" -DQT_ANDROID_ABIS=$Abi"
    $compile = "`"$cmake`" --build `"$Build`" --target apk"
} else {
    $configure = "`"$cmake`" -S `"$source`" -B `"$Build`" -G Ninja -DCMAKE_MAKE_PROGRAM=`"$ninja`" -DCMAKE_PREFIX_PATH=`"$host64`" -DCMAKE_BUILD_TYPE=$Config"
    $compile = "`"$cmake`" --build `"$Build`""
}
cmd /c "call `"$vcvars`" >nul 2>nul && $configure && $compile"
if ($LASTEXITCODE -ne 0) { throw "The build failed ($LASTEXITCODE)." }

if ($Android) {
    $apk = Join-Path $Build 'android-build\qor-launcher.apk'
    if (-not (Test-Path $apk)) { throw 'The build made no APK.' }
    # Qt's release APK is unsigned, and Android installs nothing unsigned. For testing it is signed with Android's own
    # debug key, the one every Android developer's tools make and use; a release for testers is signed with the
    # project's test key instead (DIRECTION L13.8).
    $buildTools = Join-Path $Sdk 'build-tools\36.0.0'
    $debugKey = Join-Path $env:USERPROFILE '.android\debug.keystore'
    if (-not (Test-Path $debugKey)) {
        New-Item -ItemType Directory -Force (Split-Path $debugKey -Parent) | Out-Null
        & (Join-Path $Jdk 'bin\keytool.exe') -genkeypair -keystore $debugKey -storepass android -alias androiddebugkey `
            -keypass android -keyalg RSA -keysize 2048 -validity 10000 -dname 'CN=Android Debug,O=Android,C=US' | Out-Null
    }
    $signed = Join-Path $Build "qor-launcher-$Abi-debug.apk"
    & (Join-Path $buildTools 'apksigner.bat') sign --ks $debugKey --ks-pass pass:android --key-pass pass:android --out $signed $apk
    if ($LASTEXITCODE -ne 0) { throw 'apksigner could not sign the APK.' }
    & (Join-Path $buildTools 'apksigner.bat') verify $signed
    if ($LASTEXITCODE -ne 0) { throw 'The signed APK does not verify.' }
    $size = (Get-Item $signed).Length
    Write-Host "APK, debug-signed for testing: $signed ($([math]::Round($size / 1MB, 1)) MiB)"
    exit 0
}

$env:PATH = (Join-Path $host64 'bin') + ';' + $env:PATH
if ($Run) {
    & (Join-Path $Build 'qor-launcher.exe')
    exit $LASTEXITCODE
}
$failed = 0
foreach ($test in Get-ChildItem $Build -Filter 'tst_*.exe') {
    $log = Join-Path $Build ($test.BaseName + '.txt')
    & $test.FullName -o "$log,txt" | Out-Null
    $code = $LASTEXITCODE
    Get-Content $log | Where-Object { $_ -match '^(PASS|FAIL|SKIP|QWARN|Totals)|failure location' }
    if ($code -ne 0) { $failed += 1 }
}
if ($failed) { throw "$failed test suite(s) failed." }
Write-Host 'All shell tests passed.'

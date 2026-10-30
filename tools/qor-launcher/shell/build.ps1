<#
.SYNOPSIS
    Build the QOR Launcher's Qt shell (tools/qor-launcher/shell) over the Rust core: the desktop build and its tests,
    or the Android APK.

.DESCRIPTION
        pwsh tools/qor-launcher/shell/build.ps1              # desktop (Windows) build, then its Qt Test suites
        pwsh tools/qor-launcher/shell/build.ps1 -Run         # desktop build, then open it
        pwsh tools/qor-launcher/shell/build.ps1 -Android     # the phone's APK, arm64-v8a, debug-signed for testing
        pwsh tools/qor-launcher/shell/build.ps1 -Android -Abi x86_64   # for the emulator on this PC
        pwsh tools/qor-launcher/shell/build.ps1 -Android -TestKey ~/.qor-launcher/android/qor-test.p12   # for testers

    -TestKey signs a release for testers with the project's test key (DIRECTION L13.8), kept off the repository: a
    PKCS12 file whose password sits beside it in `<file>.dpapi`, sealed by Windows DPAPI to the account that made it, so
    only that account on that PC can sign. Nothing prints the password. The APK is qor-mobile-<version>-<abi>.apk.

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
    [string] $TestKey,
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
    $apksigner = Join-Path $buildTools 'apksigner.bat'
    if ($TestKey) {
        if (-not (Test-Path "$TestKey.dpapi")) { throw "No sealed password beside the test key: $TestKey.dpapi" }
        $version = (Select-String -Path (Join-Path $source 'CMakeLists.txt') -Pattern 'project\(\S+ VERSION (\S+)').Matches[0].Groups[1].Value
        $signed = Join-Path $Build "qor-mobile-$version-$Abi.apk"
        $sealed = Get-Content "$TestKey.dpapi" -Raw | ConvertTo-SecureString
        $env:QOR_ANDROID_KS_PASS = [System.Net.NetworkCredential]::new('', $sealed).Password
        try {
            & $apksigner sign --ks $TestKey --ks-key-alias qor-test --ks-pass env:QOR_ANDROID_KS_PASS --out $signed $apk
        } finally {
            Remove-Item Env:QOR_ANDROID_KS_PASS
        }
        $what = 'signed with the test key, for testers'
    } else {
        $signed = Join-Path $Build "qor-launcher-$Abi-debug.apk"
        & $apksigner sign --ks $debugKey --ks-pass pass:android --key-pass pass:android --out $signed $apk
        $what = 'debug-signed for testing'
    }
    if ($LASTEXITCODE -ne 0) { throw 'apksigner could not sign the APK.' }
    & $apksigner verify $signed
    if ($LASTEXITCODE -ne 0) { throw 'The signed APK does not verify.' }
    $size = (Get-Item $signed).Length
    Write-Host "APK, $what`: $signed ($([math]::Round($size / 1MB, 1)) MiB)"
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

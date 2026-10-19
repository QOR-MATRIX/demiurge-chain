<#
.SYNOPSIS
    Install what building the QOR Launcher for Android needs on this PC: JDK 21, Android's command-line tools, SDK
    platform 36, build tools and NDK r27c, and Rust's Android targets.

.DESCRIPTION
    The versions are Qt 6.12's for Android (its android.html: JDK 21, NDK r27c, Android 9 to 16, API 28 to 36). Every
    download is checked against its published SHA-256 before it is unpacked. Nothing is installed system-wide: the JDK goes
    to %LOCALAPPDATA%\Programs\jdk-21 and the SDK to %LOCALAPPDATA%\Android\Sdk, Android Studio's own default, and the
    variables below are set for this user only.

        pwsh tools/qor-launcher/shell/android-toolchain.ps1

    Running it again checks what is there and installs only what is missing. Android's SDK licences are accepted by
    running it (sdkmanager --licenses); read them at https://developer.android.com/studio/terms.
#>
param(
    [string] $Jdk = (Join-Path $env:LOCALAPPDATA 'Programs\jdk-21'),
    [string] $Sdk = (Join-Path $env:LOCALAPPDATA 'Android\Sdk'),
    [string] $Ndk = '27.2.12479018',
    [string] $Platform = 'android-36',
    [string] $BuildTools = '36.0.0'
)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
$work = Join-Path $env:TEMP 'qor-android-toolchain'
New-Item -ItemType Directory -Force $work | Out-Null

function Get-Checked([string] $Url, [string] $Sha256, [string] $Name) {
    $file = Join-Path $work $Name
    if (-not (Test-Path $file) -or (Get-FileHash $file -Algorithm SHA256).Hash -ne $Sha256.ToUpper()) {
        Write-Host "Downloading $Name"
        Invoke-WebRequest -Uri $Url -OutFile $file
    }
    $got = (Get-FileHash $file -Algorithm SHA256).Hash
    if ($got -ne $Sha256.ToUpper()) { throw "$Name has SHA-256 $got, not the published $Sha256." }
    return $file
}

# JDK 21, Eclipse Temurin, from Adoptium's API, checked against the checksum the API publishes for it.
if (-not (Test-Path (Join-Path $Jdk 'bin\java.exe'))) {
    $asset = (Invoke-RestMethod 'https://api.adoptium.net/v3/assets/latest/21/hotspot?os=windows&architecture=x64&image_type=jdk&vendor=eclipse')[0].binary.package
    $zip = Get-Checked $asset.link $asset.checksum $asset.name
    $unpacked = Join-Path $work 'jdk'
    if (Test-Path $unpacked) { Remove-Item $unpacked -Recurse -Force }
    Expand-Archive $zip $unpacked
    $top = Get-ChildItem $unpacked -Directory | Select-Object -First 1
    New-Item -ItemType Directory -Force (Split-Path $Jdk -Parent) | Out-Null
    Move-Item $top.FullName $Jdk
}
& (Join-Path $Jdk 'bin\java.exe') -version

# Android's command-line tools, the build Qt Creator names with its checksum (sdk_definitions.json).
$sdkmanager = Join-Path $Sdk 'cmdline-tools\latest\bin\sdkmanager.bat'
if (-not (Test-Path $sdkmanager)) {
    $zip = Get-Checked 'https://dl.google.com/android/repository/commandlinetools-win-11076708_latest.zip' `
        '4d6931209eebb1bfb7c7e8b240a6a3cb3ab24479ea294f3539429574b1eec862' 'commandlinetools-win-11076708_latest.zip'
    $unpacked = Join-Path $work 'cmdline-tools'
    if (Test-Path $unpacked) { Remove-Item $unpacked -Recurse -Force }
    Expand-Archive $zip $unpacked
    New-Item -ItemType Directory -Force (Join-Path $Sdk 'cmdline-tools') | Out-Null
    Move-Item (Join-Path $unpacked 'cmdline-tools') (Join-Path $Sdk 'cmdline-tools\latest')
}

$env:JAVA_HOME = $Jdk
$env:ANDROID_SDK_ROOT = $Sdk
$packages = @('platform-tools', "platforms;$Platform", "build-tools;$BuildTools", "ndk;$Ndk")
# sdkmanager asks to accept each licence; this answers yes to all of them.
$yes = ('y' + [Environment]::NewLine) * 30
$yes | & $sdkmanager --sdk_root=$Sdk --licenses | Out-Null
& $sdkmanager --sdk_root=$Sdk --install @packages
if ($LASTEXITCODE -ne 0) { throw "sdkmanager could not install $($packages -join ', ')." }

# Rust for the phone (arm64) and for the emulator (x86_64).
rustup target add aarch64-linux-android x86_64-linux-android
if ($LASTEXITCODE -ne 0) { throw 'rustup could not add the Android targets.' }

# For this user, so Qt's CMake, Gradle and cargo find them in a new terminal.
[Environment]::SetEnvironmentVariable('JAVA_HOME', $Jdk, 'User')
[Environment]::SetEnvironmentVariable('ANDROID_SDK_ROOT', $Sdk, 'User')
[Environment]::SetEnvironmentVariable('ANDROID_NDK_ROOT', (Join-Path $Sdk "ndk\$Ndk"), 'User')

Write-Host ''
Write-Host "JDK          $Jdk"
Write-Host "Android SDK  $Sdk"
Write-Host "Android NDK  $(Join-Path $Sdk "ndk\$Ndk")"
Write-Host 'The Android toolchain is ready.'

param(
    [string]$UnityAndroidPlayerPath,
    [string]$UnityVersion,
    [string]$SdkPath
)

if (-not $UnityAndroidPlayerPath) {
    if (-not $UnityVersion) {
        throw "Provide -UnityAndroidPlayerPath or -UnityVersion (e.g. 6000.2.10f1)."
    }
    $UnityAndroidPlayerPath = "C:\Program Files\Unity\Hub\Editor\$UnityVersion\Editor\Data\PlaybackEngines\AndroidPlayer"
}

$UnityAndroidPlayerPath = (Resolve-Path $UnityAndroidPlayerPath).Path
$env:JAVA_HOME = Join-Path $UnityAndroidPlayerPath "OpenJDK"
if ($SdkPath) {
    $env:ANDROID_SDK_ROOT = (Resolve-Path $SdkPath).Path
} else {
    $env:ANDROID_SDK_ROOT = Join-Path $UnityAndroidPlayerPath "SDK"
}
$env:ANDROID_HOME = $env:ANDROID_SDK_ROOT
$env:ANDROID_NDK_HOME = Join-Path $UnityAndroidPlayerPath "NDK"

$localProps = Join-Path $PSScriptRoot "local.properties"
$sdkDir = $env:ANDROID_SDK_ROOT -replace "\\", "\\\\"
@"
sdk.dir=$sdkDir
"@ | Set-Content -Path $localProps -Encoding ASCII

Write-Host "Using Unity Android tools from: $UnityAndroidPlayerPath"
Write-Host "JAVA_HOME=$env:JAVA_HOME"
Write-Host "ANDROID_SDK_ROOT=$env:ANDROID_SDK_ROOT"
Write-Host "ANDROID_NDK_HOME=$env:ANDROID_NDK_HOME"

& (Join-Path $PSScriptRoot "gradlew.bat") :alvr-android-lib:assembleRelease

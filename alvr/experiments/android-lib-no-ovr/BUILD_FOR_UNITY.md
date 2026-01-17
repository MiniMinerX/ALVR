# Building for Unity

## Prerequisites
- Android SDK with NDK 23.2.8568313
- Rust with Android targets:
  ```bash
  rustup target add aarch64-linux-android
  rustup target add armv7-linux-androideabi
  ```
- Java 8-16 (Gradle 7.0.2 doesn't support Java 17+)

## Build
```powershell
cd alvr\experiments\android-lib-no-ovr
$env:JAVA_HOME="C:\Program Files\Microsoft\jdk-16.0.2.7-hotspot"  # or your JDK 16 path
$env:ANDROID_SDK_ROOT="C:\Users\YourUsername\AppData\Local\Android\Sdk"  # adjust path
$env:ANDROID_HOME=$env:ANDROID_SDK_ROOT  # optional, but recommended
.\gradlew.bat :alvr-android-lib:assembleRelease
```

Output: `alvr-android-lib/build/outputs/aar/alvr-android-lib-release.aar`

## Unity Integration
1. Drag the AAR file to `Assets/Plugins/Android/`
2. Unity automatically extracts native libraries from the AAR

**Important:** Use `[DllImport("alvr_android")]` (underscore, not hyphen)

## Troubleshooting
- **Java version error:** Install Java 8-16, set `JAVA_HOME`:
  ```powershell
  $env:JAVA_HOME="C:\Program Files\Microsoft\jdk-16.0.2.7-hotspot"  # adjust path
  ```
- **SDK location not found:** Set environment variables:
  ```powershell
  $env:ANDROID_SDK_ROOT="C:\Users\YourUsername\AppData\Local\Android\Sdk"
  $env:ANDROID_HOME=$env:ANDROID_SDK_ROOT
  ```
  To set permanently: System Properties → Environment Variables → New
- **NDK not found:** 
  - NDK should be installed at: `%ANDROID_SDK_ROOT%\ndk\23.2.8568313`
  - Default location: `C:\Users\YourUsername\AppData\Local\Android\Sdk\ndk\23.2.8568313`
  - Install via Android Studio: Tools → SDK Manager → SDK Tools → NDK (Side by side) → Check version 23.2.8568313
  - Or set `ANDROID_NDK_HOME` environment variable to the NDK directory
- **Rust fails:** Run both commands separately (see Prerequisites)

# Building for Unity

## Prerequisites
- Use Unity's bundled Android tools (preferred) or a matching Android SDK/NDK
- Rust with Android targets:
  ```bash
  rustup target add aarch64-linux-android
  rustup target add armv7-linux-androideabi
  ```
- Java 17 (Unity 6000 uses JDK 17)

## Build
```powershell
cd alvr\experiments\android-lib-no-ovr
.\build_with_unity.ps1 -UnityVersion 6000.2.10f1
```
If the Unity SDK is read-only (Program Files), pass a writable SDK path:
```powershell
.\build_with_unity.ps1 -UnityVersion 6000.2.10f1 -SdkPath "C:\Users\YourUsername\AppData\Local\Android\Sdk"
```

Output: `alvr-android-lib/build/outputs/aar/alvr-android-lib-release.aar`

## Unity Integration
1. Drag the AAR file to `Assets/Plugins/Android/`
2. Unity automatically extracts native libraries from the AAR

**Important:** Use `[DllImport("alvr_android")]` (underscore, not hyphen)

### Unity Android Manifest / Gradle
- Ensure your Unity `AndroidManifest.xml` has a `package` attribute that matches the `namespace`
  in your `mainTemplate.gradle` to avoid manifest merge errors (Unity/AGP).
- If your Unity Player Settings `minSdkVersion` is lower than the AAR's, the merge will bump it.

### Native Library Packaging
- The AAR must contain `jni/<abi>/libalvr_android.so`. If it doesn't, place the `.so` files in
  `Assets/Plugins/Android/libs/<abi>/` instead.

## Troubleshooting
- **NDK mismatch / linker errors (e.g. __cxa_pure_virtual):**
  - These often mean the native library was built with a different NDK than Unity is using.
  - Prefer Unity's bundled NDK. If you must override, pass a different NDK to Gradle via
    `ANDROID_NDK_HOME` or `-PndkPath`, and ensure `ndkVersion` matches that folder's
    `source.properties`.
- **Unity toolchain (preferred):** Use Unity's bundled tools to avoid mismatches.
  - OpenJDK: `...\AndroidPlayer\OpenJDK`
  - SDK: `...\AndroidPlayer\SDK`
  - NDK: `...\AndroidPlayer\NDK`
  - Gradle: wrapper uses Gradle 8.13 (matches Unity 6000)
- **Java version error:** Unity 6000 uses JDK 17; ensure `JAVA_HOME` points to Unity's OpenJDK.
  ```powershell
  $env:JAVA_HOME="C:\Program Files\Unity\Hub\Editor\6000.2.10f1\Editor\Data\PlaybackEngines\AndroidPlayer\OpenJDK"
  ```
- **SDK location not found:** Set environment variables:
  ```powershell
  $env:ANDROID_SDK_ROOT="C:\Users\YourUsername\AppData\Local\Android\Sdk"
  $env:ANDROID_HOME=$env:ANDROID_SDK_ROOT
  ```
  To set permanently: System Properties → Environment Variables → New
- **NDK not found:** 
  - Use Unity's bundled NDK via `build_with_unity.ps1`
  - Or set `ANDROID_NDK_HOME` environment variable to the NDK directory
- **__cxa_pure_virtual / missing C++ runtime:** Ensure `libc++_shared.so` is packaged with the AAR.
  - The Gradle build copies `libc++_shared.so` into the AAR (from the NDK) via `copyLibcxxShared`.
  - If you are not using the AAR, you must place it in Unity manually (see below).
  - Copy from NDK r23:
    - `%ANDROID_NDK_HOME%\toolchains\llvm\prebuilt\windows-x86_64\sysroot\usr\lib\aarch64-linux-android\libc++_shared.so`
    - `%ANDROID_NDK_HOME%\toolchains\llvm\prebuilt\windows-x86_64\sysroot\usr\lib\arm-linux-androideabi\libc++_shared.so`
  - Place into Unity:
    - `Assets/Plugins/Android/libs/arm64-v8a/libc++_shared.so`
    - `Assets/Plugins/Android/libs/armeabi-v7a/libc++_shared.so` (if shipping 32-bit)

## Test Multiple Unity Toolchains (PowerShell)
Use this to build the AAR against multiple Unity installs. Replace the versions with your 4
installed Unity Editor versions.
```powershell
$UnityEditorVersions = @(
  "6000.2.10f1"
  # "2022.3.22f1"
  # "2023.2.20f1"
  # "2021.3.43f1"
)

$RepoRoot = Resolve-Path "."
$ProjectDir = Join-Path $RepoRoot "alvr\experiments\android-lib-no-ovr"

foreach ($ver in $UnityEditorVersions) {
  Write-Host "== Building with Unity $ver =="
  Push-Location $ProjectDir
  .\build_with_unity.ps1 -UnityVersion $ver
  Pop-Location
}
```
- **Rust fails:** Run both commands separately (see Prerequisites)

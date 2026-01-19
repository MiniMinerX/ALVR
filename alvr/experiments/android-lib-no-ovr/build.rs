fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("android") {
        // Ensure the Android C++ runtime is linked when C++ deps (e.g. oboe) are used.
        // Some toolchains use --as-needed by default, which can drop libc++_shared from DT_NEEDED.
        println!("cargo:rustc-link-arg=-Wl,--no-as-needed");
        // Force an undefined reference so the linker keeps libc++_shared in DT_NEEDED.
        println!("cargo:rustc-link-arg=-Wl,-u,__cxa_pure_virtual");
        println!("cargo:rustc-link-lib=c++_shared");
        println!("cargo:rustc-link-arg=-Wl,--as-needed");
    }
}

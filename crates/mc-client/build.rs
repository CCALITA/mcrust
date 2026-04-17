fn main() {
    // macOS: increase the main thread stack size to 32 MB to prevent
    // stack overflow (SIGBUS) during chunk generation in debug builds.
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "macos" {
        println!("cargo:rustc-link-arg=-Wl,-stack_size,0x2000000");
    }
}

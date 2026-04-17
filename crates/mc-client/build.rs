fn main() {
    // macOS: increase the main thread stack size to 64 MB to prevent
    // stack overflow (SIGBUS) during chunk generation in debug builds.
    // The noise crate + terrain pipeline creates deep call stacks in
    // unoptimized builds.
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "macos" {
        println!("cargo:rustc-link-arg=-Wl,-stack_size,0x4000000");
    }
    // Linux: set via linker flag
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "linux" {
        println!("cargo:rustc-link-arg=-Wl,-z,stacksize=67108864");
    }
}

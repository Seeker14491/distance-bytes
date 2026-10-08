fn main() {
    // Objects nest hundreds of levels deep, and on wasm the library recurses without growing
    // the stack, so the module's stack in linear memory is 16 MB rather than 1 MB.
    if std::env::var("CARGO_CFG_TARGET_FAMILY").as_deref() == Ok("wasm") {
        println!("cargo::rustc-link-arg-cdylib=-zstack-size={}", 16 << 20);
    }
}

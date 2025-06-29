// fn main() {
//     cc::Build::new()
//         .file("runtime.c")
//         .flag_if_supported("-O2")
//         .compile("aethc_runtime");   // производи libaethc_runtime.a
//     println!("cargo:rerun-if-changed=runtime.c");
// }
fn main() {
    // 1. компајлирај runtime.c
    cc::Build::new()
        .file("runtime.c")
        .flag("/O2")
        .flag("/MT")
        .compile("aethc_runtime");

    // 2. кажи Cargo-у где је .lib
    let out = std::env::var("OUT_DIR").unwrap(); // …\build\<hash>\out
    println!("cargo:rustc-env=AETHC_RUNTIME_DIR={}", out);
    println!("cargo:rerun-if-changed=runtime.c");
}

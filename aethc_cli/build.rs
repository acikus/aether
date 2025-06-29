fn main() {
    cc::Build::new()
        .file("runtime.c")
        .flag_if_supported("-O2")
        .compile("aethc_runtime");   // производи libaethc_runtime.a
    println!("cargo:rerun-if-changed=runtime.c");
}

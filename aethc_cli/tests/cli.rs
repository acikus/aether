use std::{process::Command, fs};
use tempfile::tempdir;

fn compile_and_link(src: &str) -> String {
    let dir = tempdir().unwrap();
    let src_path = dir.path().join("main.ae");
    fs::write(&src_path, src).unwrap();
    let exe_path = dir.path().join("app");
    let cli = env!("CARGO_BIN_EXE_aethc_cli");
    let status = Command::new(cli)
        .arg("build")
        .arg(&src_path)
        .arg("-o")
        .arg(&exe_path)
        .status()
        .unwrap();
    assert!(status.success());
    exe_path.to_str().unwrap().to_string()
}

#[test]
fn print_int_and_str() {
    let src = r#"
    fn main() {
        print(42);
        print(\"hi\");
    }
"#;
    let exe = compile_and_link(src);
    let out = Command::new(exe).output().unwrap();
    assert_eq!(String::from_utf8(out.stdout).unwrap(), "42\nhi\n");
}

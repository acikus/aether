use assert_cmd::prelude::*;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn check_ok_file() -> Result<(), Box<dyn std::error::Error>> {
    Command::cargo_bin("aethc")?
        .args(["check", "samples/ok.ae"]) 
        .assert()
        .success();
    Ok(())
}

#[test]
fn build_and_run_hello() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let exe = dir.path().join("hello");
    Command::cargo_bin("aethc")?
        .args(["build", "samples/hello.ae", "-o", exe.to_str().unwrap()])
        .assert()
        .success();
    let out = Command::new(exe).output()?;
    assert_eq!(String::from_utf8(out.stdout)?, "42\nhi\n");
    Ok(())
}

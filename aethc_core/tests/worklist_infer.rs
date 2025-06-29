use aethc_core::{type_inference as infer, infer_ctx::Ty};

fn assert_type(src: &str, expected: Ty) {
    let ty = infer::infer_str(src).expect("inference failed");
    assert_eq!(ty, expected);
}

fn assert_ok(expr: &str) {
    infer::infer_str(expr).expect("expected success");
}

fn assert_err(expr: &str) {
    assert!(infer::infer_str(expr).is_err());
}

#[test]
fn promotion() {
    assert_type("1 + 2.0", Ty::Float);
}

#[test]
fn eq_bool() {
    assert_type("1 == 2", Ty::Bool);
}

#[test]
fn lit_ok() {
    assert_ok("42");
}

#[test]
fn bool_plus_int_err() {
    assert_err("true + 1");
}

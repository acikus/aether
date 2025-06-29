use aethc_core::test_harness::*;

macro_rules! assert_ok {
    ($src:expr) => {
        let res = compile_and_borrow($src);
        assert!(res.errors.is_empty(), "expected ok, got errors: {:#?}", res.errors);
    };
}

macro_rules! assert_err {
    ($src:expr, $code:expr) => {
        let res = compile_and_borrow($src);
        assert!(res.errors.iter().any(|e| e.code == $code), "expected error {}, got {:#?}", $code, res.errors);
    };
}

#[test]
fn reborrrow_ok() {
    assert_ok!(r#"
        fn main() {
            let mut x = 1;
            let y = &mut x;
            *y = 2;
            let z = &mut x;
            *z = 3;
        }
    "#);
}

#[test]
fn reborrrow_err() {
    assert_err!(r#"
        fn main() {
            let mut x = 1;
            let y = &mut x;
            let z = &mut x;
        }
    "#, "E010");
}

#[test]
fn move_ok() {
    assert_ok!(r#"
        fn main() {
            let s = "abc";
            let t = s;
        }
    "#);
}

#[test]
fn move_after_move_err() {
    assert_err!(r#"
        fn main() {
            let s = "abc";
            let t = s;
            let u = s;
        }
    "#, "E011");
}

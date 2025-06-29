use aethc_core::{parser::Parser, resolver::resolve, borrow::check_fn_body};

#[test]
fn ok() {
    let src = r#"fn main(){ let mut x = 1; let x = 2; }"#;
    let (hir_mod, res_errs) = resolve(&Parser::new(src).parse_module());
    assert!(res_errs.is_empty());
    if let aethc_core::hir::Item::Fn(f) = &hir_mod.items[0] {
        let errs = check_fn_body(&f.body);
        assert!(errs.is_empty());
    }
}

#[test]
fn bad() {
    let src = r#"fn main(){ let x = 1; let x = 2; }"#;
    let (hir_mod, res_errs) = resolve(&Parser::new(src).parse_module());
    assert_eq!(res_errs.len(), 1);
    assert!(res_errs[0].msg.contains("cannot redeclare"));
    if let Some(aethc_core::hir::Item::Fn(f)) = hir_mod.items.get(0) {
        let errs = check_fn_body(&f.body);
        assert!(errs.is_empty());
    }
}

#[test]
fn bad_move() {
    let src = r#"fn main(){ let s = "abc"; let t = s; let u = s; }"#;
    let (hir_mod, res_errs) = resolve(&Parser::new(src).parse_module());
    assert!(res_errs.is_empty());
    if let aethc_core::hir::Item::Fn(f) = &hir_mod.items[0] {
        let errs = check_fn_body(&f.body);
        assert_eq!(errs.len(), 1);
        assert!(matches!(errs[0].kind, aethc_core::borrow::BorrowErrorKind::DoubleMove | aethc_core::borrow::BorrowErrorKind::UseAfterMove));
    }
}

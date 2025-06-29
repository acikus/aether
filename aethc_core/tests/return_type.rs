use aethc_core::{parser::Parser, resolver::resolve, hir, type_::Type};

#[test]
fn annotated_return_type_ok() {
    let src = "fn foo() -> Int { return 1; }";
    let (hir_mod, errs) = resolve(&Parser::new(src).parse_module());
    assert!(errs.is_empty());
    if let hir::Item::Fn(f) = &hir_mod.items[0] {
        assert_eq!(f.return_ty, Type::Int);
    }
}

#[test]
fn missing_return_value_error() {
    let src = "fn foo() -> Int { }";
    let (_hir, errs) = resolve(&Parser::new(src).parse_module());
    assert_eq!(errs.len(), 1);
    assert!(errs[0].msg.contains("expected Int"));
}

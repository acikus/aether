use aethc_core::{parser::Parser, resolver::resolve};

#[test]
fn literals_and_let() {
    let src = r#"
        fn main() {
            let x = 42;
            let y = x;
        }
    "#;
    let module = Parser::new(src).parse_module();
    let (_hir, errs) = resolve(&module);
    assert!(errs.is_empty(), "resolve errs: {errs:?}");
}

#[test]
fn unary_not_true() {
    let src = "fn main(){ let b = !true; }";
    let module = Parser::new(src).parse_module();
    let (hir_mod, errs) = resolve(&module);
    assert!(errs.is_empty(), "resolve errs: {errs:?}");

    if let aethc_core::hir::Item::Fn(f) = &hir_mod.items[0] {
        if let aethc_core::hir::Stmt::Let(l) = &f.body.stmts[0] {
            assert_eq!(l.ty, aethc_core::type_::Type::Bool);
        }
    }
}

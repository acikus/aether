use aethc_core::{parser::Parser, resolver::resolve, hir, type_::Type};

#[test]
fn infer_binary_types() {
    let src = r#"fn main(){ let a=1+2; }"#;
    let module = Parser::new(src).parse_module();
    let (hir_mod, errs) = resolve(&module);
    assert!(errs.is_empty());

    if let hir::Item::Fn(f) = &hir_mod.items[0] {
        if let hir::Stmt::Let(l) = &f.body.stmts[0] {
            assert_eq!(l.ty, Type::Int);
        }
    }
}

use aethc_core::{parser::Parser, resolver::resolve, type_::Type};

#[test]
fn int_plus_float_is_float() {
    let src = "fn main(){ let z = 1 + 2.0; }";
    let (hir_mod, errs) = resolve(&Parser::new(src).parse_module());
    assert!(errs.is_empty());

    if let aethc_core::hir::Item::Fn(f) = &hir_mod.items[0] {
        if let aethc_core::hir::Stmt::Let(l) = &f.body.stmts[0] {
            assert_eq!(l.ty, Type::Float);
        }
    }
}

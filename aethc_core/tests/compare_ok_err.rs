use aethc_core::{parser::Parser, resolver::resolve, type_::Type};

#[test]
fn int_less_than_int_is_bool() {
    let src = "fn main(){ let b = 1 < 2; }";
    let (hir_mod, errs) = resolve(&Parser::new(src).parse_module());
    assert!(errs.is_empty());
    if let aethc_core::hir::Item::Fn(f) = &hir_mod.items[0] {
        if let aethc_core::hir::Stmt::Let(l) = &f.body.stmts[0] {
            assert_eq!(l.ty, Type::Bool);
        }
    }
}

#[test]
fn int_eq_float_is_error() {
    let src = "fn main(){ let b = 1 == 2.0; }";
    let (_hir, errs) = resolve(&Parser::new(src).parse_module());
    assert_eq!(errs.len(), 1);
    assert!(errs[0].msg.contains("type mismatch")); 
}

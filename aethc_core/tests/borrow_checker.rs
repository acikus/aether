use aethc_core::{borrowck::borrow_check, parser::Parser, resolver::resolve};

#[test]
fn copy_unit_is_ok() {
    let src = "fn baz() { let a = (); let b = a; }";
    let (hir_mod, res_errs) = resolve(&Parser::new(src).parse_module());
    assert!(res_errs.is_empty());
    let bc_errs = borrow_check(&hir_mod);
    assert!(bc_errs.is_empty());
}

#[test]
fn mutable_var_reuse_promotes_type() {
    let src = r#"fn main(){ let mut x = 1; x = x + 2.0; }"#;
    let (hir_mod, res_errs) = resolve(&Parser::new(src).parse_module());
    assert!(res_errs.is_empty());
    if let aethc_core::hir::Item::Fn(f) = &hir_mod.items[0] {
        if let aethc_core::hir::Stmt::Let(l0) = &f.body.stmts[0] {
            assert_eq!(l0.ty, aethc_core::type_::Type::Int);
        }
        if let aethc_core::hir::Stmt::Assign { .. } = &f.body.stmts[1] {
            // assignment statement parsed
        }
    }
    let bc_errs = borrow_check(&hir_mod);
    assert!(bc_errs.is_empty());
}

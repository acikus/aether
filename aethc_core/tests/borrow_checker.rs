use aethc_core::{borrowck::borrow_check, parser::Parser, resolver::resolve};

#[test]
fn copy_unit_is_ok() {
    let src = "fn baz() { let a = (); let b = a; }";
    let (hir_mod, res_errs) = resolve(&Parser::new(src).parse_module());
    assert!(res_errs.is_empty());
    let bc_errs = borrow_check(&hir_mod);
    assert!(bc_errs.is_empty());
}

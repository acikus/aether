use aethc_core::{parser::Parser, resolver::resolve};

#[test]
fn unit_in_arithmetic_is_error() {
    let src = "fn main(){ let x = 1 + (); }";
    let (_hir, errs) = resolve(&Parser::new(src).parse_module());
    assert_eq!(errs.len(), 1);
    assert!(errs[0].msg.contains("cannot apply"));
}

use aethc_core::{parser::Parser, resolver::resolve};

#[test]
fn missing_return_error() {
    let src = "fn bar() -> Int { }";
    let (_hir, errs) = resolve(&Parser::new(src).parse_module());
    assert_eq!(errs.len(), 1);
    assert!(errs[0].msg.contains("expected Int"));
}

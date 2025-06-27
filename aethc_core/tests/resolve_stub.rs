use aethc_core::{parser::Parser, resolver::resolve};

#[test]
fn resolve_no_errors() {
    let src = "fn main() { 1 + 2; }";
    let module = Parser::new(src).parse_module();
    let (_hir, errs) = resolve(&module);
    assert!(errs.is_empty());
}

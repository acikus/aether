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

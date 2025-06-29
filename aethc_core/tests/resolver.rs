use aethc_core::{parser::Parser, resolver::resolve};

#[test]
fn implicit_return_ok() {
    let src = r#"
        fn foo() {
            let x = 1;
        }
    "#;
    let module = Parser::new(src).parse_module();
    let (_hir, errs) = resolve(&module);
    assert!(errs.is_empty());
}

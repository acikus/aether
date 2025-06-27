use aethc_core::{parser::Parser, resolver::resolve};

#[test]
fn duplicate_name_error() {
    let src = r#"
        fn main() {
            let x = 1;
            let x = 2;  // дупликат
        }
    "#;

    let module = Parser::new(src).parse_module();
    let (_hir, errs) = resolve(&module);
    assert_eq!(errs.len(), 1);
    // сада очекујемо следећу поруку:
    assert!(errs[0].msg.contains("already defined"));
}

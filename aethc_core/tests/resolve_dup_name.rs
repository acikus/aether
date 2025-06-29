use aethc_core::{borrowck::borrow_check, parser::Parser, resolver::resolve};
#[test]
fn duplicate_name_error() {
    let src = r#"
        fn main() {
            let x = 1;
            let x = 2;
        }
    "#;
    let module = Parser::new(&src).parse_module();
    let (hir_mod, res_errs) = resolve(&module);

    // Resolver should report an error for the duplicate name
    assert_eq!(res_errs.len(), 1);
    assert!(res_errs[0].msg.contains("already defined"));

    // Borrow checker sees no errors when resolve already failed
    let bc_errs = borrow_check(&hir_mod);
    assert!(bc_errs.is_empty());
}

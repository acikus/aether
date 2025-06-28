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

    // Resolver should not report errors (allows shadowing)
    assert_eq!(res_errs.len(), 0);

    // Borrow checker should catch the immutable reassignment
    let bc_errs = borrow_check(&hir_mod);
    assert_eq!(bc_errs.len(), 1);
    assert!(bc_errs[0].msg.contains("cannot reassign immutable binding"));
}

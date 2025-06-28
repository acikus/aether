use aethc_core::{borrowck::borrow_check, parser::Parser, resolver::resolve};

#[test]
fn borrowck_detects_reassignment() {
    let src = r#"
        fn main() {
            let x = 1;
            let x = 2; // Error: first x was immutable
            let mut y = 3;
            let y = 4; // OK: first y was mutable
        }
    "#;
    let module = Parser::new(src).parse_module();
    let (hir_mod, res_errs) = resolve(&module);

    // Resolver should not report errors (allows shadowing)
    assert!(res_errs.is_empty(), "resolve errs: {res_errs:#?}");

    // Borrow-checker should report 1 error for x
    let bc_errs = borrow_check(&hir_mod);
    assert_eq!(
        bc_errs.len(),
        1,
        "expected 1 borrow-checker error, got: {bc_errs:#?}"
    );
    assert!(
        bc_errs[0]
            .msg
            .contains("cannot reassign immutable binding `x`")
    );
}

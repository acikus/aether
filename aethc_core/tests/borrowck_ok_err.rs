use aethc_core::{
    parser::Parser,
    resolver::resolve,
    borrowck::borrow_check,
};

#[test]
fn borrowck_detects_reassignment() {
    let src = r#"
        fn main() {
            let x = 1;
            let x = 2; // duplikacija – resolve prijavljuje grešku
            let mut y = 3;
            let y = 4; // OK, prvi je bio mutable
        }
    "#;

    let module = Parser::new(src).parse_module();
    let (hir_mod, res_errs) = resolve(&module);

    // Rezolver mora prijaviti tačno 1 grešku zbog x

    assert_eq!(res_errs.len(), 1, "resolve errs: {res_errs:#?}");

    // Borrow-checker ne prijavljuje dodatne greške
    let bc_errs = borrow_check(&hir_mod);
    assert!(
        bc_errs.is_empty(),
        "expected 0 borrow-checker errors, got: {bc_errs:#?}"
    );
}

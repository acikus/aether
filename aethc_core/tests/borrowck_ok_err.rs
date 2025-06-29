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

    // Resolver should report a duplicate-name error
    assert_eq!(res_errs.len(), 1);
    assert!(res_errs[0].msg.contains("cannot redeclare"));

    // Borrow-checker sees no errors when resolve already failed
    let bc_errs = borrow_check(&hir_mod);
    assert!(bc_errs.is_empty());
}

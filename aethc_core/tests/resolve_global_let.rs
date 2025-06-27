use aethc_core::{parser::Parser, resolver::resolve, type_::Type};

#[test]
fn global_let_ok() {
    let src = r#"
        let two = 2;

        fn main() {
            let x = two + 3;
        }
    "#;

    let module = Parser::new(src).parse_module();
    let (hir_mod, errs) = resolve(&module);
    assert!(errs.is_empty(), "resolve errs: {errs:?}");

    // proveri da je prvi item globalni let i da je tip Int
    if let aethc_core::hir::Item::Let(gl) = &hir_mod.items[0] {
        assert_eq!(gl.ty, Type::Int);
    } else {
        panic!("prvi item nije globalni let");
    }
}

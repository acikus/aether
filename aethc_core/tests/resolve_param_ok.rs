use aethc_core::{parser::Parser, resolver::resolve, type_::Type};

#[test]
fn param_with_type() {
    let src = r#"
        fn add(x: Int, y: Int) {
            let z = x + y;
        }
    "#;
    let module = Parser::new(src).parse_module();
    let (hir_mod, errs) = resolve(&module);
    assert!(errs.is_empty());

    if let aethc_core::hir::Item::Fn(f) = &hir_mod.items[0] {
        assert_eq!(f.params[0].ty, Type::Int);
        assert_eq!(f.params[1].ty, Type::Int);
    }
}

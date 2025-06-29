use aethc_core::{parser::Parser, resolver::resolve, mir::{self, Terminator, Statement, Rvalue}};

#[test]
fn add_fn_lowering() {
    let src = "fn add(a: Int, b: Int) -> Int { return a + b; }";
    println!("SRC: {src}");
    let module = Parser::new(src).parse_module();
    let (hir_mod, errs) = resolve(&module);
    assert!(errs.is_empty());
    if let aethc_core::hir::Item::Fn(f) = &hir_mod.items[0] {
        let mir_body = mir::lower_fn(f);
        assert_eq!(mir_body.blocks.len(), 1);
        assert!(matches!(mir_body.blocks[0].term, Terminator::Return));
        let binops = mir_body.blocks[0]
            .stmts
            .iter()
            .filter(|s| matches!(s, Statement::Assign { rv: Rvalue::BinaryOp { .. }, .. }))
            .count();
        assert_eq!(binops, 1);
    } else {
        panic!("expected function");
    }
}

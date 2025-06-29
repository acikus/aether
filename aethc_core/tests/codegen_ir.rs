use aethc_core::{
    parser::Parser,
    resolver::resolve,
    mir::{self},
    codegen::{LlvmCtx, codegen_fn},
};

fn build_dummy_add() -> mir::MirBody {
    let src = "fn add() -> Int { return 3 + 4; }";
    let module = Parser::new(src).parse_module();
    let (hir_mod, errs) = resolve(&module);
    assert!(errs.is_empty());
    if let aethc_core::hir::Item::Fn(f) = &hir_mod.items[0] {
        mir::lower_fn(f)
    } else {
        panic!("expected function");
    }
}

#[test]
fn simple_add() {
    let mir = build_dummy_add();
    let mut llcx = LlvmCtx::new("test");
    codegen_fn(&mut llcx, "add", &mir);
    let txt = llcx.module.print_to_string().to_string();
    assert!(txt.contains("define i32 @add()"));
    assert!(txt.contains("add i32"));
}

use aethc_core::{parser::Parser, resolver::resolve, hir, type_::Type};

#[test]
fn return_without_value_lowered_to_unit() {
    let src = "fn main(){ return; }";
    let (hir_mod, errs) = resolve(&Parser::new(src).parse_module());
    assert!(errs.is_empty());

    if let hir::Item::Fn(f) = &hir_mod.items[0] {
        if let hir::Stmt::Return(Some(expr)) = &f.body.stmts[0] {
            match expr {
                hir::Expr::Unit { ty, .. } => assert_eq!(ty, &Type::Unit),
                _ => panic!("expected unit expression"),
            }
        } else {
            panic!("expected return statement");
        }
    } else {
        panic!("expected function item");
    }
}

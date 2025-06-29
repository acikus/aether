use aethc_core::{parser::Parser, resolver::resolve, hir, type_::Type};

fn assert_expr_type(expr: &str, expected: Type) {
    let src = format!("fn main() {{ let tmp = {}; }}", expr);
    let (hir_mod, errs) = resolve(&Parser::new(&src).parse_module());
    assert!(errs.is_empty(), "errors: {:?}", errs);
    if let hir::Item::Fn(f) = &hir_mod.items[0] {
        if let hir::Stmt::Let(l) = &f.body.stmts[0] {
            assert_eq!(l.ty, expected);
        } else {
            panic!("expected let");
        }
    } else {
        panic!("expected function");
    }
}

fn assert_ok(src: &str) {
    let module = Parser::new(src).parse_module();
    let (_hir, errs) = resolve(&module);
    assert!(errs.is_empty(), "expected ok, got {:?}", errs);
}

fn assert_err(src: &str) {
    let module = Parser::new(src).parse_module();
    let (_hir, errs) = resolve(&module);
    assert!(!errs.is_empty(), "expected error");
}

#[test]
fn plus_promotes_to_float() {
    assert_expr_type("1 + 2.0", Type::Float);
}

#[test]
fn compare_int_float() {
    assert_expr_type("3 < 4.5", Type::Bool);
}

#[test]
fn int_to_float_return_ok() {
    assert_ok("fn f() -> Float { return 1; }");
}

#[test]
fn float_to_int_return_err() {
    assert_err("fn g() -> Int { return 2.5; }");
}

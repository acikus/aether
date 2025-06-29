use aethc_core::parser::Parser;
use aethc_core::ast;

fn assert_ast(src: &str) -> ast::Expr {
    // Wrap expression into a function body so we can parse it
    let wrapped = format!("fn main() {{ {src}; }}");
    let module = Parser::new(&wrapped).parse_module();
    if let ast::Item::Function(f) = &module.items[0] {
        if let ast::Stmt::Expr(e) = &f.body[0] {
            return e.clone();
        }
    }
    panic!("unexpected AST shape");
}

#[test]
fn parse_hello() {
    let src = r#"fn main() { println("hi"); }"#;
    let module = Parser::new(src).parse_module();
    assert_eq!(module.items.len(), 1);
}

#[test]
fn parse_unit_expr() {
    let expr = assert_ast("()");
    if !matches!(expr, ast::Expr::Unit) {
        panic!("expected unit expr");
    }
}

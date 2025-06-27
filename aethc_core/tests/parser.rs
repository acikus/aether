use aethc_core::parser::Parser;

#[test]
fn parse_hello() {
    let src = r#"fn main() { println("hi"); }"#;
    let module = Parser::new(src).parse_module();
    assert_eq!(module.items.len(), 1);
}

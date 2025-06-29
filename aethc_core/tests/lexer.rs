use aethc_core::lexer::{Lexer, TokenKind, TokenKind::*};

#[test]
fn simple_tokens() {
    let src = r#"
        let mut counter: Int = 42;
        // Comment
        if counter > 10 {
            println("big");
        }
    "#;

    let mut lex = Lexer::new(src);
    let kinds: Vec<TokenKind> = std::iter::from_fn(|| Some(lex.next_token().kind))
        .take_while(|k| *k != Eof)
        .collect();

    assert_eq!(
        kinds,
        vec![
            Let, Mut, Ident("counter".into()), Colon, Ident("Int".into()), Assign, Int(42), Semicolon,
            If, Ident("counter".into()), Gt, Int(10), LBrace,
            Ident("println".into()), LParen, Str("big".into()), RParen, Semicolon,
            RBrace
        ]
    );
}

#[test]
fn bool_literals() {
    let src = "true false";
    let mut lex = Lexer::new(src);
    let kinds: Vec<TokenKind> = std::iter::from_fn(|| Some(lex.next_token().kind))
        .take_while(|k| *k != Eof)
        .collect();

    assert_eq!(kinds, vec![Bool(true), Bool(false)]);
}
 

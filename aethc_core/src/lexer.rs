// lexer.rs – v0.1‑final (floats, escapes, byte‑strings, nested comments)

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: u32,
    pub column: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    // Keywords
    Let, Mut, Fn, Match, If, Else, While, For, In, Return, Spawn, Channel, Use,
    // Ident & literals
    Ident(String),
    Int(i64),
    Float(f64),
    Str(String),
    ByteStr(Vec<u8>),
    // Operators & punctuation
    Plus, Minus, Star, Slash, Percent,
    AndAnd, OrOr,
    EqEq, NotEq,
    Lt, Gt, Le, Ge,
    Bang, Assign,
    Arrow, FatArrow,
    Colon, DoubleColon, Semicolon, Comma, Dot,
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    // End of file
    Eof,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

pub struct Lexer<'a> {
    input: &'a str,
    pos: usize,
    line: u32,
    column: u32,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self { input: src, pos: 0, line: 1, column: 1 }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_ws_and_comments();
        if self.pos >= self.input.len() {
            return self.make_tok(TokenKind::Eof, 0);
        }
        let ch = self.peek();
        if ch.is_ascii_alphabetic() || ch == '_' { return self.ident_or_kw(); }
        if ch.is_ascii_digit() { return self.number(); }
        if ch == '"' || (ch == 'b' && self.peek_ahead(1) == Some('"')) {
            return self.string_like();
        }
        self.operator_or_punct()
    }

    /*──────────────── helpers ────────────────*/

    fn skip_ws_and_comments(&mut self) {
        loop {
            match self.peek() {
                ' ' | '\t' | '\r' => { self.bump(1); },
                '\n' => { self.bump(1); self.line += 1; self.column = 1; },
                '/' if self.peek_ahead(1) == Some('/') => { // line comment
                    while self.peek() != '\n' && self.peek() != '\0' { self.bump(1); }
                },
                '/' if self.peek_ahead(1) == Some('*') => { // block / nested comment
                    self.bump(2); // consume /*
                    let mut depth = 1;
                    while depth > 0 {
                        match (self.peek(), self.peek_ahead(1)) {
                            ('/', Some('*')) => { self.bump(2); depth += 1; },
                            ('*', Some('/')) => { self.bump(2); depth -= 1; },
                            ('\0', _) => break, // unterminated
                            _ => { if self.peek() == '\n' { self.line += 1; self.column = 1; } self.bump(1); },
                        }
                    }
                },
                _ => break,
            }
        }
    }

    fn ident_or_kw(&mut self) -> Token {
        let start = self.pos;
        while self.peek().is_ascii_alphanumeric() || self.peek() == '_' { self.bump(1); }
        let text = &self.input[start..self.pos];
        let kind = match text {
            "let" => TokenKind::Let, "mut" => TokenKind::Mut, "fn" => TokenKind::Fn,
            "match" => TokenKind::Match, "if" => TokenKind::If, "else" => TokenKind::Else,
            "while" => TokenKind::While, "for" => TokenKind::For, "in" => TokenKind::In,
            "return" => TokenKind::Return, "spawn" => TokenKind::Spawn, "channel" => TokenKind::Channel,
            "use" => TokenKind::Use, _ => TokenKind::Ident(text.to_string()),
        };
        self.make_tok(kind, self.pos - start)
    }

    fn number(&mut self) -> Token {
        let start = self.pos;
        while self.peek().is_ascii_digit() { self.bump(1); }
        let mut is_float = false;
        if self.peek() == '.' && self.peek_ahead(1).map_or(false, |c| c.is_ascii_digit()) {
            is_float = true; self.bump(1);
            while self.peek().is_ascii_digit() { self.bump(1); }
        }
        if matches!(self.peek(), 'e' | 'E') {
            is_float = true; self.bump(1);
            if matches!(self.peek(), '+' | '-') { self.bump(1); }
            while self.peek().is_ascii_digit() { self.bump(1); }
        }
        let text = &self.input[start..self.pos].replace('_', "");
        if is_float {
            let v = text.parse::<f64>().unwrap_or(0.0);
            self.make_tok(TokenKind::Float(v), self.pos - start)
        } else {
            let v = text.parse::<i64>().unwrap_or(0);
            self.make_tok(TokenKind::Int(v), self.pos - start)
        }
    }

    fn string_like(&mut self) -> Token {
        let is_bytes = self.peek() == 'b';
        if is_bytes { self.bump(1); }
        self.bump(1); // opening quote
        let start = self.pos;
        let mut value = String::new();
        while self.peek() != '"' && self.peek() != '\0' {
            match self.peek() {
                '\\' => { // escape
                    self.bump(1);
                    let esc = self.peek();
                    match esc {
                        'n' => { value.push('\n'); self.bump(1); },
                        't' => { value.push('\t'); self.bump(1); },
                        'r' => { value.push('\r'); self.bump(1); },
                        '\\' => { value.push('\\'); self.bump(1); },
                        '"' => { value.push('"'); self.bump(1); },
                        'u' if self.peek_ahead(1) == Some('{') => { // \u{XXXX}
                            self.bump(2);
                            let mut hex = String::new();
                            while let c @ '0'..='9' | c @ 'a'..='f' | c @ 'A'..='F' = self.peek() {
                                hex.push(c); self.bump(1);
                            }
                            if self.peek() == '}' { self.bump(1); }
                            if let Ok(code) = u32::from_str_radix(&hex, 16) { if let Some(ch) = std::char::from_u32(code) { value.push(ch); } }
                        },
                        other => { value.push(other); self.bump(1); },
                    }
                },
                ch => { value.push(ch); self.bump(1); },
            }
        }
        self.bump(1); // closing quote
        if is_bytes {
            let bytes = value.into_bytes();
            self.make_tok(TokenKind::ByteStr(bytes), self.pos - start + 2)
        } else {
            self.make_tok(TokenKind::Str(value), self.pos - start + 1)
        }
    }

    fn operator_or_punct(&mut self) -> Token {
        let start = self.pos;
        let ch1 = self.peek(); let ch2 = self.peek_ahead(1);
        let kind = match (ch1, ch2) {
            ('=', Some('=')) => { self.bump(2); TokenKind::EqEq },
            ('!', Some('=')) => { self.bump(2); TokenKind::NotEq },
            ('<', Some('=')) => { self.bump(2); TokenKind::Le },
            ('>', Some('=')) => { self.bump(2); TokenKind::Ge },
            ('&', Some('&')) => { self.bump(2); TokenKind::AndAnd },
            ('|', Some('|')) => { self.bump(2); TokenKind::OrOr },
            ('-', Some('>')) => { self.bump(2); TokenKind::Arrow },
            ('=', Some('>')) => { self.bump(2); TokenKind::FatArrow },
            (':', Some(':')) => { self.bump(2); TokenKind::DoubleColon },
            _ => { let k = match ch1 {
                    '+' => TokenKind::Plus, '-' => TokenKind::Minus, '*' => TokenKind::Star,
                    '/' => TokenKind::Slash, '%' => TokenKind::Percent, '=' => TokenKind::Assign,
                    '!' => TokenKind::Bang, '<' => TokenKind::Lt, '>' => TokenKind::Gt,
                    ':' => TokenKind::Colon, ';' => TokenKind::Semicolon, ',' => TokenKind::Comma, '.' => TokenKind::Dot,
                    '(' => TokenKind::LParen, ')' => TokenKind::RParen, '{' => TokenKind::LBrace,
                    '}' => TokenKind::RBrace, '[' => TokenKind::LBracket, ']' => TokenKind::RBracket,
                    _ => TokenKind::Eof,
                }; self.bump(1); k }
        };
        self.make_tok(kind, self.pos - start)
    }

    /*──────── misc ───────*/
    fn bump(&mut self, n: usize) { for _ in 0..n { if let Some(c) = self.input[self.pos..].chars().next() { self.pos += c.len_utf8(); self.column += 1; } } }
    fn peek(&self) -> char { self.input[self.pos..].chars().next().unwrap_or('\0') }
    fn peek_ahead(&self, n: usize) -> Option<char> { self.input[self.pos..].chars().nth(n) }
    fn make_tok(&self, kind: TokenKind, len: usize) -> Token { Token { kind, span: Span { start: self.pos - len, end: self.pos, line: self.line, column: self.column - len as u32 } } }
}

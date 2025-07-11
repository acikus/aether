//! parser.rs – рекурзивни‑десцент + Pratt (са унарним операцијама)
//! Подешено да подржи -x и !x као префикс операторе.

use crate::ast;
use crate::lexer::{Lexer, Token, TokenKind};

/*──────── Parser ───────*/

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    lookahead: Token,
}

impl<'a> Parser<'a> {
    pub fn new(src: &'a str) -> Self {
        let mut lexer = Lexer::new(src);
        let lookahead = lexer.next_token();
        Self { lexer, lookahead }
    }

    /*──────── module ─────*/
    pub fn parse_module(mut self) -> ast::Module {
        let mut items = Vec::new();
        while self.lookahead.kind != TokenKind::Eof {
            items.push(self.parse_item());
        }
        ast::Module { items }
    }

    /*──────── items ──────*/
    fn parse_item(&mut self) -> ast::Item {
        match self.lookahead.kind {
            TokenKind::Fn => ast::Item::Function(self.parse_function()),
            TokenKind::Let => ast::Item::Let(self.parse_global_let()),
            _ => panic!("unexpected token {:?}", self.lookahead.kind),
        }
    }

    /*──────── function ───*/
    fn parse_function(&mut self) -> ast::Function {
        self.expect(TokenKind::Fn);
        let name = self.expect_ident();
        self.expect(TokenKind::LParen);

        // params
        let mut params = Vec::new();
        if self.lookahead.kind != TokenKind::RParen {
            loop {
                let pname = self.expect_ident();
                let pty = if self.lookahead.kind == TokenKind::Colon {
                    self.expect(TokenKind::Colon);
                    Some(self.expect_ident())
                } else {
                    None
                };
                params.push(ast::Param {
                    name: pname,
                    ty: pty,
                });
                if self.lookahead.kind == TokenKind::Comma {
                    self.bump();
                } else {
                    break;
                }
            }
        }
        self.expect(TokenKind::RParen);

        let return_ty = if self.lookahead.kind == TokenKind::Arrow {
            self.bump();
            Some(self.expect_ident())
        } else {
            None
        };

        let body = self.parse_fn_body();

        ast::Function { name, params, return_ty, body }
    }

    fn parse_fn_body(&mut self) -> Vec<ast::Stmt> {
        self.expect(TokenKind::LBrace);
        let mut body = Vec::new();
        while self.lookahead.kind != TokenKind::RBrace {
            body.push(self.parse_stmt());
        }
        if !matches!(body.last(), Some(ast::Stmt::Return(_))) {
            body.push(ast::Stmt::Return(Some(ast::Expr::Unit)));
        }
        self.expect(TokenKind::RBrace);
        body
    }

    /*──────── statements ─*/
    fn parse_stmt(&mut self) -> ast::Stmt {
        match self.lookahead.kind {
            TokenKind::Let => self.parse_let(),
            TokenKind::Return => self.parse_return(),
            TokenKind::Ident(_) if self.peek_next(TokenKind::Assign) => {
                let name = self.expect_ident();
                self.expect(TokenKind::Assign);
                let expr = self.parse_expr(0);
                self.expect(TokenKind::Semicolon);
                ast::Stmt::Assign { name, expr }
            }
            _ => {
                let expr = self.parse_expr(0);
                self.expect(TokenKind::Semicolon);
                ast::Stmt::Expr(expr)
            }
        }
    }

    /*──────── локални let */
    fn parse_let(&mut self) -> ast::Stmt {
        self.expect(TokenKind::Let);

        let mutable = if self.lookahead.kind == TokenKind::Mut {
            self.bump();
            true
        } else {
            false
        };
        let name = self.expect_ident();
        self.expect(TokenKind::Assign);
        let expr = self.parse_expr(0);
        self.expect(TokenKind::Semicolon);

        ast::Stmt::Let {
            name,
            expr,
            mutable,
        }
    }

    fn parse_return(&mut self) -> ast::Stmt {
        self.expect(TokenKind::Return);
        if self.lookahead.kind == TokenKind::Semicolon {
            self.expect(TokenKind::Semicolon);
            ast::Stmt::Return(None)
        } else {
            let e = self.parse_expr(0);
            self.expect(TokenKind::Semicolon);
            ast::Stmt::Return(Some(e))
        }
    }

    /*──────── expressions – Pratt ─*/
    fn parse_expr(&mut self, min_bp: u8) -> ast::Expr {
        use ast::BinOp::{
            AndAnd, EqEq, Ge, Gt, Le, Lt, Minus, NotEq, OrOr, Percent, Plus, Slash, Star,
        };

        //── prefix / unary ────────────────────────────────────────
        let mut lhs = match self.lookahead.kind {
            TokenKind::Minus => {
                self.bump();
                let rhs = self.parse_expr(ast::UnOp::Negate.binding_power());
                ast::Expr::Unary {
                    op: ast::UnOp::Negate,
                    expr: Box::new(rhs),
                }
            }
            TokenKind::Bang => {
                self.bump();
                let rhs = self.parse_expr(ast::UnOp::Not.binding_power());
                ast::Expr::Unary {
                    op: ast::UnOp::Not,
                    expr: Box::new(rhs),
                }
            }
            _ => self.parse_primary(),
        };

        //── infix / binary ────────────────────────────────────────
        loop {
            let (l_bp, r_bp, op) = match self.lookahead.kind {
                TokenKind::Plus => (1, 2, Plus),
                TokenKind::Minus => (1, 2, Minus),
                TokenKind::Star => (3, 4, Star),
                TokenKind::Slash => (3, 4, Slash),
                TokenKind::Percent => (3, 4, Percent),
                TokenKind::EqEq => (0, 0, EqEq),
                TokenKind::NotEq => (0, 0, NotEq),
                TokenKind::Lt => (0, 0, Lt),
                TokenKind::Le => (0, 0, Le),
                TokenKind::Gt => (0, 0, Gt),
                TokenKind::Ge => (0, 0, Ge),
                TokenKind::AndAnd => (0, 1, AndAnd),
                TokenKind::OrOr => (0, 1, OrOr),

                TokenKind::LParen => {
                    // call
                    let args = self.parse_call_args();
                    lhs = ast::Expr::Call {
                        callee: Box::new(lhs),
                        args,
                    };
                    continue;
                }
                _ => break,
            };

            if l_bp < min_bp {
                break;
            }
            self.bump();
            let rhs = self.parse_expr(r_bp);
            lhs = ast::Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }
        lhs
    }

    /*──────── primary ─────*/
    fn parse_primary(&mut self) -> ast::Expr {
        if self.peek(TokenKind::LParen) && self.peek_next(TokenKind::RParen) {
            self.expect(TokenKind::LParen);
            self.expect(TokenKind::RParen);
            return ast::Expr::Unit;
        }
        match &self.lookahead.kind {
            TokenKind::Ident(n) => {
                let s = n.clone();
                self.bump();
                ast::Expr::Ident(s)
            }
            TokenKind::Int(v) => {
                let v = *v;
                self.bump();
                ast::Expr::Int(v)
            }
            TokenKind::Float(v) => {
                let v = *v;
                self.bump();
                ast::Expr::Float(v)
            }
            TokenKind::Bool(b) => {
                let val = *b;
                self.bump();
                ast::Expr::Bool(val)
            }
            TokenKind::Str(s) => {
                let s = s.clone();
                self.bump();
                ast::Expr::Str(s)
            }
            TokenKind::LParen => {
                self.bump();
                if self.lookahead.kind == TokenKind::RParen {
                    self.bump();
                    ast::Expr::Unit
                } else {
                    let e = self.parse_expr(0);
                    self.expect(TokenKind::RParen);
                    e
                }
            }
            _ => panic!("unexpected token {:?}", self.lookahead.kind),
        }
    }

    /*──────── call args ───*/
    fn parse_call_args(&mut self) -> Vec<ast::Expr> {
        self.expect(TokenKind::LParen);
        let mut args = Vec::new();
        if self.lookahead.kind != TokenKind::RParen {
            args.push(self.parse_expr(0));
            while self.lookahead.kind == TokenKind::Comma {
                self.bump();
                args.push(self.parse_expr(0));
            }
        }
        self.expect(TokenKind::RParen);
        args
    }

    /*──────── helpers ─────*/
    fn expect(&mut self, kind: TokenKind) {
        if std::mem::discriminant(&self.lookahead.kind) != std::mem::discriminant(&kind) {
            panic!("expected {:?}, got {:?}", kind, self.lookahead.kind);
        }
        self.bump();
    }
    fn expect_ident(&mut self) -> String {
        if let TokenKind::Ident(s) = &self.lookahead.kind {
            let n = s.clone();
            self.bump();
            n
        } else {
            panic!("expected ident, got {:?}", self.lookahead.kind);
        }
    }
    fn bump(&mut self) {
        self.lookahead = self.lexer.next_token();
    }

    fn peek(&self, kind: TokenKind) -> bool {
        std::mem::discriminant(&self.lookahead.kind) == std::mem::discriminant(&kind)
    }

    fn peek_next(&self, kind: TokenKind) -> bool {
        let mut lx = self.lexer.clone();
        let tok = lx.next_token();
        std::mem::discriminant(&tok.kind) == std::mem::discriminant(&kind)
    }

    /*──────── global let ───*/
    fn parse_global_let(&mut self) -> ast::GlobalLet {
        self.expect(TokenKind::Let);
        let mutable = if self.lookahead.kind == TokenKind::Mut {
            self.bump();
            true
        } else {
            false
        };
        let name = self.expect_ident();
        self.expect(TokenKind::Assign);
        let expr = self.parse_expr(0);
        self.expect(TokenKind::Semicolon);

        ast::GlobalLet {
            name,
            expr,
            mutable,
        }
    }
}

/*──────── utility parsers ───*/

/// Parse a single expression from `src` using the same parser
/// implementation that is used for full modules.
pub fn parse_expr(src: &str) -> ast::Expr {
    let mut p = Parser::new(src);
    let expr = p.parse_expr(0);
    assert!(p.lookahead.kind == TokenKind::Eof, "trailing input after expression");
    expr
}

/// Parse a single statement from `src`. Currently only a subset of
/// statements used in tests is supported.
pub fn parse_stmt(src: &str) -> ast::Stmt {
    let mut p = Parser::new(src);
    let stmt = p.parse_stmt();
    assert!(p.lookahead.kind == TokenKind::Eof, "trailing input after statement");
    stmt
}

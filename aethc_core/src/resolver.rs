//! resolver.rs – name/type resolver + minimal borrow‑prep
//! Ulaz: &ast::Module  →  Izlaz: hir::HirModule + Vec<ResolveError>
//! 2025‑06: mešani Int/Float, Unary, „already defined" dup‑check.

use crate::lexer::Span;
use crate::type_::Type;
use crate::{ast, hir};
use std::collections::HashMap;

/*──────────── error type ───────────*/
#[derive(Debug, Clone)]
pub struct ResolveError {
    pub span: Span,
    pub msg: String,
}

/*──────────── entry point ──────────*/
pub fn resolve(m: &ast::Module) -> (hir::HirModule, Vec<ResolveError>) {
    let mut cx = Cx::default();
    cx.push_scope(); // global scope

    let mut items = Vec::new();
    for it in &m.items {
        match it {
            ast::Item::Function(f) => match cx.lower_fn(f) {
                Ok(h) => items.push(hir::Item::Fn(h)),
                Err(e) => cx.errors.push(e),
            },
            ast::Item::Let(gl) => match cx.lower_global_let(gl) {
                Ok(h) => items.push(hir::Item::Let(h)),
                Err(e) => cx.errors.push(e),
            },
        }
    }
    (hir::HirModule { items }, cx.errors)
}

/*──────────── context ──────────────*/
#[derive(Default)]
struct Cx {
    next_id: hir::NodeId,
    scopes: Vec<HashMap<String, Symbol>>, // stack of scopes
    errors: Vec<ResolveError>,
}

#[derive(Clone)]
struct Symbol {
    id: hir::NodeId,
    ty: Type,
    mutable: bool,
}

impl Cx {
    /*── id & scope helpers ─*/
    fn fresh(&mut self) -> hir::NodeId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }
    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    /*── symbol table insert ─*/
    fn insert(&mut self, name: &str, sym: Symbol, span: Span) -> Result<(), ResolveError> {
        let top = self.scopes.last_mut().unwrap();

        if top.contains_key(name) {
            return Err(ResolveError {
                span,
                msg: format!("name `{}` is already defined in this scope", name),
            });
        }
        top.insert(name.to_owned(), sym);
        Ok(())
    }

    fn lookup(&self, name: &str) -> Option<&Symbol> {
        self.scopes.iter().rev().find_map(|s| s.get(name))
    }

    /*──────── type lookup ───────*/
    fn resolve_type(&mut self, name: &str, span: Span) -> Result<Type, ResolveError> {
        match name {
            "Int" | "int" => Ok(Type::Int),
            "Float" | "float" => Ok(Type::Float),
            "Bool" | "bool" => Ok(Type::Bool),
            "Str" | "String" => Ok(Type::Str),
            _ => Err(ResolveError {
                span,
                msg: format!("unknown type `{name}`"),
            }),
        }
    }

    /*──────── lower fn ──────────*/
    fn lower_fn(&mut self, f: &ast::Function) -> Result<hir::HirFn, ResolveError> {
        let id = self.fresh();

        // Register function name in current scope before processing body
        self.insert(
            &f.name,
            Symbol {
                id,
                ty: Type::Unit, // Functions have Unit type for now
                mutable: false,
            },
            Span::default(),
        )?;

        self.push_scope();

        // params
        let mut params = Vec::new();
        for p in &f.params {
            let ty = if let Some(tname) = &p.ty {
                self.resolve_type(tname, Span::default())?
            } else {
                Type::Unit
            };
            let pid = self.fresh();
            self.insert(
                &p.name,
                Symbol {
                    id: pid,
                    ty: ty.clone(),
                    mutable: false,
                },
                Span::default(),
            )?;
            params.push(hir::Param {
                id: pid,
                name: p.name.clone(),
                ty,
            });
        }

        // body
        let mut stmts = Vec::new();
        for s in &f.body {
            stmts.push(self.lower_stmt(s)?);
        }
        self.pop_scope();

        Ok(hir::HirFn {
            id,
            name: f.name.clone(),
            params,
            return_ty: Type::Unit,
            body: hir::Block { id, stmts },
        })
    }

    /*──────── lower global let ─*/
    fn lower_global_let(&mut self, g: &ast::GlobalLet) -> Result<hir::HirLet, ResolveError> {
        let id = self.fresh();
        let init = self.lower_expr(&g.expr)?;
        let ty = init.ty().clone();
        self.insert(
            &g.name,
            Symbol {
                id,
                ty: ty.clone(),
                mutable: g.mutable,
            },
            Span::default(),
        )?;
        Ok(hir::HirLet {
            id,
            mutable: g.mutable,
            name: g.name.clone(),
            ty,
            init,
        })
    }

    /*──────── lower stmt ────────*/
    fn lower_stmt(&mut self, s: &ast::Stmt) -> Result<hir::Stmt, ResolveError> {
        use ast::Stmt::*;
        match s {
            Let {
                name,
                expr,
                mutable,
            } => {
                let id = self.fresh();
                let rhs = self.lower_expr(expr)?;
                let ty = rhs.ty().clone();
                self.insert(
                    name,
                    Symbol {
                        id,
                        ty: ty.clone(),
                        mutable: *mutable,
                    },
                    Span::default(),
                )?;
                Ok(hir::Stmt::Let(hir::HirLet {
                    id,
                    mutable: *mutable,
                    name: name.clone(),
                    ty,
                    init: rhs,
                }))
            }
            Expr(e) => Ok(hir::Stmt::Expr(self.lower_expr(e)?)),
            Return(opt) => Ok(hir::Stmt::Return(
                opt.as_ref().map(|e| self.lower_expr(e)).transpose()?,
            )),
        }
    }

    /*──────── lower expr ────────*/
    fn lower_expr(&mut self, e: &ast::Expr) -> Result<hir::Expr, ResolveError> {
        use ast::Expr::*;
        let id = self.fresh();
        Ok(match e {
            Ident(name) => {
                let sym = self.lookup(name).ok_or_else(|| ResolveError {
                    span: Span::default(),
                    msg: format!("unknown name `{name}`"),
                })?;
                hir::Expr::Ident {
                    id: sym.id,
                    name: name.clone(),
                    ty: sym.ty.clone(),
                }
            }
            Int(v) => hir::Expr::Int {
                id,
                value: *v,
                ty: Type::Int,
            },
            Float(v) => hir::Expr::Float {
                id,
                value: *v,
                ty: Type::Float,
            },
            Bool(b) => hir::Expr::Bool {
                id,
                value: *b,
                ty: Type::Bool,
            },
            Unit => hir::Expr::Unit {
                id,
                ty: Type::Unit,
            },
            Str(s) => hir::Expr::Str {
                id,
                value: s.clone(),
                ty: Type::Str,
            },

            Call { callee, args } => {
                let cal_h = self.lower_expr(callee)?;
                let mut a = Vec::new();
                for x in args {
                    a.push(self.lower_expr(x)?);
                }
                hir::Expr::Call {
                    id,
                    callee: Box::new(cal_h),
                    args: a,
                    ty: Type::Unit,
                }
            }

            Unary { op, expr } => {
                let operand = self.lower_expr(expr)?;
                let ty = match op {
                    ast::UnOp::Negate => {
                        if operand.ty() == &Type::Int || operand.ty() == &Type::Float {
                            operand.ty().clone()
                        } else {
                            return Err(ResolveError {
                                span: Span::default(),
                                msg: format!(
                                    "cannot negate type `{:?}`, expected Int or Float",
                                    operand.ty()
                                ),
                            });
                        }
                    }
                    ast::UnOp::Not => {
                        if operand.ty() == &Type::Bool {
                            Type::Bool
                        } else {
                            return Err(ResolveError {
                                span: Span::default(),
                                msg: format!(
                                    "cannot apply logical NOT to type `{:?}`, expected Bool",
                                    operand.ty()
                                ),
                            });
                        }
                    }
                };
                hir::Expr::Unary {
                    id,
                    op: hir::UnOp::from_ast(*op),
                    rhs: Box::new(operand),
                    ty,
                }
            }

            Binary { op, lhs, rhs } => {
                let l = self.lower_expr(lhs)?;
                let r = self.lower_expr(rhs)?;
                let ty = match op {
                    // арифметика - now supports mixed Int/Float operations
                    ast::BinOp::Plus
                    | ast::BinOp::Minus
                    | ast::BinOp::Star
                    | ast::BinOp::Slash
                    | ast::BinOp::Percent => {
                        match (l.ty(), r.ty()) {
                            (Type::Int, Type::Int) => Type::Int,
                            (Type::Float, Type::Float) => Type::Float,
                            (Type::Int, Type::Float) | (Type::Float, Type::Int) => Type::Float, // Mixed operations promote to Float
                            _ => {
                                return Err(ResolveError {
                                    span: Span::default(),
                                    msg: format!(
                                        "cannot apply {:?} to types `{:?}` and `{:?}`",
                                        op,
                                        l.ty(),
                                        r.ty()
                                    ),
                                });
                            }
                        }
                    }

                    // логика
                    ast::BinOp::AndAnd | ast::BinOp::OrOr => {
                        if l.ty() == &Type::Bool && r.ty() == &Type::Bool {
                            Type::Bool
                        } else {
                            return Err(ResolveError {
                                span: Span::default(),
                                msg: format!(
                                    "logical operation requires Bool operands, got `{:?}` and `{:?}`",
                                    l.ty(),
                                    r.ty()
                                ),
                            });
                        }
                    }
                    // сравнение - now supports mixed Int/Float comparisons
                    ast::BinOp::EqEq | ast::BinOp::NotEq => match (l.ty(), r.ty()) {
                        (Type::Int, Type::Int)
                        | (Type::Float, Type::Float)
                        | (Type::Int, Type::Float)
                        | (Type::Float, Type::Int)
                        | (Type::Bool, Type::Bool)
                        | (Type::Str, Type::Str) => Type::Bool,
                        _ => {
                            return Err(ResolveError {
                                span: Span::default(),
                                msg: format!(
                                    "cannot compare types `{:?}` and `{:?}`",
                                    l.ty(),
                                    r.ty()
                                ),
                            });
                        }
                    },
                    ast::BinOp::Lt | ast::BinOp::Le | ast::BinOp::Gt | ast::BinOp::Ge => {
                        match (l.ty(), r.ty()) {
                            (Type::Int, Type::Int)
                            | (Type::Float, Type::Float)
                            | (Type::Int, Type::Float)
                            | (Type::Float, Type::Int) => Type::Bool,
                            _ => {
                                return Err(ResolveError {
                                    span: Span::default(),
                                    msg: format!(
                                        "cannot order-compare types `{:?}` and `{:?}`",
                                        l.ty(),
                                        r.ty()
                                    ),
                                });
                            }
                        }
                    }
                };
                hir::Expr::Binary {
                    id,
                    lhs: Box::new(l),
                    op: hir::BinOp::from_ast(*op),
                    rhs: Box::new(r),
                    ty,
                }
            }
        })
    }
}

/*─────────── unary op conversion ──*/
impl hir::UnOp {
    pub fn from_ast(op: ast::UnOp) -> Self {
        use ast::UnOp::*;
        match op {
            Negate => hir::UnOp::Negate,
            Not => hir::UnOp::Not,
        }
    }
}

/*─────────── binop conversion ──*/
impl hir::BinOp {
    pub fn from_ast(op: ast::BinOp) -> Self {
        use ast::BinOp::*;
        match op {
            Plus => hir::BinOp::Plus,
            Minus => hir::BinOp::Minus,
            Star => hir::BinOp::Star,
            Slash => hir::BinOp::Slash,
            Percent => hir::BinOp::Percent,
            EqEq => hir::BinOp::EqEq,
            NotEq => hir::BinOp::NotEq,
            Lt => hir::BinOp::Lt,
            Le => hir::BinOp::Le,
            Gt => hir::BinOp::Gt,
            Ge => hir::BinOp::Ge,
            AndAnd => hir::BinOp::AndAnd,
            OrOr => hir::BinOp::OrOr,
        }
    }
}

/*─────────── tests ──────────────*/
#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    #[test]
    fn int_less_than_int_is_bool() {
        let src = "fn main(){ let b = 1 < 2; }";
        let (hir_mod, errs) = resolve(&Parser::new(src).parse_module());
        assert!(errs.is_empty());
        if let hir::Item::Fn(f) = &hir_mod.items[0] {
            if let hir::Stmt::Let(l) = &f.body.stmts[0] {
                assert_eq!(l.ty, Type::Bool);
            }
        }
    }

    #[test]
    fn int_eq_float_is_ok_now() {
        let src = "fn main(){ let b = 1 == 2.0; }";
        let (hir_mod, errs) = resolve(&Parser::new(src).parse_module());
        assert!(errs.is_empty()); // Should now work
        if let hir::Item::Fn(f) = &hir_mod.items[0] {
            if let hir::Stmt::Let(l) = &f.body.stmts[0] {
                assert_eq!(l.ty, Type::Bool);
            }
        }
    }

    #[test]
    fn mixed_arithmetic_promotes_to_float() {
        let src = "fn main(){ let f = 1 + 2.0; }";
        let (hir_mod, errs) = resolve(&Parser::new(src).parse_module());
        assert!(errs.is_empty());
        if let hir::Item::Fn(f) = &hir_mod.items[0] {
            if let hir::Stmt::Let(l) = &f.body.stmts[0] {
                assert_eq!(l.ty, Type::Float);
            }
        }
    }

    #[test]
    fn unary_negate_int() {
        let src = "fn main(){ let x = -42; }";
        let (hir_mod, errs) = resolve(&Parser::new(src).parse_module());
        assert!(errs.is_empty());
        if let hir::Item::Fn(f) = &hir_mod.items[0] {
            if let hir::Stmt::Let(l) = &f.body.stmts[0] {
                assert_eq!(l.ty, Type::Int);
            }
        }
    }

    #[test]
    fn unary_not_bool() {
        let src = "fn main(){ let b = !true; }";
        let (hir_mod, errs) = resolve(&Parser::new(src).parse_module());
        assert!(errs.is_empty());
        if let hir::Item::Fn(f) = &hir_mod.items[0] {
            if let hir::Stmt::Let(l) = &f.body.stmts[0] {
                assert_eq!(l.ty, Type::Bool);
            }
        }
    }

    #[test]
    fn duplicate_definition_error() {
        let src = "fn main(){ let x = 1; let x = 2; }";
        let (_hir, errs) = resolve(&Parser::new(src).parse_module());
        assert_eq!(errs.len(), 1);
        assert!(errs[0].msg.contains("already defined"));
    }

    #[test]
    fn unary_negate_string_error() {
        let src = r#"fn main(){ let x = -"hello"; }"#;
        let (_hir, errs) = resolve(&Parser::new(src).parse_module());
        assert_eq!(errs.len(), 1);
        assert!(errs[0].msg.contains("cannot negate"));
    }
}

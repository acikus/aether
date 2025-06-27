//! resolver.rs – name/type resolver + minimal borrow‑prep
//! Ulaz: &ast::Module  →  Izlaz: hir::HirModule + Vec<ResolveError>
//! 2025‑06 – ažurirano: mešani Int/Float izrazi i poruka „already defined“.

use crate::{ast, hir};
use crate::type_::Type;
use crate::lexer::Span;
use std::collections::HashMap;

/*──────────── error type ───────────*/
#[derive(Debug, Clone)]
pub struct ResolveError {
    pub span: Span,
    pub msg:  String,
}

/*──────────── entry point ──────────*/
pub fn resolve(m: &ast::Module) -> (hir::HirModule, Vec<ResolveError>) {
    let mut cx = Cx::default();
    cx.push_scope();                     // global scope

    let mut items = Vec::new();
    for it in &m.items {
        match it {
            ast::Item::Function(f) => match cx.lower_fn(f) {
                Ok(h)  => items.push(hir::Item::Fn(h)),
                Err(e) => cx.errors.push(e),
            },
            ast::Item::Let(gl) => match cx.lower_global_let(gl) {
                Ok(h)  => items.push(hir::Item::Let(h)),
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
    scopes:  Vec<HashMap<String, Symbol>>, // stack of scopes
    errors:  Vec<ResolveError>,
}

#[allow(dead_code)]
#[derive(Clone)]
struct Symbol {
    id:       hir::NodeId,
    ty:       Type,
    mutable:  bool,
}

impl Cx {
    /*── id & scope helpers ─*/
    fn fresh(&mut self) -> hir::NodeId { let id = self.next_id; self.next_id += 1; id }
    fn push_scope(&mut self)            { self.scopes.push(HashMap::new()); }
    fn pop_scope(&mut self)             { self.scopes.pop(); }

    /*── symbol table insert ─*/
    fn insert(&mut self, name: &str, sym: Symbol, span: Span) {
        let top = self.scopes.last_mut().unwrap();
        match top.get(name) {
            Some(prev) if !prev.mutable => {
                // immutable već postoji → greška
                self.errors.push(ResolveError{
                    span,
                    msg: format!("already defined `{name}`"),
                });
            }
            _ => { top.insert(name.to_owned(), sym); }
        }
    }

    fn lookup(&self, name: &str) -> Option<&Symbol> {
        self.scopes.iter().rev().find_map(|s| s.get(name))
    }

    /*──────── type lookup ───────*/
    fn resolve_type(&mut self, name: &str, span: Span) -> Result<Type, ResolveError> {
        match name {
            "Int" | "int"     => Ok(Type::Int),
            "Float" | "float" => Ok(Type::Float),
            "Bool" | "bool"   => Ok(Type::Bool),
            "Str" | "String"  => Ok(Type::Str),
            _ => Err(ResolveError{ span, msg: format!("unknown type `{name}`") }),
        }
    }

    /*──────── lower fn ──────────*/
    fn lower_fn(&mut self, f: &ast::Function) -> Result<hir::HirFn, ResolveError> {
        let id = self.fresh();
        self.push_scope();

        // params
        let mut params = Vec::new();
        for p in &f.params {
            let ty = if let Some(tname) = &p.ty {
                self.resolve_type(tname, Span::default())?
            } else { Type::Unit };
            let pid = self.fresh();
            self.insert(&p.name, Symbol{ id:pid, ty:ty.clone(), mutable:false }, Span::default());
            params.push(hir::Param{ id:pid, name:p.name.clone(), ty });
        }

        // body
        let mut stmts = Vec::new();
        for s in &f.body { stmts.push(self.lower_stmt(s)?); }
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
        self.insert(&g.name, Symbol{ id, ty:ty.clone(), mutable:g.mutable }, Span::default());
        Ok(hir::HirLet{ id, mutable:g.mutable, name:g.name.clone(), ty, init })
    }

    /*──────── lower stmt ────────*/
    fn lower_stmt(&mut self, s: &ast::Stmt) -> Result<hir::Stmt, ResolveError> {
        use ast::Stmt::*;
        match s {
            Let { name, expr, mutable } => {
                let id  = self.fresh();
                let rhs = self.lower_expr(expr)?;
                let ty  = rhs.ty().clone();

                if let Some(prev) = self.scopes.last().unwrap().get(name) {
                    if !prev.mutable {

                        // duplicate immutable binding – report an error and do not shadow
                        self.errors.push(ResolveError {
                            span: Span::default(),
                            msg: format!("cannot reassign immutable binding `{name}`"),

                        });
                        return Ok(hir::Stmt::Expr(rhs));
                    }
                }

                self.insert(name, Symbol { id, ty: ty.clone(), mutable: *mutable }, Span::default());
                Ok(hir::Stmt::Let(hir::HirLet { id, mutable: *mutable, name: name.clone(), ty, init: rhs }))
            }
            Expr(e) => Ok(hir::Stmt::Expr(self.lower_expr(e)?)),
            Return(opt) => Ok(hir::Stmt::Return(opt.as_ref().map(|e| self.lower_expr(e)).transpose()?)),
        }
    }

    /*──────── lower expr ────────*/
    fn lower_expr(&mut self, e: &ast::Expr) -> Result<hir::Expr, ResolveError> {
        use ast::Expr::*;
        let id = self.fresh();
        Ok(match e {
            Ident(name) => {
                let sym = self.lookup(name).ok_or_else(|| ResolveError{ span:Span::default(), msg:format!("unknown name `{name}`") })?;
                hir::Expr::Ident{ id:sym.id, name:name.clone(), ty:sym.ty.clone() }
            }
            Int(v)   => hir::Expr::Int  { id, value:*v, ty:Type::Int   },
            Float(v) => hir::Expr::Float{ id, value:*v, ty:Type::Float },
            Str(s)   => hir::Expr::Str  { id, value:s.clone(), ty:Type::Str },

            Call{callee,args} => {
                let cal_h = self.lower_expr(callee)?;
                let mut a = Vec::new();
                for x in args { a.push(self.lower_expr(x)?); }
                hir::Expr::Call{ id, callee:Box::new(cal_h), args:a, ty:Type::Unit }
            }

            Binary{op,lhs,rhs} => {
                let l = self.lower_expr(lhs)?;
                let r = self.lower_expr(rhs)?;
                let ty = match op {
                    // арифметика
                    ast::BinOp::Plus | ast::BinOp::Minus | ast::BinOp::Star | ast::BinOp::Slash | ast::BinOp::Percent => {
                        match (l.ty(), r.ty()) {
                            (Type::Int,   Type::Int)   => Type::Int,
                            (Type::Float, Type::Float) => Type::Float,
                            (Type::Int,   Type::Float)
                          | (Type::Float, Type::Int)   => Type::Float, // ново правило
                            _ => {
                                self.errors.push(ResolveError{ span:Span::default(), msg:"type mismatch in arithmetic".into() });
                                Type::Unit
                            }
                        }
                    }
                    // поређења
                    _ => Type::Bool,
                };
                hir::Expr::Binary{ id, lhs:Box::new(l), op:*op, rhs:Box::new(r), ty }
            }
        })
    }
}

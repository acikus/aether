use crate::ast::{self, BinOp};
use crate::lexer::Span;
use crate::parser;
use crate::infer_ctx::{InferCtx, TvOrTy, Constraint, Ty};

fn gen_constraints(expr: &ast::Expr, cx: &mut InferCtx) -> TvOrTy {
    use ast::Expr::*;
    match expr {
        Int(_) => {
            let tv = cx.fresh(Span::default());
            cx.constraints.push_back(Constraint::Eq(tv.clone(), TvOrTy::Ty(Ty::Int)));
            tv
        }
        Float(_) => {
            let tv = cx.fresh(Span::default());
            cx.constraints.push_back(Constraint::Eq(tv.clone(), TvOrTy::Ty(Ty::Float)));
            tv
        }
        Bool(_) => {
            let tv = cx.fresh(Span::default());
            cx.constraints.push_back(Constraint::Eq(tv.clone(), TvOrTy::Ty(Ty::Bool)));
            tv
        }
        Str(_) => {
            let tv = cx.fresh(Span::default());
            cx.constraints.push_back(Constraint::Eq(tv.clone(), TvOrTy::Ty(Ty::Str)));
            tv
        }
        Unit => {
            let tv = cx.fresh(Span::default());
            cx.constraints.push_back(Constraint::Eq(tv.clone(), TvOrTy::Ty(Ty::Unit)));
            tv
        }
        Binary { op, lhs, rhs } => {
            let l = gen_constraints(lhs, cx);
            let r = gen_constraints(rhs, cx);
            let res = cx.fresh(Span::default());
            cx.constraints.push_back(Constraint::Eq(l.clone(), r.clone()));
            match op {
                BinOp::Plus | BinOp::Minus | BinOp::Star | BinOp::Slash => {
                    cx.constraints.push_back(Constraint::Eq(res.clone(), l));
                }
                BinOp::EqEq | BinOp::NotEq | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                    cx.constraints.push_back(Constraint::Eq(res.clone(), TvOrTy::Ty(Ty::Bool)));
                }
                _ => {}
            }
            res
        }
        Unary { expr, .. } => gen_constraints(expr, cx),
        Ident(_) | Call { .. } => cx.fresh(Span::default()),
    }
}

pub fn infer_expr(expr: &ast::Expr) -> Result<Ty, String> {
    let mut cx = InferCtx::new();
    let root = gen_constraints(expr, &mut cx);
    cx.solve()?;
    match cx.apply(root) {
        TvOrTy::Ty(t) => Ok(t),
        TvOrTy::Var(_) => Err("cannot infer type".to_string()),
    }
}

/// Convenience wrapper for tests: parse and infer a single expression.
pub fn infer_str(src: &str) -> Result<Ty, String> {
    let expr = parser::parse_expr(src);
    infer_expr(&expr)
}

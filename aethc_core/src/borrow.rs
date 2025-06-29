// borrow.rs - block-local borrow checker

use std::collections::HashMap;

use crate::hir::{self, Expr, Stmt};
use crate::lexer::Span;

pub type VarId = hir::NodeId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BorrowState {
    Live,
    Moved,
    MutBorrowed(BorrowId),
}

pub type BorrowId = u32;

pub type VarStates = HashMap<VarId, BorrowState>;

pub struct BorrowCtx<'hir> {
    states: VarStates,
    errors: Vec<BorrowError>,
    hir: &'hir hir::Block,
    next_borrow_id: BorrowId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BorrowErrorKind {
    UseAfterMove,
    AssignWhileBorrowed,
    SecondMutBorrow,
}

#[derive(Clone, Debug)]
pub struct BorrowError {
    pub kind: BorrowErrorKind,
    pub span: Span,
    pub prev_span: Span,
}

impl<'hir> BorrowCtx<'hir> {
    pub fn new(hir: &'hir hir::Block) -> Self {
        Self {
            states: HashMap::new(),
            errors: Vec::new(),
            hir,
            next_borrow_id: 0,
        }
    }

    pub fn check(mut self) -> Vec<BorrowError> {
        for stmt in &self.hir.stmts {
            self.check_stmt(stmt);
            self.cleanup();
        }
        self.errors
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let(l) => {
                self.check_expr(&l.init, true);
                self.states.insert(l.id, BorrowState::Live);
            }
            Stmt::Assign { id, expr, .. } => {
                if matches!(self.states.get(id), Some(BorrowState::MutBorrowed(_))) {
                    self.errors.push(BorrowError {
                        kind: BorrowErrorKind::AssignWhileBorrowed,
                        span: Span::default(),
                        prev_span: Span::default(),
                    });
                }
                self.check_expr(expr, true);
                self.states.insert(*id, BorrowState::Live);
            }
            Stmt::Expr(e) | Stmt::Semi(e) => {
                self.check_expr(e, false);
            }
            Stmt::Return(opt) => {
                if let Some(e) = opt {
                    self.check_expr(e, true);
                }
            }
        }
    }

    fn check_expr(&mut self, expr: &Expr, move_ctx: bool) {
        use Expr::*;
        match expr {
            Ident { id, ty, .. } => {
                if move_ctx && !is_copy_type(ty) {
                    self.move_var(*id);
                } else {
                    self.use_var(*id);
                }
            }
            Binary { lhs, rhs, .. } => {
                self.check_expr(lhs, false);
                self.check_expr(rhs, false);
            }
            Unary { rhs, .. } => {
                self.check_expr(rhs, false);
            }
            Call { callee, args, .. } => {
                self.check_expr(callee, false);
                for a in args {
                    self.check_expr(a, false);
                }
            }
            _ => {}
        }
    }

    fn use_var(&mut self, id: VarId) {
        if let Some(state) = self.states.get(&id) {
            if let BorrowState::Moved = state {
                self.errors.push(BorrowError {
                    kind: BorrowErrorKind::UseAfterMove,
                    span: Span::default(),
                    prev_span: Span::default(),
                });
            }
        }
    }

    fn move_var(&mut self, id: VarId) {
        if let Some(state) = self.states.get(&id) {
            match state {
                BorrowState::MutBorrowed(_) => self.errors.push(BorrowError {
                    kind: BorrowErrorKind::AssignWhileBorrowed,
                    span: Span::default(),
                    prev_span: Span::default(),
                }),
                BorrowState::Moved => self.errors.push(BorrowError {
                    kind: BorrowErrorKind::UseAfterMove,
                    span: Span::default(),
                    prev_span: Span::default(),
                }),
                _ => {}
            }
        }
        self.states.insert(id, BorrowState::Moved);
    }

    fn cleanup(&mut self) {
        for state in self.states.values_mut() {
            if matches!(state, BorrowState::MutBorrowed(_)) {
                *state = BorrowState::Live;
            }
        }
    }
}

pub fn check_fn_body(body: &hir::Block) -> Vec<BorrowError> {
    BorrowCtx::new(body).check()
}

fn is_copy_type(ty: &crate::type_::Type) -> bool {
    use crate::type_::Type::*;
    matches!(ty, Int | Float | Bool | Unit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    use crate::resolver::resolve;

    #[test]
    fn simple_reassignment() {
        let src = "fn main(){ let mut x = 1; x = 2; }";
        let (hir_mod, errs) = resolve(&Parser::new(src).parse_module());
        assert!(errs.is_empty());
        if let hir::Item::Fn(f) = &hir_mod.items[0] {
            let errs = check_fn_body(&f.body);
            assert!(errs.is_empty());
        }
    }
}


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
    pub states: VarStates,
    pub errors: Vec<BorrowError>,
    pub hir: &'hir hir::Block,
    pub next_borrow_id: BorrowId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BorrowErrorKind {
    UseAfterMove,
    AssignWhileBorrowed,
    SecondMutBorrow,
    DoubleMove,
}

impl BorrowErrorKind {
    pub fn code(&self) -> &'static str {
        match self {
            BorrowErrorKind::AssignWhileBorrowed | BorrowErrorKind::SecondMutBorrow => "E010",
            BorrowErrorKind::UseAfterMove | BorrowErrorKind::DoubleMove => "E011",
        }
    }
}

#[derive(Clone, Debug)]
pub struct BorrowError {
    pub code: &'static str,
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
                    let kind = BorrowErrorKind::AssignWhileBorrowed;
                    self.errors.push(BorrowError {
                        code: kind.code(),
                        kind,
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
                if move_ctx && !ty.is_copy() {
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
                    self.check_expr(a, true);
                }
            }
            _ => {}
        }
    }

    pub fn use_var(&mut self, id: VarId) {
        if let Some(state) = self.states.get(&id) {
            if let BorrowState::Moved = state {
                self.errors.push(BorrowError {
                    code: BorrowErrorKind::UseAfterMove.code(),
                    kind: BorrowErrorKind::UseAfterMove,
                    span: Span::default(),
                    prev_span: Span::default(),
                });
            }
        }
    }

    pub fn move_var(&mut self, id: VarId) {
        if let Some(state) = self.states.get(&id) {
            match state {
                BorrowState::MutBorrowed(_) => {
                    let kind = BorrowErrorKind::AssignWhileBorrowed;
                    self.errors.push(BorrowError {
                        code: kind.code(),
                        kind,
                        span: Span::default(),
                        prev_span: Span::default(),
                    })
                }
                BorrowState::Moved => {
                    let kind = BorrowErrorKind::DoubleMove;
                    self.errors.push(BorrowError {
                        code: kind.code(),
                        kind,
                        span: Span::default(),
                        prev_span: Span::default(),
                    })
                }
                _ => {}
            }
        }
        self.states.insert(id, BorrowState::Moved);
    }

    pub fn borrow_var(&mut self, id: VarId) {
        if let Some(state) = self.states.get(&id) {
            match state {
                BorrowState::MutBorrowed(_) => {
                    let kind = BorrowErrorKind::SecondMutBorrow;
                    self.errors.push(BorrowError {
                        code: kind.code(),
                        kind,
                        span: Span::default(),
                        prev_span: Span::default(),
                    });
                }
                BorrowState::Moved => {
                    let kind = BorrowErrorKind::UseAfterMove;
                    self.errors.push(BorrowError {
                        code: kind.code(),
                        kind,
                        span: Span::default(),
                        prev_span: Span::default(),
                    });
                }
                _ => {}
            }
        }
        self.states.insert(id, BorrowState::MutBorrowed(self.next_borrow_id));
        self.next_borrow_id += 1;
    }

    pub fn cleanup(&mut self) {
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


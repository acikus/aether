use crate::lexer::Span;
use std::collections::{HashMap, VecDeque};

pub type TypeVarId = u32;

#[derive(Clone, Debug)]
pub struct TypeVar {
    pub id: TypeVarId,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ty {
    Int,
    Float,
    Bool,
    Str,
    Unit,
    Error,
    Var(TypeVarId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TvOrTy {
    Var(TypeVarId),
    Ty(Ty),
}

#[derive(Clone, Debug)]
pub struct Constraint {
    pub left: TvOrTy,
    pub right: TvOrTy,
    pub left_span: Span,
    pub right_span: Span,
}

#[derive(Clone, Debug)]
pub struct TypeError {
    pub kind: TypeErrorKind,
    pub primary_span: Span,
    pub secondary_span: Span,
}

#[derive(Clone, Debug)]
pub enum TypeErrorKind {
    Mismatch { found: Ty, expected: Ty },
}

pub struct InferCtx {
    next_tv: TypeVarId,
    pub vars: Vec<TypeVar>,
    pub constraints: VecDeque<Constraint>,
    pub subst: HashMap<TypeVarId, Ty>,
    pub errors: Vec<TypeError>,
}

impl InferCtx {
    pub fn new() -> Self {
        Self {
            next_tv: 0,
            vars: Vec::new(),
            constraints: VecDeque::new(),
            subst: HashMap::new(),
            errors: Vec::new(),
        }
    }

    pub fn fresh(&mut self, span: Span) -> TvOrTy {
        let id = self.next_tv;
        self.next_tv += 1;
        self.vars.push(TypeVar { id, span });
        TvOrTy::Var(id)
    }

    fn apply_ty(&self, ty: Ty) -> Ty {
        match ty {
            Ty::Var(v) => match self.subst.get(&v) {
                Some(t) => self.apply_ty(t.clone()),
                None => Ty::Var(v),
            },
            Ty::Error => Ty::Error,
            other => other,
        }
    }

    pub fn apply(&self, x: TvOrTy) -> TvOrTy {
        match x {
            TvOrTy::Var(v) => match self.subst.get(&v) {
                Some(t) => TvOrTy::Ty(self.apply_ty(t.clone())),
                None => TvOrTy::Var(v),
            },
            TvOrTy::Ty(t) => TvOrTy::Ty(self.apply_ty(t)),
        }
    }

    pub fn solve(&mut self) {
        while let Some(c) = self.constraints.pop_front() {
            let a_res = self.apply(c.left.clone());
            let b_res = self.apply(c.right.clone());

            if a_res == b_res {
                continue;
            }

            match (a_res, b_res) {
                (TvOrTy::Var(v), TvOrTy::Ty(ty)) | (TvOrTy::Ty(ty), TvOrTy::Var(v)) => {
                    self.subst.insert(v, ty);
                }
                (TvOrTy::Var(v1), TvOrTy::Var(v2)) => {
                    self.subst.insert(v1, Ty::Var(v2));
                }
                (TvOrTy::Ty(ty1), TvOrTy::Ty(ty2)) => {
                    let unified = match (ty1.clone(), ty2.clone()) {
                        (Ty::Int, Ty::Float) | (Ty::Float, Ty::Int) => Some(Ty::Float),
                        (ref x, ref y) if x == y => Some(x.clone()),
                        (found, expected) => {
                            self.errors.push(TypeError {
                                kind: TypeErrorKind::Mismatch { found, expected },
                                primary_span: c.left_span,
                                secondary_span: c.right_span,
                            });
                            Some(Ty::Error)
                        }
                    };

                    if let Some(t) = unified {
                        if t == Ty::Float {
                            self.subst_promote_float();
                        }
                    }
                }
            }
        }
    }

    fn subst_promote_float(&mut self) {
        for v in self.subst.values_mut() {
            if matches!(v, Ty::Int) {
                *v = Ty::Float;
            }
        }
    }
}

impl Ty {
    pub fn unify(a: &Ty, b: &Ty) -> Result<Ty, ()> {
        use Ty::*;
        match (a, b) {
            (Int, Float) | (Float, Int) => Ok(Float),
            (Int, Int) => Ok(Int),
            (Float, Float) => Ok(Float),
            (Bool, Bool) => Ok(Bool),
            (Str, Str) => Ok(Str),
            (Unit, Unit) => Ok(Unit),
            (Error, _) | (_, Error) => Ok(Error),
            _ => Err(()),
        }
    }
}

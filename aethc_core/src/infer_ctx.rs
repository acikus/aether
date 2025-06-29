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
    Var(TypeVarId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TvOrTy {
    Var(TypeVarId),
    Ty(Ty),
}

#[derive(Clone, Debug)]
pub enum Constraint {
    Eq(TvOrTy, TvOrTy),
}

pub struct InferCtx {
    next_tv: TypeVarId,
    pub vars: Vec<TypeVar>,
    pub constraints: VecDeque<Constraint>,
    pub subst: HashMap<TypeVarId, Ty>,
}

impl InferCtx {
    pub fn new() -> Self {
        Self { next_tv: 0, vars: Vec::new(), constraints: VecDeque::new(), subst: HashMap::new() }
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

    pub fn solve(&mut self) -> Result<(), String> {
        while let Some(c) = self.constraints.pop_front() {
            match c {
                Constraint::Eq(a, b) => {
                    let a_res = self.apply(a.clone());
                    let b_res = self.apply(b.clone());
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
                            match Ty::unify(&ty1, &ty2) {
                                Ok(t) => {
                                    if t == Ty::Float {
                                        self.subst_promote_float();
                                    }
                                }
                                Err(_) => {
                                    return Err(format!("type mismatch: {:?} vs {:?}", ty1, ty2));
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
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
            _ => Err(()),
        }
    }
}

//! hir.rs – High‑level IR (after resolve)
//! Potpuno usklađeno sa novim UnOp + Binary/Unary operacijama

pub use crate::ast::BinOp;
pub use crate::ast::UnOp;
use crate::type_::Type;

pub type NodeId = u32; // simple counter assigned by resolver

#[derive(Debug, Clone)]
pub enum Builtin {
    Print,
}

/*─────────── HIR root ───────────*/
#[derive(Debug, Clone)]
pub struct HirModule {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone)]
pub enum Item {
    Fn(HirFn),
    Let(HirLet), // global let 0.1
}

/*─────────── functions ──────────*/
#[derive(Debug, Clone)]
pub struct HirFn {
    pub id: NodeId,
    pub name: String,
    pub params: Vec<Param>,
    pub return_ty: Type,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub id: NodeId,
    pub name: String,
    pub ty: Type,
}

/*─────────── let binding ────────*/
#[derive(Debug, Clone)]
pub struct HirLet {
    pub id: NodeId,
    pub mutable: bool,
    pub name: String,
    pub ty: Type,
    pub init: Expr,
}

/*─────────── statements ─────────*/
#[derive(Debug, Clone)]
pub struct Block {
    pub id: NodeId,
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let(HirLet),
    Assign { id: NodeId, name: String, expr: Expr },
    Expr(Expr), // value used
    Semi(Expr), // value ignored
    Return(Option<Expr>),
}

/*─────────── expressions ───────*/
#[derive(Debug, Clone)]
pub enum Expr {
    Ident {
        id: NodeId,
        name: String,
        ty: Type,
    },
    Int {
        id: NodeId,
        value: i64,
        ty: Type,
    },
    Float {
        id: NodeId,
        value: f64,
        ty: Type,
    },
    Bool {
        id: NodeId,
        value: bool,
        ty: Type,
    },
    Unit {
        id: NodeId,
        ty: Type,
    },
    Str {
        id: NodeId,
        value: String,
        ty: Type,
    },
    Builtin {
        id: NodeId,
        kind: Builtin,
        ty: Type,
    },
    Call {
        id: NodeId,
        callee: Box<Expr>,
        args: Vec<Expr>,
        ty: Type,
    },
    Unary {
        id: NodeId,
        op: UnOp,
        rhs: Box<Expr>,
        ty: Type,
    },
    Binary {
        id: NodeId,
        lhs: Box<Expr>,
        op: BinOp,
        rhs: Box<Expr>,
        ty: Type,
    },
}

impl Expr {
    /// Get type of expression (after resolver filled it)
    pub fn ty(&self) -> &Type {
        use Expr::*;
        match self {
            Ident { ty, .. }
            | Int { ty, .. }
            | Float { ty, .. }
            | Bool { ty, .. }
            | Unit { ty, .. }
            | Str { ty, .. }
            | Builtin { ty, .. }
            | Call { ty, .. }
            | Unary { ty, .. }
            | Binary { ty, .. } => ty,
        }
    }

    /// Treat a block as Unit expression (placeholder until we have real value)
    pub fn from_block(b: Block) -> Self {
        Expr::Unit { id: b.id, ty: Type::Unit }
    }
}

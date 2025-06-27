use crate::type_::Type;
use crate::ast::BinOp;

pub type NodeId = u32;   // simple counter assigned by resolver

#[derive(Debug)]
pub struct HirModule {
    pub items: Vec<Item>,
}

#[derive(Debug)]
pub enum Item {
    Fn(HirFn),
    Let(HirLet), // global let for 0.1
}

#[derive(Debug)]
pub struct HirFn {
    pub id: NodeId,
    pub name: String,
    pub params: Vec<Param>,
    pub return_ty: Type,
    pub body: Block,
}

#[derive(Debug)]
pub struct Param {
    pub id: NodeId,
    pub name: String,
    pub ty: Type,
}

#[derive(Debug)]
pub struct HirLet {
    pub id: NodeId,
    pub mutable: bool,
    pub name: String,
    pub ty: Type,
    pub init: Expr,
}

#[derive(Debug)]
pub struct Block {
    pub id: NodeId,
    pub stmts: Vec<Stmt>,
}

#[derive(Debug)]
pub enum Stmt {
    Let(HirLet),
    Expr(Expr),
    Semi(Expr),
    Return(Option<Expr>),
}

#[derive(Debug)]
pub enum Expr {
    Ident { id: NodeId, name: String, ty: Type },
    Int  { id: NodeId, value: i64, ty: Type },
    Float{ id: NodeId, value: f64, ty: Type },
    Str  { id: NodeId, value: String, ty: Type },
    Call { id: NodeId, callee: Box<Expr>, args: Vec<Expr>, ty: Type },
    // Unary operations will be supported later
    Binary{ id: NodeId, lhs: Box<Expr>, op: BinOp, rhs: Box<Expr>, ty: Type },
}

impl Expr {
    pub fn ty(&self) -> &Type {
        use Expr::*;
        match self {
            Ident { ty, .. }
            | Int { ty, .. }
            | Float { ty, .. }
            | Str { ty, .. }
            | Call { ty, .. }
            | Binary { ty, .. } => ty,
        }
    }

    pub fn from_block(b: Block) -> Self {
        // treat block as Unit expression for now
        Expr::Call {
            id: b.id,
            callee: Box::new(Expr::Ident { id: b.id, name: "{block}".into(), ty: Type::Unit }),
            args: vec![],
            ty: Type::Unit,
        }
    }
}

// ast.rs – zajednički, netipizovan AST

#[derive(Debug, Clone)]
pub struct Module {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone)]
pub enum Item {
    Function(Function),
    Let(GlobalLet), //  globalni let
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: Option<String>, // ako postoji anotacija:  x: Int
}

#[derive(Debug, Clone)]
pub struct GlobalLet {
    pub name: String,
    pub expr: Expr,
    pub mutable: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    EqEq,
    NotEq,
    Lt,
    Le,
    Gt,
    Ge,
    AndAnd,
    OrOr,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Negate,
    Not,
}
impl UnOp {
    pub fn binding_power(&self) -> u8 {
        match self {
            UnOp::Negate => 5, // Example: Higher than multiplication/division (3,4)
            UnOp::Not => 5,    // Example: Higher than logical AND/OR (0,1)
        }
    }
}
#[derive(Debug, Clone)]
pub enum Stmt {
    Let {
        name: String,
        expr: Expr,
        mutable: bool,
    },
    Expr(Expr),
    Return(Option<Expr>),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Ident(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Unit,
    Str(String),
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    Binary {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Unary {
        op: UnOp,
        expr: Box<Expr>,
    },
}

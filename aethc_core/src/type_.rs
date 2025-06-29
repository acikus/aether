// type.rs  (извезен као crate::type_)

use std::fmt;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Int,
    Float,
    Bool,
    Str,
    Unit,                     // ()
    Custom(String),           // struct / enum (за 0.1 само име)
    Ref {                     // &T / &mut T
        mutability: bool,
        inner: Box<Type>,
        lifetime: Option<String>,
    },
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Type::*;
        match self {
            Int   => write!(f, "Int"),
            Float => write!(f, "Float"),
            Bool  => write!(f, "Bool"),
            Str   => write!(f, "String"),
            Unit  => write!(f, "()"),
            Custom(s) => write!(f, "{s}"),
            Ref { mutability, inner, lifetime } => {
                write!(f, "&")?;
                if *mutability { write!(f, "mut ")?; }
                if let Some(l) = lifetime { write!(f, "'{l} ")?; }
                write!(f, "{inner:?}")
            }
        }
    }
}

impl Type {
    /// Attempt to unify two types. Int and Float unify to Float.
    pub fn unify(a: &Type, b: &Type) -> Result<Type, ()> {
        use Type::*;
        match (a, b) {
            (Int, Float) | (Float, Int) => Ok(Float),
            (Int, Int) => Ok(Int),
            (Float, Float) => Ok(Float),
            (Bool, Bool) => Ok(Bool),
            (Str, Str) => Ok(Str),
            (Unit, Unit) => Ok(Unit),
            (Custom(x), Custom(y)) if x == y => Ok(Custom(x.clone())),
            _ => Err(()),
        }
    }
}

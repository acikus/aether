//! mir.rs - Minimal MIR representation and lowering from HIR
use crate::hir::{self, Expr, Stmt};

pub type BlockId = u32;
pub type TempId = u32;
pub type VarId = hir::NodeId;

#[derive(Debug, Clone)]
pub enum MirType {
    Int,
    Float,
    Bool,
    Str,
    Unit,
}

#[derive(Debug, Clone)]
pub enum Constant {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Unit,
}

#[derive(Debug, Clone)]
pub enum Operand {
    Const(Constant),
    Var(VarId),
    Temp(TempId),
}

#[derive(Debug, Clone)]
pub enum Rvalue {
    Use(Operand),
    BinaryOp { op: hir::BinOp, lhs: Operand, rhs: Operand },
    UnaryOp { op: hir::UnOp, src: Operand },
    Call { fn_name: String, args: Vec<Operand> },
}

#[derive(Debug, Clone)]
pub enum Statement {
    Assign { dst: TempId, rv: Rvalue },
    StorageLive(TempId),
    StorageDead(TempId),
}

#[derive(Debug, Clone)]
pub enum Terminator {
    Return,
    Goto(BlockId),
    CondBranch { cond: Operand, then_bb: BlockId, else_bb: BlockId },
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub stmts: Vec<Statement>,
    pub term: Terminator,
}

#[derive(Debug, Clone)]
pub struct MirBody {
    pub blocks: Vec<BasicBlock>,
    pub ret_ty: MirType,
}

pub const RET_TEMP: TempId = u32::MAX;

struct LowerCtx<'a> {
    hir: &'a hir::Block,
    blocks: Vec<BasicBlock>,
    cur_block: BlockId,
    next_temp: TempId,
}

impl<'a> LowerCtx<'a> {
    fn new(hir: &'a hir::Block) -> Self {
        Self {
            hir,
            blocks: vec![BasicBlock { stmts: Vec::new(), term: Terminator::Return }],
            cur_block: 0,
            next_temp: 0,
        }
    }

    fn fresh_temp(&mut self) -> TempId {
        let t = self.next_temp;
        self.next_temp += 1;
        t
    }

    fn push_stmt(&mut self, stmt: Statement) {
        self.blocks[self.cur_block as usize].stmts.push(stmt);
    }

    fn set_term(&mut self, term: Terminator) {
        self.blocks[self.cur_block as usize].term = term;
    }

    fn new_block(&mut self) -> BlockId {
        let id = self.blocks.len() as BlockId;
        self.blocks.push(BasicBlock { stmts: Vec::new(), term: Terminator::Return });
        id
    }

    fn lower_expr(&mut self, e: &Expr) -> Operand {
        use Expr::*;
        match e {
            Int { value, .. } => Operand::Const(Constant::Int(*value)),
            Float { value, .. } => Operand::Const(Constant::Float(*value)),
            Bool { value, .. } => Operand::Const(Constant::Bool(*value)),
            Str { value, .. } => Operand::Const(Constant::Str(value.clone())),
            Unit { .. } => Operand::Const(Constant::Unit),
            Builtin { .. } => Operand::Const(Constant::Unit),
            Ident { id, .. } => Operand::Var(*id),
            Binary { op, lhs, rhs, .. } => {
                let l = self.lower_expr(lhs);
                let r = self.lower_expr(rhs);
                let t = self.fresh_temp();
                self.push_stmt(Statement::StorageLive(t));
                self.push_stmt(Statement::Assign { dst: t, rv: Rvalue::BinaryOp { op: *op, lhs: l, rhs: r } });
                Operand::Temp(t)
            }
            Unary { op, rhs, .. } => {
                let src = self.lower_expr(rhs);
                let t = self.fresh_temp();
                self.push_stmt(Statement::StorageLive(t));
                self.push_stmt(Statement::Assign { dst: t, rv: Rvalue::UnaryOp { op: *op, src } });
                Operand::Temp(t)
            }
            Call { callee, args, .. } => {
                let name = match &**callee {
                    Ident { name, .. } => name.clone(),
                    Builtin { kind: hir::Builtin::Print, .. } => "print".to_string(),
                    _ => "<fn>".to_string(),
                };
                let mut a = Vec::new();
                for arg in args {
                    a.push(self.lower_expr(arg));
                }
                let t = self.fresh_temp();
                self.push_stmt(Statement::StorageLive(t));
                self.push_stmt(Statement::Assign { dst: t, rv: Rvalue::Call { fn_name: name, args: a } });
                Operand::Temp(t)
            }
        }
    }

    fn lower_stmt(&mut self, s: &Stmt) {
        use Stmt::*;
        match s {
            Let(l) => {
                let op = self.lower_expr(&l.init);
                self.push_stmt(Statement::Assign { dst: l.id, rv: Rvalue::Use(op) });
            }
            Assign { id, expr, .. } => {
                let op = self.lower_expr(expr);
                self.push_stmt(Statement::Assign { dst: *id, rv: Rvalue::Use(op) });
            }
            Expr(e) | Semi(e) => {
                self.lower_expr(e);
            }
            Return(opt) => {
                if let Some(e) = opt {
                    let op = self.lower_expr(e);
                    self.push_stmt(Statement::Assign { dst: RET_TEMP, rv: Rvalue::Use(op) });
                }
                self.set_term(Terminator::Return);
            }
        }
    }

    fn lower_block(&mut self, block: &hir::Block) {
        for stmt in &block.stmts {
            self.lower_stmt(stmt);
            if matches!(self.blocks[self.cur_block as usize].term, Terminator::Return) {
                break;
            }
        }
    }
}

pub fn lower_fn(hir_fn: &hir::HirFn) -> MirBody {
    let mut cx = LowerCtx::new(&hir_fn.body);
    cx.lower_block(&hir_fn.body);
    MirBody {
        blocks: cx.blocks,
        ret_ty: MirType::from(&hir_fn.return_ty),
    }
}

impl From<&crate::type_::Type> for MirType {
    fn from(t: &crate::type_::Type) -> Self {
        use crate::type_::Type::*;
        match t {
            Int => MirType::Int,
            Float => MirType::Float,
            Bool => MirType::Bool,
            Str => MirType::Str,
            Unit => MirType::Unit,
            Custom(_) | Ref { .. } => MirType::Unit,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parser::Parser, resolver::resolve};

    #[test]
    fn simple_add() {
        let src = r#"fn add(a: Int, b: Int) -> Int { return a + b; }"#;
        let module = Parser::new(src).parse_module();
        let (hir_mod, errs) = resolve(&module);
        assert!(errs.is_empty());
        if let hir::Item::Fn(f) = &hir_mod.items[0] {
            let mir = lower_fn(f);
            assert_eq!(mir.blocks.len(), 1);
            assert!(matches!(mir.blocks[0].term, Terminator::Return));
            let assigns = mir.blocks[0].stmts.iter().filter(|s| matches!(s, Statement::Assign { rv: Rvalue::BinaryOp { .. }, .. })).count();
            assert_eq!(assigns, 1);
        } else {
            panic!("expected function");
        }
    }
}


use std::collections::HashMap;
use std::rc::Rc;

use inkwell::{
    AddressSpace,
    builder::Builder,
    context::Context,
    module::Module,
    types::{BasicType, BasicTypeEnum},
    values::{BasicValueEnum, FunctionValue},
};

use crate::hir::BinOp;
use crate::mir::{
    BasicBlock, Constant, MirBody, MirType, Operand, RET_TEMP, Rvalue, Statement, TempId,
    Terminator,
};

// Safe approach: Use Rc to share ownership of the context
pub struct LlvmCtx<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
}

pub struct LlvmContext {
    context: Context,
}

impl LlvmCtx<'static> {
    /// Create a new LLVM context, module and builder with built-in functions.
    pub fn new(name: &str) -> Self {
        let context = Box::leak(Box::new(Context::create()));
        let module = context.create_module(name);
        let builder = context.create_builder();

        let i32_ty = context.i32_type();
        let i8_ptr = context.i8_type().ptr_type(AddressSpace::default());

        module.add_function(
            "aethc_print_int",
            context.void_type().fn_type(&[i32_ty.into()], false),
            None,
        );

        module.add_function(
            "aethc_print_str",
            context.void_type().fn_type(&[i8_ptr.into()], false),
            None,
        );

        Self {
            context,
            module,
            builder,
        }
    }
}

impl LlvmContext {
    pub fn new() -> Self {
        Self {
            context: Context::create(),
        }
    }

    pub fn create_llvm_ctx<'ctx>(&'ctx self, name: &str) -> LlvmCtx<'ctx> {
        let module = self.context.create_module(name);
        let builder = self.context.create_builder();

        let i32_ty = self.context.i32_type();
        let i8_ptr = self.context.i8_type().ptr_type(AddressSpace::default());

        module.add_function(
            "aethc_print_int",
            self.context.void_type().fn_type(&[i32_ty.into()], false),
            None,
        );

        module.add_function(
            "aethc_print_str",
            self.context.void_type().fn_type(&[i8_ptr.into()], false),
            None,
        );

        LlvmCtx {
            context: &self.context,
            module,
            builder,
        }
    }
}

impl<'ctx> LlvmCtx<'ctx> {
    fn ll_ty(&self, ty: &MirType) -> BasicTypeEnum<'ctx> {
        match ty {
            MirType::Int => self.context.i32_type().into(),
            MirType::Float => self.context.f64_type().into(),
            MirType::Bool => self.context.bool_type().into(),
            MirType::Str => self
                .context
                .i8_type()
                .ptr_type(AddressSpace::default())
                .into(),
            MirType::Unit => unreachable!("unit type has no LLVM equivalent"),
        }
    }
}

pub fn codegen_fn<'ctx>(llcx: &mut LlvmCtx<'ctx>, name: &str, mir: &MirBody) {
    let func = match mir.ret_ty {
        MirType::Unit => {
            llcx.module
                .add_function(name, llcx.context.void_type().fn_type(&[], false), None)
        }
        _ => {
            let ret_ty = llcx.ll_ty(&mir.ret_ty);
            llcx.module
                .add_function(name, ret_ty.fn_type(&[], false), None)
        }
    };

    let entry_bb = llcx.context.append_basic_block(func, "bb0");
    llcx.builder.position_at_end(entry_bb);

    let mut temps: HashMap<TempId, BasicValueEnum<'ctx>> = HashMap::new();

    lower_block(
        llcx,
        &mir.blocks[0],
        func,
        &mut temps,
        &mir.blocks,
        &mir.ret_ty,
    );
}

fn get_or_create_bb<'ctx>(
    llcx: &mut LlvmCtx<'ctx>,
    func: FunctionValue<'ctx>,
    id: u32,
) -> inkwell::basic_block::BasicBlock<'ctx> {
    for bb in func.get_basic_blocks() {
        if bb
            .get_name()
            .to_str()
            .map(|s| s == format!("bb{}", id))
            .unwrap_or(false)
        {
            return bb;
        }
    }
    llcx.context.append_basic_block(func, &format!("bb{}", id))
}

fn succ_blocks(term: &Terminator) -> Vec<u32> {
    match term {
        Terminator::Goto(id) => vec![*id],
        Terminator::CondBranch {
            then_bb, else_bb, ..
        } => vec![*then_bb, *else_bb],
        _ => Vec::new(),
    }
}

fn lower_block<'ctx>(
    llcx: &mut LlvmCtx<'ctx>,
    bb: &BasicBlock,
    func: FunctionValue<'ctx>,
    temps: &mut HashMap<TempId, BasicValueEnum<'ctx>>,
    all: &[BasicBlock],
    ret_ty: &MirType,
) {
    for stmt in &bb.stmts {
        match stmt {
            Statement::Assign { dst, rv } => {
                let val = lower_rvalue(llcx, rv, temps);
                temps.insert(*dst, val);
            }
            _ => {}
        }
    }

    match &bb.term {
        Terminator::Return => {
            // Fix: Use matches! macro or implement PartialEq for MirType
            if matches!(ret_ty, MirType::Unit) {
                let _ = llcx.builder.build_return(None);
            } else {
                let ret_val = temps.get(&RET_TEMP).expect("ret temp");
                let _ = llcx.builder.build_return(Some(ret_val));
            }
        }
        Terminator::Goto(id) => {
            let l_bb = get_or_create_bb(llcx, func, *id);
            let _ = llcx.builder.build_unconditional_branch(l_bb);
        }
        Terminator::CondBranch {
            cond,
            then_bb,
            else_bb,
        } => {
            let cond_val = lower_operand(llcx, cond, temps).into_int_value();
            let then_ll = get_or_create_bb(llcx, func, *then_bb);
            let else_ll = get_or_create_bb(llcx, func, *else_bb);
            let _ = llcx
                .builder
                .build_conditional_branch(cond_val, then_ll, else_ll);
        }
    }

    for succ in succ_blocks(&bb.term) {
        if !func.get_basic_blocks().iter().any(|b| {
            b.get_name()
                .to_str()
                .map(|s| s == format!("bb{}", succ))
                .unwrap_or(false)
        }) {
            let new_bb = llcx
                .context
                .append_basic_block(func, &format!("bb{}", succ));
            llcx.builder.position_at_end(new_bb);
            lower_block(llcx, &all[succ as usize], func, temps, all, ret_ty);
        }
    }
}

fn lower_operand<'ctx>(
    llcx: &LlvmCtx<'ctx>,
    op: &Operand,
    temps: &HashMap<TempId, BasicValueEnum<'ctx>>,
) -> BasicValueEnum<'ctx> {
    match op {
        Operand::Const(c) => match c {
            Constant::Int(i) => llcx.context.i32_type().const_int(*i as u64, true).into(),
            Constant::Float(f) => llcx.context.f64_type().const_float(*f).into(),
            Constant::Bool(b) => llcx.context.bool_type().const_int(*b as u64, false).into(),
            Constant::Str(s) => {
                let gv = llcx
                    .builder
                    .build_global_string_ptr(&format!("{}\0", s), "strlit")
                    .expect("Failed to build global string ptr");
                gv.as_pointer_value().into()
            }
            Constant::Unit => panic!("unit is never a value"),
        },
        Operand::Temp(t) => *temps.get(t).expect("temp"),
        Operand::Var(v) => {
            let ptr = *temps.get(v).expect("var ptr");
            let ptr_val = ptr.into_pointer_value();

            // For LLVM 16+, we need to specify the type explicitly
            // You'll need to determine the correct type based on your type system
            let elem_ty = llcx.context.i32_type(); // Adjust this based on your actual type

            llcx.builder
                .build_load(elem_ty, ptr_val, "varload")
                .expect("Failed to build load")
        }
    }
}

fn lower_rvalue<'ctx>(
    llcx: &mut LlvmCtx<'ctx>,
    rv: &Rvalue,
    temps: &HashMap<TempId, BasicValueEnum<'ctx>>,
) -> BasicValueEnum<'ctx> {
    match rv {
        Rvalue::Use(op) => lower_operand(llcx, op, temps),
        Rvalue::BinaryOp { op, lhs, rhs } => {
            let l = lower_operand(llcx, lhs, temps);
            let r = lower_operand(llcx, rhs, temps);
            match op {
                BinOp::Plus => {
                    if l.is_float_value() || r.is_float_value() {
                        let l_val = if l.is_int_value() {
                            llcx.builder
                                .build_signed_int_to_float(
                                    l.into_int_value(),
                                    llcx.context.f64_type(),
                                    "sitofp",
                                )
                                .expect("Failed to build sitofp")
                        } else {
                            l.into_float_value()
                        };
                        let r_val = if r.is_int_value() {
                            llcx.builder
                                .build_signed_int_to_float(
                                    r.into_int_value(),
                                    llcx.context.f64_type(),
                                    "sitofp",
                                )
                                .expect("Failed to build sitofp")
                        } else {
                            r.into_float_value()
                        };

                        llcx.builder
                            .build_float_add(l_val, r_val, "faddtmp")
                            .expect("Failed to build float add")
                            .into()
                    } else {
                        llcx.builder
                            .build_int_add(l.into_int_value(), r.into_int_value(), "iaddtmp")
                            .expect("Failed to build int add")
                            .into()
                    }
                }
                _ => todo!("other binops"),
            }
        }
        Rvalue::Call { fn_name, args } => {
            if fn_name == "print" {
                let arg_val = lower_operand(llcx, &args[0], temps);
                if arg_val.is_int_value() {
                    let f = llcx.module.get_function("aethc_print_int").unwrap();
                    let _ = llcx.builder.build_call(f, &[arg_val.into()], "");
                } else {
                    let f = llcx.module.get_function("aethc_print_str").unwrap();
                    let _ = llcx.builder.build_call(f, &[arg_val.into()], "");
                }
                llcx.context.i32_type().const_int(0, false).into()
            } else {
                todo!("non builtin call")
            }
        }
        _ => todo!("unary"),
    }
}

pub fn write_ir<'ctx>(llcx: &LlvmCtx<'ctx>, path: &str) {
    llcx.module.print_to_file(path).unwrap();
}

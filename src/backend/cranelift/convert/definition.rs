use std::collections::HashMap;

use cranelift_codegen::ir::{
    types, AbiParam, Function, InstBuilder, Signature, UserFuncName, Value,
};
use cranelift_codegen::isa::CallConv;
use cranelift_codegen::Context as ClifContext;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_module::{FuncId, Linkage, Module};
use cranelift_object::ObjectModule;

use crate::backend::cranelift::convert::common::fun_name;
use crate::backend::cranelift::convert::{FnLower, Funcs};
use crate::backend::cranelift::primitive::{cranelift_type, cranelift_type_of_name};
use crate::backend::cranelift::result::{BackendErr, BackendResult};
use crate::check::ast::{ASTTy, NodeTy};
use crate::check::name::Name;
use crate::common::position::Position;
use crate::parse::ast::node_op::NodeOp;

/// A `FunDef`'s Cranelift signature, built from its declared argument types and return type.
pub(super) fn fun_signature(
    args: &[ASTTy],
    ret: Option<&Name>,
    call_conv: CallConv,
    pos_ast: &ASTTy,
) -> BackendResult<Signature> {
    let mut params = vec![];
    for arg in args {
        params.push(AbiParam::new(arg_type(arg)?));
    }
    let returns = match ret {
        Some(ret) => vec![AbiParam::new(cranelift_type_of_name(ret, pos_ast.pos)?)],
        None => vec![],
    };
    Ok(Signature {
        params,
        returns,
        call_conv,
    })
}

/// The Cranelift type of a `FunArg`'s declared parameter type -- which lives in the `FunArg`
/// variant's own `ty` field, not in the surrounding node's resolved type (see
/// `name::cranelift_type_of_name`'s doc comment).
pub(super) fn arg_type(arg: &ASTTy) -> BackendResult<cranelift_codegen::ir::Type> {
    match &arg.node {
        NodeTy::FunArg { ty: Some(ty), .. } => cranelift_type_of_name(ty, arg.pos),
        NodeTy::FunArg { ty: None, .. } => Err(BackendErr::new(
            arg.pos,
            "Function argument must have a type",
        )),
        other => Err(BackendErr::unimplemented(
            arg,
            &format!("{other:?} argument"),
        )),
    }
}

/// Declare the two external `libc` functions `print` may need. Declared lazily -- and re-declared
/// per function via `Module::declare_function`, which is idempotent (merges with the existing
/// declaration of the same name) -- rather than threading a single shared declaration through.
pub(super) fn declare_libc(
    module: &mut ObjectModule,
    call_conv: CallConv,
) -> BackendResult<(FuncId, FuncId)> {
    let pointer_type = module.isa().pointer_type();

    let puts_sig = Signature {
        params: vec![AbiParam::new(pointer_type)],
        returns: vec![AbiParam::new(types::I32)],
        call_conv,
    };
    let puts_id = module
        .declare_function("puts", Linkage::Import, &puts_sig)
        .map_err(|e| BackendErr::new(Position::invisible(), &e.to_string()))?;

    let printf_sig = Signature {
        params: vec![AbiParam::new(pointer_type), AbiParam::new(types::I64)],
        returns: vec![AbiParam::new(types::I32)],
        call_conv,
    };
    let printf_id = module
        .declare_function("printf", Linkage::Import, &printf_sig)
        .map_err(|e| BackendErr::new(Position::invisible(), &e.to_string()))?;

    Ok((puts_id, printf_id))
}

/// Define a top-level `FunDef`'s body as a real Cranelift function.
///
/// Returns the function's disassembly text when `want_asm` is set -- `None` otherwise, so callers
/// that don't need it (e.g. [`super::compile`]) don't pay for [`ClifContext::set_disasm`].
pub(super) fn define_function(
    module: &mut ObjectModule,
    func_id: FuncId,
    sig: Signature,
    args: &[ASTTy],
    body: Option<&ASTTy>,
    funcs: &Funcs,
    want_asm: bool,
) -> BackendResult<Option<String>> {
    let mut ctx = ClifContext::new();
    ctx.set_disasm(want_asm);
    ctx.func = Function::with_name_signature(UserFuncName::user(0, func_id.as_u32()), sig);
    let mut fb_ctx = FunctionBuilderContext::new();

    {
        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut fb_ctx);
        let entry = builder.create_block();
        builder.append_block_params_for_function_params(entry);
        builder.switch_to_block(entry);

        let (puts_id, printf_id) = declare_libc(module, builder.func.signature.call_conv)?;
        let mut lower = FnLower {
            builder,
            module,
            vars: HashMap::new(),
            var_seq: 0,
            funcs,
            puts_id,
            printf_id,
        };

        let block_params = lower.builder.block_params(entry).to_vec();
        for (arg, value) in args.iter().zip(block_params) {
            lower.bind_arg(arg, value)?;
        }

        match body {
            Some(body) => lower.lower_tail(body)?,
            None => {
                lower.builder.ins().return_(&[]);
            }
        }

        lower.builder.seal_all_blocks();
        lower.builder.finalize();
    }

    module
        .define_function(func_id, &mut ctx)
        .map_err(|e| BackendErr::new(Position::invisible(), &e.to_string()))?;
    Ok(want_asm.then(|| disasm_text(&ctx)))
}

/// Define the synthetic `main` collecting every top-level statement that isn't a `FunDef`.
///
/// Returns its disassembly text when `want_asm` is set -- see [`define_function`]'s doc comment.
pub(super) fn define_main(
    module: &mut ObjectModule,
    func_id: FuncId,
    sig: Signature,
    statements: &[ASTTy],
    funcs: &Funcs,
    want_asm: bool,
) -> BackendResult<Option<String>> {
    let mut ctx = ClifContext::new();
    ctx.set_disasm(want_asm);
    ctx.func = Function::with_name_signature(UserFuncName::user(0, func_id.as_u32()), sig);
    let mut fb_ctx = FunctionBuilderContext::new();

    {
        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut fb_ctx);
        let entry = builder.create_block();
        builder.switch_to_block(entry);

        let (puts_id, printf_id) = declare_libc(module, builder.func.signature.call_conv)?;
        let mut lower = FnLower {
            builder,
            module,
            vars: HashMap::new(),
            var_seq: 0,
            funcs,
            puts_id,
            printf_id,
        };

        for statement in statements {
            lower.lower_stmt(statement)?;
        }
        let zero = lower.builder.ins().iconst(types::I32, 0);
        lower.builder.ins().return_(&[zero]);

        lower.builder.seal_all_blocks();
        lower.builder.finalize();
    }

    module
        .define_function(func_id, &mut ctx)
        .map_err(|e| BackendErr::new(Position::invisible(), &e.to_string()))?;
    Ok(want_asm.then(|| disasm_text(&ctx)))
}

/// The disassembly text Cranelift collected while compiling `ctx.func`, if any -- empty when the
/// backend couldn't produce one, which the `vcode` field's doc comment says can happen even with
/// [`ClifContext::set_disasm`] on. Only call this when disasm was actually requested.
fn disasm_text(ctx: &ClifContext) -> String {
    ctx.compiled_code()
        .and_then(|compiled| compiled.vcode.clone())
        .unwrap_or_default()
}

impl<'a> FnLower<'a> {
    /// Lower a `VariableDef` with an initializer, binding a new Cranelift variable.
    pub(super) fn lower_variable_def(&mut self, ast: &ASTTy) -> BackendResult<()> {
        match &ast.node {
            NodeTy::VariableDef {
                var,
                expr: Some(expr),
                ..
            } => {
                let name = fun_name(var)?;
                let ty = cranelift_type(expr)?;
                let value = self.lower_expr(expr)?;
                let var = self.new_var(ty);
                self.builder.def_var(var, value);
                self.vars.insert(name, (var, ty));
                Ok(())
            }
            other => Err(BackendErr::unimplemented(
                ast,
                &format!("{other:?} variable definition"),
            )),
        }
    }

    /// Lower a plain (`:=`) reassignment to an already-declared variable. Compound assignment
    /// (`+=` and friends) is out of scope -- write `x := x + y` instead of `x += y`.
    pub(super) fn lower_reassign(&mut self, ast: &ASTTy) -> BackendResult<()> {
        match &ast.node {
            NodeTy::Reassign {
                left,
                right,
                op: NodeOp::Assign,
            } => {
                let name = fun_name(left)?;
                let (var, _) = *self.vars.get(&name).ok_or_else(|| {
                    BackendErr::new(ast.pos, &format!("Undefined variable '{name}'"))
                })?;
                let value = self.lower_expr(right)?;
                self.builder.def_var(var, value);
                Ok(())
            }
            NodeTy::Reassign { op, .. } => Err(BackendErr::unimplemented(
                ast,
                &format!("{op:?} compound reassignment"),
            )),
            other => Err(BackendErr::unimplemented(
                ast,
                &format!("{other:?} reassignment"),
            )),
        }
    }

    pub(super) fn bind_arg(&mut self, arg: &ASTTy, value: Value) -> BackendResult<()> {
        let name = match &arg.node {
            NodeTy::FunArg { var, .. } => fun_name(var)?,
            _ => return Err(BackendErr::unimplemented(arg, "non-identifier argument")),
        };
        let ty = arg_type(arg)?;
        let var = self.new_var(ty);
        self.builder.def_var(var, value);
        self.vars.insert(name, (var, ty));
        Ok(())
    }

    pub(super) fn new_var(&mut self, ty: cranelift_codegen::ir::Type) -> Variable {
        let var = Variable::from_u32(self.var_seq);
        self.var_seq += 1;
        self.builder.declare_var(var, ty);
        var
    }
}

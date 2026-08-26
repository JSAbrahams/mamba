use std::collections::HashMap;

use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::{
    types, AbiParam, Function, InstBuilder, Signature, UserFuncName, Value,
};
use cranelift_codegen::isa::CallConv;
use cranelift_codegen::Context as ClifContext;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_module::{DataDescription, FuncId, Linkage, Module};
use cranelift_object::ObjectModule;

use crate::backend::cranelift::primitive::{cranelift_type, cranelift_type_of_name};
use crate::backend::cranelift::result::{BackendErr, BackendResult};
use crate::check::ast::{ASTTy, NodeTy};
use crate::check::context::function::PRINT;
use crate::check::name::Name;
use crate::Context;

/// Declared user functions, keyed by their Mamba name -- shared across every function body so
/// forward references (function `a` calling function `b` defined later in the same file) work.
type Funcs = HashMap<String, FuncId>;

/// Lower an entire checked file (a top-level `NodeTy::Block`) into `module`.
///
/// Top-level `FunDef`s become real Cranelift functions. Every other top-level statement is
/// collected into a synthetic `main`, mirroring how a `.mamba` file's top-level statements run
/// top-to-bottom as a script in the Python backend -- machine code needs an explicit entry point,
/// which Python's linear script execution doesn't.
pub fn lower_program(
    ast_ty: &ASTTy,
    _ctx: &Context,
    module: &mut ObjectModule,
) -> BackendResult<()> {
    let statements = match &ast_ty.node {
        NodeTy::Block { statements } => statements,
        _ => std::slice::from_ref(ast_ty),
    };

    let call_conv = module.isa().default_call_conv();

    // Pass 1: declare every top-level function's signature, so calls to a function defined later
    // in the file still resolve.
    let mut funcs = Funcs::new();
    for statement in statements {
        if let NodeTy::FunDef { id, args, ret, .. } = &statement.node {
            let name = fun_name(id)?;
            let sig = fun_signature(args, ret.as_ref(), call_conv, statement)?;
            let func_id = module
                .declare_function(&name, Linkage::Export, &sig)
                .map_err(|e| BackendErr::new(statement.pos, &e.to_string()))?;
            funcs.insert(name, func_id);
        }
    }

    // Pass 2: define each function's body, plus a synthetic `main` for everything else.
    let mut main_body = vec![];
    for statement in statements {
        match &statement.node {
            NodeTy::FunDef {
                id,
                args,
                ret,
                body,
                ..
            } => {
                let name = fun_name(id)?;
                let func_id = *funcs.get(&name).expect("declared in pass 1");
                let sig = fun_signature(args, ret.as_ref(), call_conv, statement)?;
                define_function(module, func_id, sig, args, body.as_deref(), &funcs)?;
            }
            _ => main_body.push(statement.clone()),
        }
    }

    let main_sig = Signature {
        params: vec![],
        returns: vec![AbiParam::new(types::I32)],
        call_conv,
    };
    let main_id = module
        .declare_function("main", Linkage::Export, &main_sig)
        .map_err(|e| BackendErr::new(ast_ty.pos, &e.to_string()))?;
    define_main(module, main_id, main_sig, &main_body, &funcs)?;

    Ok(())
}

fn fun_name(id: &ASTTy) -> BackendResult<String> {
    match &id.node {
        NodeTy::Id { lit } => Ok(lit.clone()),
        other => Err(BackendErr::unimplemented(
            id,
            &format!("{other:?} function name"),
        )),
    }
}

fn fun_signature(
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
/// `types::cranelift_type_of_name`'s doc comment).
fn arg_type(arg: &ASTTy) -> BackendResult<cranelift_codegen::ir::Type> {
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
fn declare_libc(module: &mut ObjectModule, call_conv: CallConv) -> BackendResult<(FuncId, FuncId)> {
    let pointer_type = module.isa().pointer_type();

    let puts_sig = Signature {
        params: vec![AbiParam::new(pointer_type)],
        returns: vec![AbiParam::new(types::I32)],
        call_conv,
    };
    let puts_id = module
        .declare_function("puts", Linkage::Import, &puts_sig)
        .map_err(|e| {
            BackendErr::new(
                crate::common::position::Position::invisible(),
                &e.to_string(),
            )
        })?;

    let printf_sig = Signature {
        params: vec![AbiParam::new(pointer_type), AbiParam::new(types::I64)],
        returns: vec![AbiParam::new(types::I32)],
        call_conv,
    };
    let printf_id = module
        .declare_function("printf", Linkage::Import, &printf_sig)
        .map_err(|e| {
            BackendErr::new(
                crate::common::position::Position::invisible(),
                &e.to_string(),
            )
        })?;

    Ok((puts_id, printf_id))
}

fn define_function(
    module: &mut ObjectModule,
    func_id: FuncId,
    sig: Signature,
    args: &[ASTTy],
    body: Option<&ASTTy>,
    funcs: &Funcs,
) -> BackendResult<()> {
    let mut ctx = ClifContext::new();
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

        let block_params: Vec<Value> = lower.builder.block_params(entry).to_vec();
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

    module.define_function(func_id, &mut ctx).map_err(|e| {
        BackendErr::new(
            crate::common::position::Position::invisible(),
            &e.to_string(),
        )
    })
}

fn define_main(
    module: &mut ObjectModule,
    func_id: FuncId,
    sig: Signature,
    statements: &[ASTTy],
    funcs: &Funcs,
) -> BackendResult<()> {
    let mut ctx = ClifContext::new();
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

    module.define_function(func_id, &mut ctx).map_err(|e| {
        BackendErr::new(
            crate::common::position::Position::invisible(),
            &e.to_string(),
        )
    })
}

/// Per-function-body lowering state.
struct FnLower<'a> {
    builder: FunctionBuilder<'a>,
    module: &'a mut ObjectModule,
    vars: HashMap<String, (Variable, cranelift_codegen::ir::Type)>,
    var_seq: u32,
    funcs: &'a Funcs,
    puts_id: FuncId,
    printf_id: FuncId,
}

impl<'a> FnLower<'a> {
    fn new_var(&mut self, ty: cranelift_codegen::ir::Type) -> Variable {
        let var = Variable::from_u32(self.var_seq);
        self.var_seq += 1;
        self.builder.declare_var(var, ty);
        var
    }

    fn bind_arg(&mut self, arg: &ASTTy, value: Value) -> BackendResult<()> {
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

    /// Lower `ast` as a statement: for side effects only, its value (if any) is discarded.
    fn lower_stmt(&mut self, ast: &ASTTy) -> BackendResult<()> {
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
            NodeTy::IfElse { cond, then, el } => {
                let cond_value = self.lower_expr(cond)?;
                let then_block = self.builder.create_block();
                let merge_block = self.builder.create_block();
                let else_block = if el.is_some() {
                    self.builder.create_block()
                } else {
                    merge_block
                };

                self.builder
                    .ins()
                    .brif(cond_value, then_block, &[], else_block, &[]);

                self.builder.switch_to_block(then_block);
                self.lower_stmt(then)?;
                self.builder.ins().jump(merge_block, &[]);

                if let Some(el) = el {
                    self.builder.switch_to_block(else_block);
                    self.lower_stmt(el)?;
                    self.builder.ins().jump(merge_block, &[]);
                }

                self.builder.switch_to_block(merge_block);
                Ok(())
            }
            NodeTy::Block { statements } => {
                for statement in statements {
                    self.lower_stmt(statement)?;
                }
                Ok(())
            }
            NodeTy::FunctionCall { name, .. } if name.name == PRINT => {
                self.lower_print(ast).map(|_| ())
            }
            NodeTy::FunctionCall { .. } => self.lower_expr(ast).map(|_| ()),
            other => Err(BackendErr::unimplemented(
                ast,
                &format!("{other:?} statement"),
            )),
        }
    }

    /// Lower `ast` as the tail of a function body: it must end the current block with a `return`
    /// (possibly by recursing into `Block`'s last statement, or into each arm of an `IfElse`).
    fn lower_tail(&mut self, ast: &ASTTy) -> BackendResult<()> {
        match &ast.node {
            NodeTy::Return { expr } => {
                let value = self.lower_expr(expr)?;
                self.builder.ins().return_(&[value]);
                Ok(())
            }
            NodeTy::ReturnEmpty => {
                self.builder.ins().return_(&[]);
                Ok(())
            }
            NodeTy::Block { statements } => match statements.split_last() {
                Some((last, init)) => {
                    for statement in init {
                        self.lower_stmt(statement)?;
                    }
                    self.lower_tail(last)
                }
                None => {
                    self.builder.ins().return_(&[]);
                    Ok(())
                }
            },
            NodeTy::IfElse { cond, then, el } => {
                let cond_value = self.lower_expr(cond)?;
                let then_block = self.builder.create_block();
                let else_block = self.builder.create_block();

                self.builder
                    .ins()
                    .brif(cond_value, then_block, &[], else_block, &[]);

                self.builder.switch_to_block(then_block);
                self.lower_tail(then)?;

                self.builder.switch_to_block(else_block);
                match el {
                    Some(el) => self.lower_tail(el)?,
                    None => {
                        self.builder.ins().return_(&[]);
                    }
                }
                Ok(())
            }
            _ => {
                let value = self.lower_expr(ast)?;
                self.builder.ins().return_(&[value]);
                Ok(())
            }
        }
    }

    /// Lower `ast` as a value-producing expression.
    fn lower_expr(&mut self, ast: &ASTTy) -> BackendResult<Value> {
        match &ast.node {
            // Int/Bool literals' own resolved `ty` can come back widened to a union (e.g. a
            // literal argument to `print`, whose parameter accepts several printable types
            // unifies to that broader union rather than staying just `Int`) -- but the node
            // variant itself already tells us the literal's true type, so there's no need to
            // consult `ast.ty` at all here.
            NodeTy::Int { lit } => {
                let value: i64 = lit.parse().map_err(|_| {
                    BackendErr::new(ast.pos, &format!("Invalid int literal '{lit}'"))
                })?;
                Ok(self.builder.ins().iconst(types::I64, value))
            }
            NodeTy::Bool { lit } => Ok(self.builder.ins().iconst(types::I8, i64::from(*lit))),
            NodeTy::Id { lit } => {
                let (var, _) = self.vars.get(lit).ok_or_else(|| {
                    BackendErr::new(ast.pos, &format!("Undefined variable '{lit}'"))
                })?;
                Ok(self.builder.use_var(*var))
            }
            NodeTy::Add { left, right } => {
                self.lower_arith(ast, left, right, |b, a, c| b.ins().iadd(a, c))
            }
            NodeTy::Sub { left, right } => {
                self.lower_arith(ast, left, right, |b, a, c| b.ins().isub(a, c))
            }
            NodeTy::Mul { left, right } => {
                self.lower_arith(ast, left, right, |b, a, c| b.ins().imul(a, c))
            }
            NodeTy::Div { left, right } => {
                self.lower_arith(ast, left, right, |b, a, c| b.ins().sdiv(a, c))
            }
            NodeTy::Le { left, right } => self.lower_cmp(left, right, IntCC::SignedLessThan),
            NodeTy::Leq { left, right } => {
                self.lower_cmp(left, right, IntCC::SignedLessThanOrEqual)
            }
            NodeTy::Ge { left, right } => self.lower_cmp(left, right, IntCC::SignedGreaterThan),
            NodeTy::Geq { left, right } => {
                self.lower_cmp(left, right, IntCC::SignedGreaterThanOrEqual)
            }
            NodeTy::Eq { left, right } => self.lower_cmp(left, right, IntCC::Equal),
            NodeTy::Neq { left, right } => self.lower_cmp(left, right, IntCC::NotEqual),
            NodeTy::FunctionCall { name, args } => {
                let func_id = *self.funcs.get(&name.name).ok_or_else(|| {
                    BackendErr::new(ast.pos, &format!("Undefined function '{}'", name.name))
                })?;
                let local = self.module.declare_func_in_func(func_id, self.builder.func);
                let mut arg_values = vec![];
                for arg in args {
                    arg_values.push(self.lower_expr(arg)?);
                }
                let call = self.builder.ins().call(local, &arg_values);
                self.builder
                    .inst_results(call)
                    .first()
                    .copied()
                    .ok_or_else(|| {
                        BackendErr::new(
                            ast.pos,
                            &format!("'{}' does not return a value", name.name),
                        )
                    })
            }
            other => Err(BackendErr::unimplemented(
                ast,
                &format!("{other:?} expression"),
            )),
        }
    }

    fn lower_arith(
        &mut self,
        ast: &ASTTy,
        left: &ASTTy,
        right: &ASTTy,
        op: impl Fn(&mut FunctionBuilder, Value, Value) -> Value,
    ) -> BackendResult<Value> {
        cranelift_type(ast)?; // reject non-primitive-typed arithmetic early, with a clear error
        let l = self.lower_expr(left)?;
        let r = self.lower_expr(right)?;
        Ok(op(&mut self.builder, l, r))
    }

    fn lower_cmp(&mut self, left: &ASTTy, right: &ASTTy, cc: IntCC) -> BackendResult<Value> {
        let l = self.lower_expr(left)?;
        let r = self.lower_expr(right)?;
        Ok(self.builder.ins().icmp(cc, l, r))
    }

    /// `print(...)`: a string literal goes through `puts` (which appends its own newline, like
    /// Mamba/Python's `print`); a primitive value goes through `printf` with a fixed `%lld\n`
    /// format. Anything else (interpolated strings, non-primitive values, multiple arguments) is
    /// out of scope for this backend.
    fn lower_print(&mut self, ast: &ASTTy) -> BackendResult<Option<Value>> {
        let args = match &ast.node {
            NodeTy::FunctionCall { args, .. } => args,
            _ => unreachable!("only called for a FunctionCall node"),
        };
        let arg = match args.as_slice() {
            [arg] => arg,
            _ => return Err(BackendErr::unimplemented(ast, "print with != 1 argument")),
        };

        match &arg.node {
            NodeTy::Str { lit, expressions } if expressions.is_empty() => {
                let data = format!("{lit}\0").into_bytes().into_boxed_slice();
                let data_id = self
                    .module
                    .declare_anonymous_data(false, false)
                    .map_err(|e| BackendErr::new(ast.pos, &e.to_string()))?;
                let mut desc = DataDescription::new();
                desc.define(data);
                self.module
                    .define_data(data_id, &desc)
                    .map_err(|e| BackendErr::new(ast.pos, &e.to_string()))?;

                let gv = self.module.declare_data_in_func(data_id, self.builder.func);
                let pointer_type = self.module.isa().pointer_type();
                let ptr = self.builder.ins().global_value(pointer_type, gv);
                let callee = self
                    .module
                    .declare_func_in_func(self.puts_id, self.builder.func);
                self.builder.ins().call(callee, &[ptr]);
                Ok(None)
            }
            NodeTy::Str { .. } => Err(BackendErr::unimplemented(
                ast,
                "print of an interpolated string",
            )),
            _ => {
                let value = self.lower_expr(arg)?;
                let fmt = format!("{}\0", "%lld\n").into_bytes().into_boxed_slice();
                let data_id = self
                    .module
                    .declare_anonymous_data(false, false)
                    .map_err(|e| BackendErr::new(ast.pos, &e.to_string()))?;
                let mut desc = DataDescription::new();
                desc.define(fmt);
                self.module
                    .define_data(data_id, &desc)
                    .map_err(|e| BackendErr::new(ast.pos, &e.to_string()))?;

                let gv = self.module.declare_data_in_func(data_id, self.builder.func);
                let pointer_type = self.module.isa().pointer_type();
                let ptr = self.builder.ins().global_value(pointer_type, gv);
                // Widen a narrower-than-i64 value (e.g. Bool, stored as i8) to match `%lld`.
                let value = if self.builder.func.dfg.value_type(value) != types::I64 {
                    self.builder.ins().sextend(types::I64, value)
                } else {
                    value
                };
                let callee = self
                    .module
                    .declare_func_in_func(self.printf_id, self.builder.func);
                self.builder.ins().call(callee, &[ptr, value]);
                Ok(None)
            }
        }
    }
}

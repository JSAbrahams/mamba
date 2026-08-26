use std::collections::HashMap;

use cranelift_codegen::ir::{types, AbiParam, InstBuilder, Signature, Value};
use cranelift_frontend::{FunctionBuilder, Variable};
use cranelift_module::{FuncId, Linkage, Module};
use cranelift_object::ObjectModule;

use crate::backend::cranelift::convert::definition::{define_function, define_main, fun_signature};
use crate::backend::cranelift::result::{BackendErr, BackendResult};
use crate::check::ast::{ASTTy, NodeTy};
use crate::check::context::function::PRINT;
use crate::Context;

mod call;
mod common;
mod control_flow;
mod definition;
mod operation;

/// Declared user functions, keyed by their Mamba name -- shared across every function body so
/// forward references (function `a` calling function `b` defined later in the same file) work.
type Funcs = HashMap<String, FuncId>;

/// Lower an entire checked file (a top-level `NodeTy::Block`) into `module`.
///
/// Top-level `FunDef`s become real Cranelift functions. Every other top-level statement is
/// collected into a synthetic `main`, mirroring how a `.mamba` file's top-level statements run
/// top-to-bottom as a script in the Python backend -- machine code needs an explicit entry point,
/// which Python's linear script execution doesn't.
pub(super) fn lower_program(
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
            let name = common::fun_name(id)?;
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
                let name = common::fun_name(id)?;
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
    /// Lower `ast` as a statement: for side effects only, its value (if any) is discarded.
    fn lower_stmt(&mut self, ast: &ASTTy) -> BackendResult<()> {
        match &ast.node {
            NodeTy::VariableDef { .. } => self.lower_variable_def(ast),
            NodeTy::IfElse { cond, then, el } => self.lower_if_else_stmt(cond, then, el.as_deref()),
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
            NodeTy::IfElse { cond, then, el } => self.lower_if_else_tail(cond, then, el.as_deref()),
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
            NodeTy::Add { .. }
            | NodeTy::Sub { .. }
            | NodeTy::Mul { .. }
            | NodeTy::Div { .. }
            | NodeTy::Le { .. }
            | NodeTy::Leq { .. }
            | NodeTy::Ge { .. }
            | NodeTy::Geq { .. }
            | NodeTy::Eq { .. }
            | NodeTy::Neq { .. } => self.lower_operation(ast),
            NodeTy::FunctionCall { .. } => self.lower_call(ast),
            other => Err(BackendErr::unimplemented(
                ast,
                &format!("{other:?} expression"),
            )),
        }
    }
}

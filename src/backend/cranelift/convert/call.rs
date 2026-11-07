use cranelift_codegen::ir::{types, InstBuilder, Value};
use cranelift_module::{DataDescription, Module};

use crate::backend::cranelift::convert::FnLower;
use crate::backend::cranelift::result::{BackendErr, BackendResult};
use crate::check::ast::{ASTTy, NodeTy};

impl<'a> FnLower<'a> {
    /// Lower a (non-`print`) `FunctionCall` to a user-defined function, in statement position:
    /// any return value is discarded, so a call to a function with no return type (`None` here)
    /// is perfectly fine -- unlike [`Self::lower_call_expr`], which needs one.
    pub(super) fn lower_call_stmt(&mut self, ast: &ASTTy) -> BackendResult<()> {
        if let Some(value) = self.lower_call(ast)? {
            self.drop_temp(value);
        }
        Ok(())
    }

    /// Lower a (non-`print`) `FunctionCall` to a user-defined function, in expression position:
    /// errors if the callee has no return type to produce a value with.
    pub(super) fn lower_call_expr(&mut self, ast: &ASTTy) -> BackendResult<Value> {
        self.lower_call(ast)?.ok_or_else(|| {
            let name = match &ast.node {
                NodeTy::FunctionCall { name, .. } => name.name.as_str(),
                _ => unreachable!("only called for a FunctionCall node"),
            };
            BackendErr::new(ast.pos, &format!("'{name}' does not return a value"))
        })
    }

    fn lower_call(&mut self, ast: &ASTTy) -> BackendResult<Option<Value>> {
        let (name, args) = match &ast.node {
            NodeTy::FunctionCall {
                name,
                args,
                is_index: false,
            } => (name, args),
            other => {
                return Err(BackendErr::unimplemented(
                    ast,
                    &format!("{other:?} function call"),
                ))
            }
        };

        let func_id = *self.funcs.get(&name.name).ok_or_else(|| {
            BackendErr::new(ast.pos, &format!("Undefined function '{}'", name.name))
        })?;
        let local = self.module.declare_func_in_func(func_id, self.builder.func);
        let mut arg_values = vec![];
        for arg in args {
            let value = self.lower_expr(arg)?;
            self.own(value);
            arg_values.push(value);
        }
        let call = self.builder.ins().call(local, &arg_values);
        let result = self.builder.inst_results(call).first().copied();
        if let Some(result) = result {
            if self.builder.func.dfg.value_type(result) == types::I64 {
                self.temps.insert(result);
            }
        }
        Ok(result)
    }

    /// `print(...)`: a string literal goes through `puts`, which appends its own newline, like Mamba's `print`.
    /// An `Int` is printed in decimal by the runtime, and so is a `Bool`, as the integer 0 or 1.
    /// Anything else (interpolated strings, a `Float` value, non-primitive values, multiple arguments) is out of scope for this backend.
    ///
    /// A `Float` is deliberately rejected rather than attempted: formatting one correctly.
    /// E.g. shortest round-tripping decimal output, the way Python's own `print` does, is a meaningfully harder problem than an integer.
    pub(super) fn lower_print(&mut self, ast: &ASTTy) -> BackendResult<Option<Value>> {
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
                let word = match self.builder.func.dfg.value_type(value) {
                    types::F64 => {
                        return Err(BackendErr::unimplemented(ast, "print of a Float value"))
                    }
                    types::I64 => value,
                    _ => {
                        let wide = self.builder.ins().uextend(types::I64, value);
                        self.builder.ins().ishl_imm(wide, 1)
                    }
                };
                let callee = self
                    .module
                    .declare_func_in_func(self.runtime.print, self.builder.func);
                self.builder.ins().call(callee, &[word]);
                self.drop_temp(value);
                Ok(None)
            }
        }
    }
}

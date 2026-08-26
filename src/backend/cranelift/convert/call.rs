use cranelift_codegen::ir::{types, InstBuilder, Value};
use cranelift_module::{DataDescription, Module};

use crate::backend::cranelift::convert::FnLower;
use crate::backend::cranelift::result::{BackendErr, BackendResult};
use crate::check::ast::{ASTTy, NodeTy};

impl<'a> FnLower<'a> {
    /// Lower a (non-`print`) `FunctionCall` to a user-defined function.
    pub(super) fn lower_call(&mut self, ast: &ASTTy) -> BackendResult<Value> {
        let (name, args) = match &ast.node {
            NodeTy::FunctionCall { name, args } => (name, args),
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
            arg_values.push(self.lower_expr(arg)?);
        }
        let call = self.builder.ins().call(local, &arg_values);
        self.builder
            .inst_results(call)
            .first()
            .copied()
            .ok_or_else(|| {
                BackendErr::new(ast.pos, &format!("'{}' does not return a value", name.name))
            })
    }

    /// `print(...)`: a string literal goes through `puts` (which appends its own newline, like
    /// Mamba/Python's `print`); an `Int`/`Bool` value goes through `printf` with a fixed
    /// `%lld\n` format. Anything else (interpolated strings, a `Float` value, non-primitive
    /// values, multiple arguments) is out of scope for this backend.
    ///
    /// A `Float` is deliberately rejected rather than attempted: `%lld` would read raw float bits
    /// as an integer (garbage, not a crash), and doing this properly means both a `%f`-style
    /// format string and setting `%al` to the SysV-mandated vector-register count for a variadic
    /// call passing a float -- printf-only ABI plumbing this backend doesn't have yet.
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
                if self.builder.func.dfg.value_type(value) == types::F64 {
                    return Err(BackendErr::unimplemented(ast, "print of a Float value"));
                }
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

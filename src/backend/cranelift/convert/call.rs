use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::{types, InstBuilder, MemFlags, StackSlotData, StackSlotKind, Value};
use cranelift_module::{DataDescription, Module};

use crate::backend::cranelift::convert::FnLower;
use crate::backend::cranelift::result::{BackendErr, BackendResult};
use crate::check::ast::{ASTTy, NodeTy};

impl<'a> FnLower<'a> {
    /// Lower a (non-`print`) `FunctionCall` to a user-defined function, in statement position:
    /// any return value is discarded, so a call to a function with no return type (`None` here)
    /// is perfectly fine -- unlike [`Self::lower_call_expr`], which needs one.
    pub(super) fn lower_call_stmt(&mut self, ast: &ASTTy) -> BackendResult<()> {
        self.lower_call(ast).map(|_| ())
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
        Ok(self.builder.inst_results(call).first().copied())
    }

    /// `print(...)`: a string literal goes through `puts` (which appends its own newline, like Mamba/Python's `print`).
    /// An `Int`/`Bool` value is formatted to a decimal string ourselves (see [`Self::lower_print_int`]) and then also printed via `puts`.
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
                if self.builder.func.dfg.value_type(value) == types::F64 {
                    return Err(BackendErr::unimplemented(ast, "print of a Float value"));
                }
                self.lower_print_int(value)?;
                Ok(None)
            }
        }
    }

    /// Format an `Int`/`Bool` value (`value`, an `I64` or `I8`) to a NUL-terminated decimal ASCII string on the stack then print it via `puts`.
    /// See [`Self::lower_print`]'s doc comment for why this hand-rolled formatting exists instead of a `printf` call.
    ///
    /// Classic `itoa: write digits` into a stack buffer back-to-front (least-significant first, at the end, working backward).
    /// There, no separate reversal pass is needed.
    /// Then `puts` the resulting suffix of the buffer.
    /// `value`'s sign is handled by working with its absolute value throughout and writing a leading `-` afterwards if negative.
    ///
    /// The buffer is deliberately generous.
    /// 24 bytes: at most 19 digits for any `i64` magnitude, one sign byte, one NUL, with room to spare so indices never need bounds-checking.
    ///
    /// `i64::MIN`'s magnitude doesn't fit in a positive `i64` -- naively negating it would overflow.
    /// `ineg`'s raw two's-complement bit pattern is exactly the right *unsigned* magnitude even in that case.
    /// E.g. `i64::MIN`'s bit pattern negates right back to itself, which reinterpreted as `u64` is precisely `-i64::MIN`.
    /// So as long as everything from there on (`udiv_imm`/`urem_imm`) treats the value as unsigned rather than signed the `i64::MIN` case falls out correctly with no special-casing.
    fn lower_print_int(&mut self, value: Value) -> BackendResult<()> {
        let value = if self.builder.func.dfg.value_type(value) != types::I64 {
            self.builder.ins().sextend(types::I64, value)
        } else {
            value
        };

        const BUF_SIZE: u32 = 24;
        let slot = self.builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            BUF_SIZE,
            0,
        ));
        let pointer_type = self.module.isa().pointer_type();
        let base = self.builder.ins().stack_addr(pointer_type, slot, 0);

        let last = i64::from(BUF_SIZE) - 1;
        let zero_byte = self.builder.ins().iconst(types::I8, 0);
        let nul_addr = self.builder.ins().iadd_imm(base, last);
        self.builder
            .ins()
            .store(MemFlags::new(), zero_byte, nul_addr, 0);

        let is_neg = self.builder.ins().icmp_imm(IntCC::SignedLessThan, value, 0);
        let negated = self.builder.ins().ineg(value);
        let magnitude = self.builder.ins().select(is_neg, negated, value);

        let idx_var = self.new_var(types::I64);
        let first_digit_idx = self.builder.ins().iconst(types::I64, last - 1);
        self.builder.def_var(idx_var, first_digit_idx);
        let mag_var = self.new_var(types::I64);
        self.builder.def_var(mag_var, magnitude);

        let loop_block = self.builder.create_block();
        let exit_block = self.builder.create_block();
        self.builder.ins().jump(loop_block, &[]);

        self.builder.switch_to_block(loop_block);
        let mag = self.builder.use_var(mag_var);
        let digit = self.builder.ins().urem_imm(mag, 10);
        let quotient = self.builder.ins().udiv_imm(mag, 10);
        let digit_char = self.builder.ins().iadd_imm(digit, i64::from(b'0'));
        let digit_byte = self.builder.ins().ireduce(types::I8, digit_char);
        let idx = self.builder.use_var(idx_var);
        let digit_addr = self.builder.ins().iadd(base, idx);
        self.builder
            .ins()
            .store(MemFlags::new(), digit_byte, digit_addr, 0);

        let next_idx = self.builder.ins().iadd_imm(idx, -1);
        self.builder.def_var(idx_var, next_idx);
        self.builder.def_var(mag_var, quotient);

        let more_digits = self.builder.ins().icmp_imm(IntCC::NotEqual, quotient, 0);
        self.builder
            .ins()
            .brif(more_digits, loop_block, &[], exit_block, &[]);

        self.builder.switch_to_block(exit_block);
        // `idx_var` now holds one index before the first (most significant) digit written --
        // exactly where a '-' belongs, or where the digits themselves start if there isn't one.
        let sign_idx = self.builder.use_var(idx_var);
        let unsigned_start_idx = self.builder.ins().iadd_imm(sign_idx, 1);
        let minus_byte = self.builder.ins().iconst(types::I8, i64::from(b'-'));
        let sign_addr = self.builder.ins().iadd(base, sign_idx);
        self.builder
            .ins()
            .store(MemFlags::new(), minus_byte, sign_addr, 0);
        let start_idx = self
            .builder
            .ins()
            .select(is_neg, sign_idx, unsigned_start_idx);

        let str_ptr = self.builder.ins().iadd(base, start_idx);
        let callee = self
            .module
            .declare_func_in_func(self.puts_id, self.builder.func);
        self.builder.ins().call(callee, &[str_ptr]);
        Ok(())
    }
}

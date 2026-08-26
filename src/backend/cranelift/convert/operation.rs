use cranelift_codegen::ir::condcodes::{FloatCC, IntCC};
use cranelift_codegen::ir::{types, InstBuilder, Value};
use cranelift_frontend::FunctionBuilder;

use crate::backend::cranelift::convert::FnLower;
use crate::backend::cranelift::primitive::cranelift_type;
use crate::backend::cranelift::result::{BackendErr, BackendResult};
use crate::check::ast::{ASTTy, NodeTy};

impl<'a> FnLower<'a> {
    /// Lower a binary arithmetic or comparison operation.
    pub(super) fn lower_operation(&mut self, ast: &ASTTy) -> BackendResult<Value> {
        match &ast.node {
            NodeTy::Add { left, right } => self.lower_arith(
                left,
                right,
                |b, a, c| b.ins().iadd(a, c),
                |b, a, c| b.ins().fadd(a, c),
            ),
            NodeTy::Sub { left, right } => self.lower_arith(
                left,
                right,
                |b, a, c| b.ins().isub(a, c),
                |b, a, c| b.ins().fsub(a, c),
            ),
            NodeTy::Mul { left, right } => self.lower_arith(
                left,
                right,
                |b, a, c| b.ins().imul(a, c),
                |b, a, c| b.ins().fmul(a, c),
            ),
            NodeTy::Div { left, right } => self.lower_arith(
                left,
                right,
                |b, a, c| b.ins().sdiv(a, c),
                |b, a, c| b.ins().fdiv(a, c),
            ),
            NodeTy::Le { left, right } => {
                self.lower_cmp(left, right, IntCC::SignedLessThan, FloatCC::LessThan)
            }
            NodeTy::Leq { left, right } => self.lower_cmp(
                left,
                right,
                IntCC::SignedLessThanOrEqual,
                FloatCC::LessThanOrEqual,
            ),
            NodeTy::Ge { left, right } => {
                self.lower_cmp(left, right, IntCC::SignedGreaterThan, FloatCC::GreaterThan)
            }
            NodeTy::Geq { left, right } => self.lower_cmp(
                left,
                right,
                IntCC::SignedGreaterThanOrEqual,
                FloatCC::GreaterThanOrEqual,
            ),
            NodeTy::Eq { left, right } => self.lower_cmp(left, right, IntCC::Equal, FloatCC::Equal),
            NodeTy::Neq { left, right } => {
                self.lower_cmp(left, right, IntCC::NotEqual, FloatCC::NotEqual)
            }
            other => Err(BackendErr::unimplemented(
                ast,
                &format!("{other:?} operation"),
            )),
        }
    }

    /// Lower a binary arithmetic operation, picking `int_op` or `float_op` by `left`'s own
    /// Cranelift type -- `Int` and `Float` need different opcodes entirely (`iadd` vs `fadd` and
    /// so on), unlike comparisons where only the condition code differs.
    fn lower_arith(
        &mut self,
        left: &ASTTy,
        right: &ASTTy,
        int_op: impl Fn(&mut FunctionBuilder, Value, Value) -> Value,
        float_op: impl Fn(&mut FunctionBuilder, Value, Value) -> Value,
    ) -> BackendResult<Value> {
        // Reject non-primitive-typed arithmetic early, with a clear error. Checked against
        // `left`'s own type rather than the whole expression's -- e.g. as a `print(...)` argument,
        // the *expression*'s resolved type widens to whatever union `print` accepts, even though
        // the operands (and thus the actual machine type produced) stay concretely typed.
        let ty = cranelift_type(left)?;
        let l = self.lower_expr(left)?;
        let r = self.lower_expr(right)?;
        if ty == types::F64 {
            Ok(float_op(&mut self.builder, l, r))
        } else {
            Ok(int_op(&mut self.builder, l, r))
        }
    }

    /// Lower a comparison, picking `icmp`/`fcmp` (with the matching condition code) by `left`'s
    /// own Cranelift type.
    fn lower_cmp(
        &mut self,
        left: &ASTTy,
        right: &ASTTy,
        int_cc: IntCC,
        float_cc: FloatCC,
    ) -> BackendResult<Value> {
        let ty = cranelift_type(left)?;
        let l = self.lower_expr(left)?;
        let r = self.lower_expr(right)?;
        if ty == types::F64 {
            Ok(self.builder.ins().fcmp(float_cc, l, r))
        } else {
            Ok(self.builder.ins().icmp(int_cc, l, r))
        }
    }
}

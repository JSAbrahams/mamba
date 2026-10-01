use cranelift_codegen::ir::condcodes::{FloatCC, IntCC};
use cranelift_codegen::ir::{types, InstBuilder, Value};

use crate::backend::cranelift::convert::int::Arith;
use crate::backend::cranelift::convert::FnLower;
use crate::backend::cranelift::result::{BackendErr, BackendResult};
use crate::check::ast::{ASTTy, NodeTy};

impl<'a> FnLower<'a> {
    /// Lower a binary arithmetic operation, a comparison, or a unary `+`/`-`.
    pub(super) fn lower_operation(&mut self, ast: &ASTTy) -> BackendResult<Value> {
        match &ast.node {
            NodeTy::Add { left, right } => self.lower_arith(left, right, Arith::Add),
            NodeTy::Sub { left, right } => self.lower_arith(left, right, Arith::Sub),
            NodeTy::Mul { left, right } => self.lower_arith(left, right, Arith::Mul),
            NodeTy::Div { left, right } => self.lower_div(left, right),
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
            NodeTy::AddU { expr } => self.lower_expr(expr),
            NodeTy::SubU { expr } => self.lower_negate(expr),
            other => Err(BackendErr::unimplemented(
                ast,
                &format!("{other:?} operation"),
            )),
        }
    }

    /// Lower a binary arithmetic operation, as a `Float` one if either operand turns out to be `Float`.
    /// See [`Self::float_pair`]'s doc comment for how that is decided.
    /// Otherwise it is an `Int` one, which never wraps: see [`Self::int_arith`].
    fn lower_arith(&mut self, left: &ASTTy, right: &ASTTy, op: Arith) -> BackendResult<Value> {
        let l = self.lower_expr(left)?;
        let r = self.lower_expr(right)?;
        Ok(match (self.float_pair(l, r), op) {
            (Some((l, r)), Arith::Add) => self.builder.ins().fadd(l, r),
            (Some((l, r)), Arith::Sub) => self.builder.ins().fsub(l, r),
            (Some((l, r)), Arith::Mul) => self.builder.ins().fmul(l, r),
            (None, _) => self.int_arith(op, l, r),
        })
    }

    /// Lower Mamba's `/`, which -- like Python's `/` -- is always true (float) division: unlike
    /// `+`/`-`/`*`, which preserve the operand type, `Int / Int` still produces a `Float` (see
    /// `int.__truediv__`'s signature in `check/resource/primitive/int.py`). `Int` operands are
    /// converted to `Float` first; `//` (floor division, preserving `Int`) isn't implemented.
    fn lower_div(&mut self, left: &ASTTy, right: &ASTTy) -> BackendResult<Value> {
        let l = self.lower_expr(left)?;
        let r = self.lower_expr(right)?;
        // Unlike the other arithmetic ops, `/` always produces a `Float` even for two `Int`
        // operands, so unconditionally treat this as the float pair (converting both if needed)
        // rather than only converting when one side already happens to be `Float`.
        let (l, r) = match self.float_pair(l, r) {
            Some(pair) => pair,
            None => (self.int_to_float(l), self.int_to_float(r)),
        };
        Ok(self.builder.ins().fdiv(l, r))
    }

    /// Lower unary negation (`-x`), by `expr`'s actual Cranelift value type.
    /// An `Int` is subtracted from zero, since negating the smallest small integer overflows it.
    fn lower_negate(&mut self, expr: &ASTTy) -> BackendResult<Value> {
        let value = self.lower_expr(expr)?;
        if self.builder.func.dfg.value_type(value) == types::F64 {
            Ok(self.builder.ins().fneg(value))
        } else {
            let zero = self.builder.ins().iconst(types::I64, 0);
            Ok(self.int_arith(Arith::Sub, zero, value))
        }
    }

    /// Lower a comparison, picking `icmp`/`fcmp` (with the matching condition code) depending on
    /// whether either operand turns out to be `Float`. See [`Self::float_pair`]'s doc comment.
    /// Two `Int` operands go through [`Self::int_cmp`], and only two `Bool` ones are a plain `icmp`.
    fn lower_cmp(
        &mut self,
        left: &ASTTy,
        right: &ASTTy,
        int_cc: IntCC,
        float_cc: FloatCC,
    ) -> BackendResult<Value> {
        let l = self.lower_expr(left)?;
        let r = self.lower_expr(right)?;
        match self.float_pair(l, r) {
            Some((l, r)) => Ok(self.builder.ins().fcmp(float_cc, l, r)),
            None if self.builder.func.dfg.value_type(l) == types::I64 => {
                Ok(self.int_cmp(int_cc, l, r))
            }
            None => Ok(self.builder.ins().icmp(int_cc, l, r)),
        }
    }

    /// If either `l` or `r` is actually `Float`, return both as `Float` (converting whichever
    /// isn't); otherwise `None` (both are `Int`/`Bool`, handle as integer).
    ///
    /// Decided from the *lowered values'* actual Cranelift types, not `cranelift_type(...)` on
    /// the AST: a checked operand's resolved `ty` can come back reflecting the operator's own
    /// (possibly generic) parameter type rather than the operand's own concrete type -- e.g.
    /// `int.__gt__`'s parameter is typed `Union[int, float]`, and unifying against it can leave
    /// an `Int` literal's resolved type looking like `Float` even when every value actually
    /// involved is `Int`. A value `lower_expr` already produced has no such ambiguity: it's
    /// exactly the machine type that was built, by construction -- so promotion is decided by
    /// asking "did the *other* side turn out to be a real `Float` value", not by asking either
    /// side what the checker inferred for it in isolation.
    fn float_pair(&mut self, l: Value, r: Value) -> Option<(Value, Value)> {
        let l_is_float = self.builder.func.dfg.value_type(l) == types::F64;
        let r_is_float = self.builder.func.dfg.value_type(r) == types::F64;
        if !l_is_float && !r_is_float {
            return None;
        }
        let l = if l_is_float { l } else { self.int_to_float(l) };
        let r = if r_is_float { r } else { self.int_to_float(r) };
        Some((l, r))
    }
}

use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::{InstBuilder, Value};
use cranelift_frontend::FunctionBuilder;

use crate::backend::cranelift::convert::FnLower;
use crate::backend::cranelift::primitive::cranelift_type;
use crate::backend::cranelift::result::{BackendErr, BackendResult};
use crate::check::ast::{ASTTy, NodeTy};

impl<'a> FnLower<'a> {
    /// Lower a binary arithmetic or comparison operation.
    pub(super) fn lower_operation(&mut self, ast: &ASTTy) -> BackendResult<Value> {
        match &ast.node {
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
            other => Err(BackendErr::unimplemented(
                ast,
                &format!("{other:?} operation"),
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
}

use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::{types, InstBuilder};

use crate::backend::cranelift::convert::common::fun_name;
use crate::backend::cranelift::convert::FnLower;
use crate::backend::cranelift::primitive::cranelift_type;
use crate::backend::cranelift::result::{BackendErr, BackendResult};
use crate::check::ast::{ASTTy, NodeTy};

impl<'a> FnLower<'a> {
    /// Lower an `IfElse` in statement position: both arms are lowered as statements, and control
    /// re-joins in a shared `merge_block` afterwards (or falls straight through to it when there
    /// is no `else`).
    pub(super) fn lower_if_else_stmt(
        &mut self,
        cond: &ASTTy,
        then: &ASTTy,
        el: Option<&ASTTy>,
    ) -> BackendResult<()> {
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

    /// Lower a `For` loop in statement position.
    ///
    /// Only `for <id> in <from> .. <to>` / `for <id> in <from> ..= <to>` over `Int` bounds is
    /// supported -- iterating a collection would need this backend to support collections at all,
    /// which is out of scope (see the module-level docs).
    ///
    /// Classic three-block loop: `header` checks the bound and either enters `body` or falls
    /// through to `exit`; `body` runs the loop body (with the loop variable shadowing any
    /// outer binding of the same name for its duration) and increments before jumping back to
    /// `header`.
    pub(super) fn lower_for(
        &mut self,
        expr: &ASTTy,
        col: &ASTTy,
        body: &ASTTy,
    ) -> BackendResult<()> {
        let (from, to, inclusive, step) = match &col.node {
            NodeTy::Range {
                from,
                to,
                inclusive,
                step,
            } => (from, to, *inclusive, step),
            other => {
                return Err(BackendErr::unimplemented(
                    col,
                    &format!("{other:?} for-loop collection"),
                ))
            }
        };
        let var_name = fun_name(expr)?;

        let ty = cranelift_type(from)?;
        if ty != types::I64 {
            return Err(BackendErr::new(
                from.pos,
                "For-loops are only supported over Int ranges",
            ));
        }

        let from_value = self.lower_expr(from)?;
        let to_value = self.lower_expr(to)?;
        let step_value = match step {
            Some(step) => self.lower_expr(step)?,
            None => self.builder.ins().iconst(types::I64, 1),
        };

        let loop_var = self.new_var(ty);
        self.builder.def_var(loop_var, from_value);

        let header_block = self.builder.create_block();
        let body_block = self.builder.create_block();
        let exit_block = self.builder.create_block();
        self.builder.ins().jump(header_block, &[]);

        self.builder.switch_to_block(header_block);
        let current = self.builder.use_var(loop_var);
        let cc = if inclusive {
            IntCC::SignedLessThanOrEqual
        } else {
            IntCC::SignedLessThan
        };
        let cond = self.builder.ins().icmp(cc, current, to_value);
        self.builder
            .ins()
            .brif(cond, body_block, &[], exit_block, &[]);

        self.builder.switch_to_block(body_block);
        let shadowed = self.vars.insert(var_name.clone(), (loop_var, ty));
        self.lower_stmt(body)?;
        match shadowed {
            Some(shadowed) => {
                self.vars.insert(var_name, shadowed);
            }
            None => {
                self.vars.remove(&var_name);
            }
        }
        let current = self.builder.use_var(loop_var);
        let next = self.builder.ins().iadd(current, step_value);
        self.builder.def_var(loop_var, next);
        self.builder.ins().jump(header_block, &[]);

        self.builder.switch_to_block(exit_block);
        Ok(())
    }

    /// Lower an `IfElse` in tail position: both arms must themselves end in a `return`, so unlike
    /// [`Self::lower_if_else_stmt`] there is no shared merge block to jump back to.
    pub(super) fn lower_if_else_tail(
        &mut self,
        cond: &ASTTy,
        then: &ASTTy,
        el: Option<&ASTTy>,
    ) -> BackendResult<()> {
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
}

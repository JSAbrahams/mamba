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
    /// Rotated loop, entered through a pre-check: `entry_check` handles the (possibly empty
    /// up-front) range and falls through to `exit` or into `body`; `body` runs the loop body,
    /// computes the next value, and -- checking *that* -- either commits it and loops back into
    /// `body` directly (skipping `entry_check` on every subsequent iteration) or falls through to
    /// `exit` without committing it.
    ///
    /// Mamba (like Python, which this must match) has no block scoping, so the loop variable is
    /// just an ordinary binding of its name -- if that name already existed, this loop
    /// permanently overwrites it. Committing the next value only on the branch that's actually
    /// going to use it (rather than unconditionally at the bottom of `body`, then discovering at
    /// `header` that it was one too many and exiting anyway) is what makes the loop variable keep
    /// whatever value it was last actually *used* with once the loop exits, instead of one past
    /// it -- matching what Python's own `for` leaves its loop variable holding.
    ///
    /// One known gap: for a range that's empty from the start (e.g. `for i in 5 .. 5`), Python's
    /// `i` never gets touched at all -- it keeps whatever it held *before* the loop -- whereas
    /// this still binds it to `from` before the (never-taken) entry check. Shadowing an outer
    /// variable with a loop that may run zero times is the only way this is observable, and isn't
    /// worth the extra restructuring to close.
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
        self.vars.insert(var_name, (loop_var, ty));

        let cc = if inclusive {
            IntCC::SignedLessThanOrEqual
        } else {
            IntCC::SignedLessThan
        };

        let entry_check_block = self.builder.create_block();
        let body_block = self.builder.create_block();
        let continue_block = self.builder.create_block();
        let exit_block = self.builder.create_block();
        self.builder.ins().jump(entry_check_block, &[]);

        self.builder.switch_to_block(entry_check_block);
        let current = self.builder.use_var(loop_var);
        let cond = self.builder.ins().icmp(cc, current, to_value);
        self.builder
            .ins()
            .brif(cond, body_block, &[], exit_block, &[]);

        self.builder.switch_to_block(body_block);
        self.lower_stmt(body)?;
        let current = self.builder.use_var(loop_var);
        let next = self.builder.ins().iadd(current, step_value);
        let cond = self.builder.ins().icmp(cc, next, to_value);
        self.builder
            .ins()
            .brif(cond, continue_block, &[], exit_block, &[]);

        self.builder.switch_to_block(continue_block);
        self.builder.def_var(loop_var, next);
        self.builder.ins().jump(body_block, &[]);

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

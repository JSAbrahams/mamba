use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::{types, InstBuilder};

use crate::backend::cranelift::convert::common::fun_name;
use crate::backend::cranelift::convert::FnLower;
use crate::backend::cranelift::primitive::cranelift_type;
use crate::backend::cranelift::result::{BackendErr, BackendResult};
use crate::check::ast::{ASTTy, NodeTy};

impl<'a> FnLower<'a> {
    /// Lower `ast` as a statement within its own scope: any variable binding it introduces --
    /// most directly a `def` that shadows an outer variable, but also a for-loop's own control
    /// variable (`lower_for` relies on this for that) -- is undone once `ast` is done, so it
    /// never persists past the block it belongs to.
    ///
    /// This is the whole mechanism behind Mamba having real block scoping for `def`, unlike
    /// Python (which this backend must still *behave* like Python for everything else -- e.g.
    /// reassigning an outer variable with `:=`, which isn't a new binding, still works exactly as
    /// expected; only fresh bindings are undone here, since a `:=` never touches `self.vars`, only
    /// the value already tracked by whichever `Variable` the name already resolves to).
    fn lower_scoped_stmt(&mut self, ast: &ASTTy) -> BackendResult<()> {
        let snapshot = self.vars.clone();
        let result = self.lower_stmt(ast);
        self.vars = snapshot;
        result
    }

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
        self.lower_scoped_stmt(then)?;
        self.builder.ins().jump(merge_block, &[]);

        if let Some(el) = el {
            self.builder.switch_to_block(else_block);
            self.lower_scoped_stmt(el)?;
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
    /// through to `exit`; `body` runs the loop body and increments before jumping back to
    /// `header`.
    ///
    /// The loop variable is bound inside `body`'s own scope (see [`Self::lower_scoped_stmt`]), so
    /// it's its own fresh Cranelift `Variable` (never the same one as any outer variable of the
    /// same name) and never persists past the loop. So a `for` never touches an outer variable it
    /// happens to shadow -- during the loop *or* after it -- unlike Python's own `for`, which has
    /// no block scoping and would happily clobber it. (The Python backend has to work to emulate
    /// this same guarantee, since generated Python doesn't get it for free -- see
    /// `backend::python::convert::control_flow`.)
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
        let snapshot = self.vars.clone();
        self.vars.insert(var_name, (loop_var, ty));
        let result = self.lower_stmt(body);
        self.vars = snapshot;
        result?;
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

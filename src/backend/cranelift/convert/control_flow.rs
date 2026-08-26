use cranelift_codegen::ir::InstBuilder;

use crate::backend::cranelift::convert::FnLower;
use crate::backend::cranelift::result::BackendResult;
use crate::check::ast::ASTTy;

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

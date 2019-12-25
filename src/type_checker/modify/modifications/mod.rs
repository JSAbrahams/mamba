use crate::parser::ast::AST;
use crate::type_checker::context::Context;
use crate::type_checker::type_result::TypeResult;

pub mod constructor;
pub mod forward;
pub mod retry;

pub trait Modification {
    fn modify(&self, ast: &AST, ctx: &Context) -> TypeResult<AST>;
}

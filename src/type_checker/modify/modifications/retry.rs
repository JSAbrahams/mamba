use crate::parser::ast::AST;
use crate::type_checker::context::Context;
use crate::type_checker::modify::modifications::Modification;
use crate::type_checker::type_result::TypeResult;

pub struct Retry;

impl Retry {
    pub fn new() -> Retry { Retry {} }
}

impl Modification for Retry {
    fn modify(&self, ast: &AST, ctx: &Context) -> TypeResult<AST> { unimplemented!() }
}

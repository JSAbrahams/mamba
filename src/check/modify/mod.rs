use crate::check::checker_result::TypeResult;
use crate::check::context::Context;
use crate::check::modify::modifications::constructor::Constructor;
use crate::check::modify::modifications::Modification;
use crate::parse::ast::AST;

mod modifications;

pub fn modify(ast: &AST, ctx: &Context) -> TypeResult<AST> {
    let modifications: Vec<Box<dyn Modification>> = vec![Box::from(Constructor::new())];

    let mut ast = ast.clone();
    for modification in modifications {
        ast = modification.modify(&ast, ctx)?.0;
    }

    Ok(ast)
}

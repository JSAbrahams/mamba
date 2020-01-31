use crate::parser::ast::AST;
use crate::type_checker::checker_result::TypeResult;
use crate::type_checker::context::Context;
use crate::type_checker::modify::modifications::constructor::Constructor;
use crate::type_checker::modify::modifications::Modification;

mod modifications;

pub fn modify(ast: &AST, ctx: &Context) -> TypeResult<AST> {
    let modifications: Vec<Box<dyn Modification>> = vec![Box::from(Constructor::new())];

    let mut ast = ast.clone();
    for modification in modifications {
        ast = modification.modify(&ast, ctx)?.0;
    }

    Ok(ast)
}

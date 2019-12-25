use crate::parser::ast::AST;
use crate::type_checker::context::Context;
use crate::type_checker::modify::modifications::constructor::Constructor;
use crate::type_checker::modify::modifications::forward::Forward;
use crate::type_checker::modify::modifications::retry::Retry;
use crate::type_checker::modify::modifications::Modification;
use crate::type_checker::type_result::TypeResult;

mod modifications;

pub fn modify(ast: &AST, ctx: &Context) -> TypeResult<AST> {
    let modifications: Vec<Box<dyn Modification>> =
        vec![Box::from(Constructor::new()), Box::from(Forward::new()), Box::from(Retry::new())];

    let mut ast = ast.clone();
    for modification in modifications {
        ast = modification.modify(&ast, ctx)?;
    }

    Ok(ast.clone())
}

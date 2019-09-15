use crate::parser::ast::AST;
use crate::type_checker::context::Context;
use crate::type_checker::environment::ty::Type;
use crate::type_checker::environment::Environment;
use crate::type_checker::type_result::TypeErr;
use crate::type_checker::CheckInput;

pub type InferResult<T = Option<Box<Type>>> = std::result::Result<(T, Environment), Vec<TypeErr>>;

pub fn check(inputs: &[CheckInput], env: &Environment, ctx: &Context) -> Result<(), Vec<TypeErr>> {
    for (input, ..) in inputs {
        check_direct(input, &env.clone(), ctx)?;
    }
    Ok(())
}

fn check_direct(_: &AST, _: &Environment, _: &Context) -> InferResult { unimplemented!() }

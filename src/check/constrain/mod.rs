use crate::check::constrain::constraint::iterator::Constraints;
use crate::check::constrain::generate::gen_all;
use crate::check::constrain::unify::unify;
use crate::check::context::Context;
use crate::check::result::TypeErr;
use crate::parse::ast::AST;

pub mod constraint;

mod generate;
mod unify;

pub type Unified<T = Constraints> = Result<T, Vec<TypeErr>>;

pub fn constraints(ast: &AST, ctx: &Context) -> Unified<Vec<Constraints>> {
    let constrained = gen_all(ast, ctx)?;
    unify(&constrained, ctx)
}

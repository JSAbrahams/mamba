use crate::check::constrain::generate::gen_all;
use crate::check::constrain::unify::finished::Finished;
use crate::check::constrain::unify::unify;
use crate::check::context::Context;
use crate::check::result::TypeErr;
use crate::parse::ast::AST;

pub(super) mod constraint;

mod generate;
pub(super) mod unify;

pub type Unified<T = Finished> = Result<T, Vec<TypeErr>>;

pub fn constraints(ast: &AST, ctx: &Context) -> Unified<Finished> {
    let constrained = gen_all(ast, ctx)?;
    unify(&constrained, ctx)
}

#[cfg(test)]
mod tests {
    use crate::check::constrain::constraints;
    use crate::check::context::Context;
    use crate::check::name::{Name, Nullable};
    use crate::common::position::{CaretPos, Position};
    use crate::parse::parse;

    #[test]
    fn if_stmt_no_type() {
        let src = "if True then 10 else 20";
        let ast = parse(src).unwrap();
        let finished = constraints(&ast, &Context::default().into_with_primitives().unwrap()).unwrap().pos_to_name;

        let pos_bool = Position::new(CaretPos::new(1, 4), CaretPos::new(1, 8));
        // is interchangeable since call to __bool__() in Bool
        assert_eq!(finished[&pos_bool], Name::from("Bool"));

        let pos_10 = Position::new(CaretPos::new(1, 14), CaretPos::new(1, 16));
        assert_eq!(finished[&pos_10], Name::from("Int"));
        let pos_20 = Position::new(CaretPos::new(1, 22), CaretPos::new(1, 24));
        assert_eq!(finished[&pos_20], Name::from("Int"));

        assert_eq!(finished.len(), 3);
    }

    #[test]
    fn it_stmt_as_expression() {
        let src = "def a := if True then 10 else 20";
        let ast = parse(src).unwrap();
        let finished = constraints(&ast, &Context::default().into_with_primitives().unwrap()).unwrap().pos_to_name;

        let pos_20 = Position::new(CaretPos::new(1, 31), CaretPos::new(1, 33));
        assert_eq!(finished[&pos_20], Name::from("Int"));
        let pos_10 = Position::new(CaretPos::new(1, 23), CaretPos::new(1, 25));
        assert_eq!(finished[&pos_10], Name::from("Int"));
        let pos_bool = Position::new(CaretPos::new(1, 13), CaretPos::new(1, 17));
        // is interchangeable since call to __bool__() in Bool
        assert_eq!(finished[&pos_bool], Name::from("Bool"));

        let pos_if = Position::new(CaretPos::new(1, 10), CaretPos::new(1, 33));
        assert_eq!(finished[&pos_if], Name::from("Int"));

        let pos_var = Position::new(CaretPos::new(1, 5), CaretPos::new(1, 6));
        assert_eq!(finished[&pos_var], Name::from("Int"));

        assert_eq!(finished.len(), 5);
    }

    #[test]
    fn it_stmt_as_expression_none() {
        let src = "def a := if True then 10 else None";
        let ast = parse(src).unwrap();
        let finished = constraints(&ast, &Context::default().into_with_primitives().unwrap()).unwrap().pos_to_name;

        let pos_none = Position::new(CaretPos::new(1, 31), CaretPos::new(1, 35));
        assert_eq!(finished[&pos_none], Name::from("None"));
        let pos_10 = Position::new(CaretPos::new(1, 23), CaretPos::new(1, 25));
        assert_eq!(finished[&pos_10], Name::from("Int"));
        let pos_bool = Position::new(CaretPos::new(1, 13), CaretPos::new(1, 17));
        // is interchangeable since call to __bool__() in Bool
        assert_eq!(finished[&pos_bool], Name::from("Bool"));

        let pos_if = Position::new(CaretPos::new(1, 10), CaretPos::new(1, 35));
        assert_eq!(finished[&pos_if], Name::from("Int").as_nullable());
    }
}

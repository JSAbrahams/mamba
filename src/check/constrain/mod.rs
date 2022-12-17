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

#[cfg(test)]
mod tests {
    use crate::check::constrain::constraint::iterator::Constraints;
    use crate::check::constrain::constraints;
    use crate::check::context::Context;
    use crate::check::name::{Name, Nullable};
    use crate::common::position::{CaretPos, Position};
    use crate::parse::parse;

    #[test]
    fn if_stmt_no_type() {
        let src = "if True then 10 else 20";
        let ast = parse(src).unwrap();
        let constraints = constraints(&ast, &Context::default().into_with_primitives().unwrap()).unwrap();

        // if and then branch
        let inner_constr = constraints[0].clone();

        assert_eq!(inner_constr.finished.len(), 2);
        let pos_10 = Position::new(CaretPos::new(1, 14), CaretPos::new(1, 16));
        assert_eq!(inner_constr.finished[&pos_10], Name::from("Int"));
        let pos_bool = Position::new(CaretPos::new(1, 4), CaretPos::new(1, 8));
        assert_eq!(inner_constr.finished[&pos_bool], Name::from("Bool"));

        // if and else branch
        let inner_constr = constraints[1].clone();

        assert_eq!(inner_constr.finished.len(), 2);
        let pos_20 = Position::new(CaretPos::new(1, 22), CaretPos::new(1, 24));
        assert_eq!(inner_constr.finished[&pos_20], Name::from("Int"));
        let pos_bool = Position::new(CaretPos::new(1, 4), CaretPos::new(1, 8));
        assert_eq!(inner_constr.finished[&pos_bool], Name::from("Bool"));

        // if
        let inner_constr = constraints[2].clone();
        assert_eq!(inner_constr.len(), 0);
    }

    #[test]
    fn it_stmt_as_expression() {
        let src = "def a := if True then 10 else 20";
        let ast = parse(src).unwrap();
        let constraints = constraints(&ast, &Context::default().into_with_primitives().unwrap()).unwrap();

        // Ignore then and else branch
        let inner_constr = constraints
            .iter()
            .fold(Constraints::new(), |acc, constr| {
                constr.finished.iter().fold(acc, |mut acc, (pos, name)| {
                    acc.push_ty(*pos, name);
                    acc
                })
            }).finished;

        assert_eq!(inner_constr.len(), 4);
        let pos_20 = Position::new(CaretPos::new(1, 31), CaretPos::new(1, 33));
        assert_eq!(inner_constr[&pos_20], Name::from("Int"));
        let pos_10 = Position::new(CaretPos::new(1, 23), CaretPos::new(1, 25));
        assert_eq!(inner_constr[&pos_10], Name::from("Int"));
        let pos_bool = Position::new(CaretPos::new(1, 13), CaretPos::new(1, 17));
        assert_eq!(inner_constr[&pos_bool], Name::from("Bool"));

        let pos_if = Position::new(CaretPos::new(1, 10), CaretPos::new(1, 33));
        assert_eq!(inner_constr[&pos_if], Name::from("Int"));
    }

    #[test]
    #[ignore] // not sure if the check stage should pass as of yet
    fn it_stmt_as_expression_none() {
        let src = "def a := if True then 10 else None";
        let ast = parse(src).unwrap();
        let constraints = constraints(&ast, &Context::default().into_with_primitives().unwrap()).unwrap();

        // Ignore then and else branch
        let inner_constr = constraints
            .iter()
            .fold(Constraints::new(), |acc, constr| {
                constr.finished.iter().fold(acc, |mut acc, (pos, name)| {
                    acc.push_ty(*pos, name);
                    acc
                })
            }).finished;

        let pos_none = Position::new(CaretPos::new(1, 31), CaretPos::new(1, 35));
        assert_eq!(inner_constr[&pos_none], Name::from("None"));
        let pos_10 = Position::new(CaretPos::new(1, 23), CaretPos::new(1, 25));
        assert_eq!(inner_constr[&pos_10], Name::from("Int"));
        let pos_bool = Position::new(CaretPos::new(1, 13), CaretPos::new(1, 17));
        assert_eq!(inner_constr[&pos_bool], Name::from("Bool"));

        let pos_if = Position::new(CaretPos::new(1, 10), CaretPos::new(1, 35));
        assert_eq!(inner_constr[&pos_if], Name::from("Int").as_nullable());
    }
}

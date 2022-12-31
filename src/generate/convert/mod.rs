use std::convert::TryFrom;

use crate::{ASTTy, Context};
use crate::check::ast::NodeTy;
use crate::check::context::clss::concrete_to_python;
use crate::generate::ast::node::{Core, CoreOp};
use crate::generate::convert::builder::convert_builder;
use crate::generate::convert::call::convert_call;
use crate::generate::convert::class::convert_class;
use crate::generate::convert::common::convert_vec;
use crate::generate::convert::control_flow::convert_cntrl_flow;
use crate::generate::convert::definition::convert_def;
use crate::generate::convert::handle::convert_handle;
use crate::generate::convert::range_slice::convert_range_slice;
use crate::generate::convert::state::{Imports, State};
use crate::generate::convert::ty::convert_ty;
use crate::generate::result::{GenResult, UnimplementedErr};

mod builder;
mod call;
mod class;
mod common;
mod control_flow;
mod definition;
mod handle;
mod range_slice;
mod ty;

pub mod state;

pub fn convert_node(ast: &ASTTy, imp: &mut Imports, state: &State, ctx: &Context) -> GenResult {
    // Prevent these state properties from propagating further
    let must_assign_to = state.must_assign_to.clone();
    let is_last_must_be_ret = state.is_last_must_be_ret;

    let state = &state.must_assign_to(None).is_last_must_be_ret(false);

    let core = match &ast.node {
        NodeTy::Import { from, import, alias } => Core::Import {
            from: if let Some(from) = from {
                Some(Box::from(convert_node(from, imp, state, ctx)?))
            } else {
                None
            },
            import: convert_vec(import, imp, state, ctx)?,
            alias: convert_vec(alias, imp, state, ctx)?,
        },

        NodeTy::VariableDef { .. } | NodeTy::FunDef { .. } | NodeTy::FunArg { .. } => {
            convert_def(ast, imp, state, ctx)?
        }
        NodeTy::Reassign { left, right, op } => Core::Assign {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
            op: CoreOp::try_from((ast, op))?,
        },

        NodeTy::Block { statements } => Core::Block {
            statements: convert_vec(statements, imp, state, ctx)?,
        },

        NodeTy::Int { lit } => Core::Int { int: lit.clone() },
        NodeTy::Real { lit } => Core::Float { float: lit.clone() },
        NodeTy::ENum { num, exp } => Core::ENum {
            num: num.clone(),
            exp: if exp.is_empty() { String::from("0") } else { exp.clone() },
        },
        NodeTy::DocStr { lit } => Core::DocStr { string: lit.clone() },
        NodeTy::Str { lit, expressions } if expressions.is_empty() => {
            Core::Str { string: lit.clone() }
        }
        NodeTy::Str { lit, .. } => Core::FStr { string: lit.clone() },

        NodeTy::QuestionOp { .. } => convert_ty(ast, imp, state, ctx)?,

        NodeTy::Undefined => Core::None,
        NodeTy::ExpressionType { .. } => convert_ty(ast, imp, state, ctx)?,
        NodeTy::Id { lit } => Core::Id { lit: concrete_to_python(lit) },
        NodeTy::Bool { lit } => Core::Bool { boolean: *lit },

        NodeTy::Tuple { elements } if state.tup_lit => {
            Core::TupleLiteral { elements: convert_vec(elements, imp, state, ctx)? }
        }
        NodeTy::Tuple { elements } => Core::Tuple { elements: convert_vec(elements, imp, state, ctx)? },
        NodeTy::List { elements } => Core::List { elements: convert_vec(elements, imp, state, ctx)? },
        NodeTy::Set { elements } => Core::Set { elements: convert_vec(elements, imp, state, ctx)? },
        NodeTy::Index { item, range } => Core::Index {
            item: Box::from(convert_node(item, imp, state, ctx)?),
            range: Box::from(convert_node(range, imp, state, ctx)?),
        },

        NodeTy::ListBuilder { .. } => convert_builder(ast, imp, state, ctx)?,
        NodeTy::SetBuilder { .. } => convert_builder(ast, imp, state, ctx)?,

        NodeTy::ReturnEmpty => Core::Return { expr: Box::from(Core::None) },
        NodeTy::Return { expr } if state.is_remove_last_ret => convert_node(expr, imp, &state.remove_ret(false), ctx)?,
        NodeTy::Return { expr } => {
            Core::Return { expr: Box::from(convert_node(expr, imp, state, ctx)?) }
        }

        NodeTy::IfElse { .. } => convert_cntrl_flow(ast, imp, &state.is_last_must_be_ret(is_last_must_be_ret).must_assign_to(must_assign_to.as_ref()), ctx)?,
        NodeTy::Match { .. } => convert_cntrl_flow(ast, imp, &state.expand_ty(false).is_last_must_be_ret(is_last_must_be_ret).must_assign_to(must_assign_to.as_ref()), ctx)?,
        NodeTy::While { .. } | NodeTy::For { .. } | NodeTy::Break | NodeTy::Continue => convert_cntrl_flow(ast, imp, state, ctx)?,

        NodeTy::Not { expr } => Core::Not { expr: Box::from(convert_node(expr, imp, state, ctx)?) },
        NodeTy::And { left, right } => Core::And {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },
        NodeTy::Or { left, right } => Core::Or {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },
        NodeTy::Is { left, right } => Core::Is {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },
        NodeTy::IsN { left, right } => Core::IsN {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },
        NodeTy::Eq { left, right } => Core::Eq {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },
        NodeTy::Neq { left, right } => Core::Neq {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },
        NodeTy::IsA { left, right } => Core::IsA {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },
        NodeTy::IsNA { left, right } => Core::Not {
            expr: Box::from(Core::IsA {
                left: Box::from(convert_node(left, imp, state, ctx)?),
                right: Box::from(convert_node(right, imp, state, ctx)?),
            }),
        },

        NodeTy::Add { left, right } => Core::Add {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },
        NodeTy::Sub { left, right } => Core::Sub {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },
        NodeTy::Mul { left, right } => Core::Mul {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },
        NodeTy::Div { left, right } => Core::Div {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },
        NodeTy::FDiv { left, right } => Core::FDiv {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },
        NodeTy::Mod { left, right } => Core::Mod {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },
        NodeTy::Pow { left, right } => Core::Pow {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },

        NodeTy::BAnd { left, right } => Core::BAnd {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },
        NodeTy::BOr { left, right } => Core::BOr {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },
        NodeTy::BXOr { left, right } => Core::BXOr {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },
        NodeTy::BOneCmpl { expr } => {
            Core::BOneCmpl { expr: Box::from(convert_node(expr, imp, state, ctx)?) }
        }
        NodeTy::BLShift { left, right } => Core::BLShift {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },
        NodeTy::BRShift { left, right } => Core::BRShift {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },

        NodeTy::AddU { expr } => Core::AddU { expr: Box::from(convert_node(expr, imp, state, ctx)?) },
        NodeTy::SubU { expr } => Core::SubU { expr: Box::from(convert_node(expr, imp, state, ctx)?) },
        NodeTy::Sqrt { expr } => {
            imp.add_import("math");
            Core::Sqrt { expr: Box::from(convert_node(expr, imp, state, ctx)?) }
        }

        NodeTy::Le { left, right } => Core::Le {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },
        NodeTy::Leq { left, right } => Core::Leq {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },
        NodeTy::Ge { left, right } => Core::Ge {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },
        NodeTy::Geq { left, right } => Core::Geq {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },

        NodeTy::FunctionCall { .. } | NodeTy::PropertyCall { .. } => convert_call(ast, imp, state, ctx)?,
        NodeTy::AnonFun { args, body } => Core::AnonFun {
            args: convert_vec(args, imp, &state.expand_ty(false), ctx)?,
            body: Box::from(convert_node(body, imp, state, ctx)?),
        },

        NodeTy::In { left, right } => Core::In {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },
        NodeTy::Range { .. } | NodeTy::Slice { .. } => convert_range_slice(ast, imp, state, ctx)?,

        NodeTy::Underscore => Core::UnderScore,
        NodeTy::Question { left, right } => Core::Or {
            left: Box::from(convert_node(left, imp, state, ctx)?),
            right: Box::from(convert_node(right, imp, state, ctx)?),
        },

        NodeTy::TypeTup { .. }
        | NodeTy::Type { .. }
        | NodeTy::TypeFun { .. }
        | NodeTy::TypeUnion { .. } => convert_ty(ast, imp, state, ctx)?,

        NodeTy::TypeDef { .. } | NodeTy::TypeAlias { .. } => convert_class(ast, imp, state, ctx)?,
        NodeTy::Class { .. } => convert_class(ast, imp, state, ctx)?,
        NodeTy::Generic { .. } => Core::Empty,
        NodeTy::Parent { .. } => convert_class(ast, imp, state, ctx)?,

        NodeTy::Condition { .. } => return Err(Box::from(UnimplementedErr::new(ast, "condition"))),

        NodeTy::With { resource, alias: Some((alias, ..)), expr } => Core::WithAs {
            resource: Box::from(convert_node(resource, imp, state, ctx)?),
            alias: Box::from(convert_node(alias, imp, &state.expand_ty(false), ctx)?),
            expr: Box::from(convert_node(expr, imp, state, ctx)?),
        },
        NodeTy::With { resource, expr, .. } => Core::With {
            resource: Box::from(convert_node(resource, imp, state, ctx)?),
            expr: Box::from(convert_node(expr, imp, state, ctx)?),
        },

        NodeTy::Raises { .. } | NodeTy::Raise { .. } | NodeTy::Handle { .. } => {
            convert_handle(ast, imp, state, ctx)?
        }

        NodeTy::Pass => Core::Pass,
        _ => Core::Empty,
    };

    let core = if let Some(assign_to) = must_assign_to {
        append_assign(&core, &assign_to)
    } else { core };

    let core = if is_last_must_be_ret {
        append_ret(&core)
    } else { core };

    Ok(core)
}

fn append_assign(core: &Core, assign_to: &Core) -> Core {
    match &core {
        Core::Block { ref statements } => match statements.last() {
            Some(last) => {
                let last = append_assign(last, assign_to);
                let (mut statements, idx): (Vec<Core>, usize) = (statements.clone(), statements.len() - 1);
                statements[idx] = last;
                Core::Block { statements }
            }
            None => core.clone()
        }
        Core::IfElse { cond, then, el } => Core::IfElse {
            cond: cond.clone(),
            then: Box::from(append_assign(then, assign_to)),
            el: Box::from(append_assign(el, assign_to)),
        },
        Core::Match { expr, cases } => Core::Match {
            expr: expr.clone(),
            cases: cases.iter().map(|c| append_assign(c, assign_to)).collect(),
        },
        Core::Case { expr, body } => Core::Case {
            expr: expr.clone(),
            body: Box::from(append_assign(body, assign_to)),
        },
        Core::TryExcept { setup, attempt, except } => Core::TryExcept {
            setup: setup.clone(),
            attempt: Box::from(append_assign(attempt, assign_to)),
            except: except.iter().map(|e| append_assign(e, assign_to)).collect(),
        },
        Core::Except { id, class, body } => Core::Except {
            id: id.clone(),
            class: class.clone(),
            body: Box::from(append_assign(body, assign_to)),
        },
        expr if skip_assign(expr) => core.clone(),
        _ => Core::Assign {
            left: Box::from(assign_to.clone()),
            right: Box::from(core.clone()),
            op: CoreOp::Assign,
        },
    }
}

fn append_ret(core: &Core) -> Core {
    match core {
        Core::Block { ref statements } => match statements.last() {
            Some(last) => {
                let last = append_ret(last);
                let (mut statements, idx): (Vec<Core>, usize) = (statements.clone(), statements.len() - 1);
                statements[idx] = last;
                Core::Block { statements }
            }
            None => Core::Block { statements: vec![Core::Return { expr: Box::from(Core::None) }] }
        }
        Core::IfElse { cond, then, el } => Core::IfElse {
            cond: cond.clone(),
            then: Box::from(append_ret(then)),
            el: Box::from(append_ret(el)),
        },
        Core::Match { expr, cases } => Core::Match {
            expr: expr.clone(),
            cases: cases.iter().map(append_ret).collect(),
        },
        Core::Case { expr, body } => Core::Case {
            expr: expr.clone(),
            body: Box::from(append_ret(body)),
        },
        Core::TryExcept { setup, attempt, except } => Core::TryExcept {
            setup: setup.clone(),
            attempt: Box::from(append_ret(attempt)),
            except: except.iter().map(append_ret).collect(),
        },
        Core::Except { id, class, body } => Core::Except {
            id: id.clone(),
            class: class.clone(),
            body: Box::from(append_ret(body)),
        },
        core if skip_return(core) => core.clone(),
        _ => Core::Return { expr: Box::from(core.clone()) }
    }
}

fn skip_assign(core: &Core) -> bool {
    skip_return(core) || matches!(core, Core::VarDef { .. } | Core::Assign { .. })
}

fn skip_return(core: &Core) -> bool {
    matches!(core, Core::Return { .. } | Core::Raise { .. })
}

#[cfg(test)]
mod tests {
    use crate::ASTTy;
    use crate::common::position::Position;
    use crate::generate::ast::node::Core;
    use crate::generate::gen;
    use crate::parse::ast::AST;
    use crate::parse::ast::Node;

    macro_rules! to_pos_unboxed {
        ($node:expr) => {{
            AST { pos: Position::default(), node: $node }
        }};
    }

    macro_rules! to_pos {
        ($node:expr) => {{
            Box::from(to_pos_unboxed!($node))
        }};
    }

    #[test]
    fn break_verify() {
        let _break = to_pos!(Node::Break);
        assert_eq!(gen(&ASTTy::from(&_break)).unwrap(), Core::Break);
    }

    #[test]
    fn continue_verify() {
        let _continue = to_pos!(Node::Continue);
        assert_eq!(gen(&ASTTy::from(&_continue)).unwrap(), Core::Continue);
    }

    #[test]
    fn pass_verify() {
        let pass = to_pos!(Node::Pass);
        assert_eq!(gen(&ASTTy::from(&pass)).unwrap(), Core::Pass);
    }

    #[test]
    fn return_verify() {
        let expr = to_pos!(Node::Str { lit: String::from("a"), expressions: vec![] });
        let print_stmt = to_pos!(Node::Return { expr });

        assert_eq!(
            gen(&ASTTy::from(&print_stmt)).unwrap(),
            Core::Return { expr: Box::from(Core::Str { string: String::from("a") }) }
        );
    }

    #[test]
    fn return_empty_verify() {
        let print_stmt = to_pos!(Node::ReturnEmpty);
        assert_eq!(
            gen(&ASTTy::from(&print_stmt)).unwrap(),
            Core::Return { expr: Box::from(Core::None) }
        );
    }

    #[test]
    fn import_verify() {
        let _break = to_pos!(Node::Import {
            from: None,
            import: vec![to_pos_unboxed!(Node::Id { lit: String::from("a") })],
            alias: vec![to_pos_unboxed!(Node::Id { lit: String::from("b") })]
        });

        assert_eq!(
            gen(&ASTTy::from(&_break)).unwrap(),
            Core::Import {
                from: None,
                import: vec![Core::Id { lit: String::from("a") }],
                alias: vec![Core::Id { lit: String::from("b") }],
            }
        );
    }

    #[test]
    fn raises_empty_verify() {
        let type_def = to_pos!(Node::Raises {
            expr_or_stmt: Box::from(to_pos!(Node::Id { lit: String::from("a") })),
            errors: vec![]
        });
        assert_eq!(gen(&ASTTy::from(&type_def)).unwrap(), Core::Id { lit: String::from("a") });
    }

    macro_rules! verify {
        ($ast:ident) => {{
            let left = Node::Id { lit: String::from("left") };
            let right = Node::Id { lit: String::from("right") };
            let add_node = to_pos!(Node::$ast { left: to_pos!(left), right: to_pos!(right) });

            let (left, right) = match gen(&ASTTy::from(&add_node)) {
                Ok(Core::$ast { left, right }) => (left, right),
                other => panic!("Expected binary operation but was {:?}", other),
            };

            assert_eq!(*left, Core::Id { lit: String::from("left") });
            assert_eq!(*right, Core::Id { lit: String::from("right") });
        }};
    }

    macro_rules! verify_unary {
        ($ast:ident) => {{
            let expr = to_pos!(Node::Id { lit: String::from("expression") });
            let add_node = to_pos!(Node::$ast { expr });

            let expr_des = match gen(&ASTTy::from(&add_node)) {
                Ok(Core::$ast { expr }) => expr,
                other => panic!("Expected unary operation but was {:?}", other),
            };

            assert_eq!(*expr_des, Core::Id { lit: String::from("expression") });
        }};
    }

    #[test]
    fn add_verify() {
        verify!(Add);
    }

    #[test]
    fn sub_verify() {
        verify!(Sub);
    }

    #[test]
    fn mul_verify() {
        verify!(Mul);
    }

    #[test]
    fn div_verify() {
        verify!(Div);
    }

    #[test]
    fn mod_verify() {
        verify!(Mod);
    }

    #[test]
    fn pow_verify() {
        verify!(Pow);
    }

    #[test]
    fn add_unary_verify() {
        verify_unary!(AddU);
    }

    #[test]
    fn sub_unary_verify() {
        verify_unary!(SubU);
    }

    #[test]
    fn sqrt_verify() {
        let expr = to_pos!(Node::Id { lit: String::from("expression") });
        let add_node = to_pos!(Node::Sqrt { expr });

        let (import, expr_des) = match gen(&ASTTy::from(&add_node)) {
            Ok(Core::Block { statements }) => (statements[0].clone(), statements[1].clone()),
            other => panic!("Expected unary operation but was {:?}", other),
        };

        assert_eq!(
            import,
            Core::Import {
                from: None,
                import: vec![Core::Id { lit: String::from("math") }],
                alias: vec![],
            }
        );
        assert_eq!(
            expr_des,
            Core::Sqrt { expr: Box::from(Core::Id { lit: String::from("expression") }) }
        );
    }

    #[test]
    fn le_verify() {
        verify!(Le);
    }

    #[test]
    fn leq_verify() {
        verify!(Leq);
    }

    #[test]
    fn ge_verify() {
        verify!(Ge);
    }

    #[test]
    fn geq_verify() {
        verify!(Geq);
    }

    #[test]
    fn neq_verify() {
        verify!(Neq);
    }

    #[test]
    fn is_verify() {
        verify!(Is);
    }

    #[test]
    fn not_verify() {
        verify_unary!(Not);
    }

    #[test]
    fn and_verify() {
        verify!(And);
    }

    #[test]
    fn or_verify() {
        verify!(Or);
    }

    #[test]
    fn tuple_verify() {
        let elements = vec![
            to_pos_unboxed!(Node::ENum { num: String::from("a"), exp: String::from("100") }),
            to_pos_unboxed!(Node::Real { lit: String::from("3000.5") }),
        ];
        let tuple = to_pos!(Node::Tuple { elements });
        let core = gen(&ASTTy::from(&tuple));

        let core_elements = match core {
            Ok(Core::Tuple { elements }) => elements,
            other => panic!("Expected tuple but got {:?}", other),
        };

        assert_eq!(
            core_elements[0],
            Core::ENum { num: String::from("a"), exp: String::from("100") }
        );
        assert_eq!(core_elements[1], Core::Float { float: String::from("3000.5") });
    }

    #[test]
    fn set_verify() {
        let elements = vec![
            to_pos_unboxed!(Node::Id { lit: String::from("a") }),
            to_pos_unboxed!(Node::Bool { lit: true }),
        ];
        let set = to_pos!(Node::Set { elements });
        let core = gen(&ASTTy::from(&set));

        let core_elements = match core {
            Ok(Core::Set { elements }) => elements,
            other => panic!("Expected set but got {:?}", other),
        };

        assert_eq!(core_elements[0], Core::Id { lit: String::from("a") });
        assert_eq!(core_elements[1], Core::Bool { boolean: true });
    }

    #[test]
    fn list_verify() {
        let elements = vec![
            to_pos_unboxed!(Node::ENum { num: String::from("a"), exp: String::from("100") }),
            to_pos_unboxed!(Node::Real { lit: String::from("3000.5") }),
        ];
        let tuple = to_pos!(Node::List { elements });
        let core = gen(&ASTTy::from(&tuple));

        let core_elements = match core {
            Ok(Core::List { elements }) => elements,
            other => panic!("Expected tuple but got {:?}", other),
        };

        assert_eq!(
            core_elements[0],
            Core::ENum { num: String::from("a"), exp: String::from("100") }
        );
        assert_eq!(core_elements[1], Core::Float { float: String::from("3000.5") });
    }

    #[test]
    fn set_builder_verify() {
        let item = to_pos!(Node::Id { lit: String::from("a") });
        let conditions = vec![];
        let list_builder = to_pos!(Node::SetBuilder { item, conditions });

        let desugar_result = gen(&ASTTy::from(&list_builder));
        assert!(desugar_result.is_err());
    }

    #[test]
    fn list_builder_verify() {
        let item = to_pos!(Node::Id { lit: String::from("a") });
        let conditions = vec![];
        let list_builder = to_pos!(Node::ListBuilder { item, conditions });

        let desugar_result = gen(&ASTTy::from(&list_builder));
        assert!(desugar_result.is_err());
    }

    #[test]
    fn with_verify() {
        let resource = to_pos!(Node::Id { lit: String::from("my_resource") });
        let alias = Some((to_pos!(Node::Id { lit: String::from("other") }), false, None));
        let expr = to_pos!(Node::Int { lit: String::from("9") });
        let with = to_pos!(Node::With { resource, alias, expr });

        let Ok(Core::WithAs { resource, alias, expr }) = gen(&ASTTy::from(&with)) else {
            panic!("Expected with as but was {:?}", gen(&ASTTy::from(&with)))
        };

        assert_eq!(*resource, Core::Id { lit: String::from("my_resource") });
        assert_eq!(*alias, Core::Id { lit: String::from("other") });
        assert_eq!(*expr, Core::Int { int: String::from("9") });
    }

    #[test]
    fn with_no_as_verify() {
        let resource = to_pos!(Node::Id { lit: String::from("other") });
        let expr = to_pos!(Node::Int { lit: String::from("2341") });
        let with = to_pos!(Node::With { resource, alias: None, expr });

        let (resource, expr) = match gen(&ASTTy::from(&with)) {
            Ok(Core::With { resource, expr }) => (resource, expr),
            other => panic!("Expected with but was {:?}", other),
        };

        assert_eq!(*resource, Core::Id { lit: String::from("other") });
        assert_eq!(*expr, Core::Int { int: String::from("2341") });
    }

    #[test]
    fn handle_empty_verify() {
        let expr_or_stmt = to_pos!(Node::Id { lit: String::from("my_fun") });
        let handle = to_pos!(Node::Handle { expr_or_stmt, cases: vec![] });

        let (setup, _try, except) = match gen(&ASTTy::from(&handle)) {
            Ok(Core::TryExcept { setup, attempt, except }) => {
                (setup.clone(), attempt.clone(), except.clone())
            }
            other => panic!("Expected try except but was {:?}", other),
        };

        assert_eq!(setup, None);
        assert_eq!(*_try, Core::Id { lit: String::from("my_fun") });
        assert!(except.is_empty());
    }

    #[test]
    fn handle_verify() {
        let expr_or_stmt = to_pos!(Node::Id { lit: String::from("my_fun") });
        let cond = to_pos!(Node::ExpressionType {
            expr: to_pos!(Node::Id { lit: String::from("err") }),
            mutable: false,
            ty: Some(to_pos!(Node::Type {
                id: to_pos!(Node::Id { lit: String::from("my_type") }),
                generics: vec![]
            }))
        });
        let body = to_pos!(Node::Int { lit: String::from("9999") });
        let case = to_pos_unboxed!(Node::Case { cond, body });
        let handle = to_pos!(Node::Handle { expr_or_stmt, cases: vec![case] });

        let Ok(Core::TryExcept { setup, attempt, except }) = gen(&ASTTy::from(&handle)) else {
            panic!("Expected try except but was {:?}", gen(&ASTTy::from(&handle)))
        };

        assert_eq!(setup, None);
        assert_eq!(*attempt, Core::Id { lit: String::from("my_fun") });
        assert_eq!(except.len(), 1);
        let Core::Except { id, class, body } = &except[0] else {
            panic!("Expected except case but was {:?}", except[0])
        };

        assert_eq!(*id, Box::from(Core::Id { lit: String::from("err") }));
        assert_eq!(
            *class,
            Some(Box::from(Core::Type { lit: String::from("my_type"), generics: vec![] }))
        );
        assert_eq!(*body, Box::from(Core::Int { int: String::from("9999") }));
    }
}

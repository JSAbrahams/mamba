use crate::check::context::clss;
use crate::check::context::clss::concrete_to_python;
use crate::generate::ast::node::{Core, CoreOp};
use crate::generate::convert::call::convert_call;
use crate::generate::convert::class::convert_class;
use crate::generate::convert::common::{convert_stmts, convert_vec};
use crate::generate::convert::control_flow::convert_cntrl_flow;
use crate::generate::convert::definition::convert_def;
use crate::generate::convert::handle::convert_handle;
use crate::generate::convert::state::{Imports, State};
use crate::generate::convert::ty::convert_ty;
use crate::generate::result::{GenResult, UnimplementedErr};
use crate::parse::ast::{AST, Node};
use crate::parse::ast::node_op::NodeOp;

mod call;
mod class;
mod common;
mod control_flow;
mod definition;
mod handle;
mod ty;

pub mod state;

pub fn convert_node(ast: &AST, imp: &mut Imports, state: &State) -> GenResult {
    let assign_to = state.assign_to.clone();
    let state = &state.assign_to(None);

    let core = match &ast.node {
        Node::Import { import, aliases } if aliases.is_empty() => {
            Core::Import { imports: convert_vec(import, imp, state)? }
        }
        Node::Import { import, aliases } => Core::ImportAs {
            imports: convert_vec(import, imp, state)?,
            aliases: convert_vec(aliases, imp, state)?,
        },
        Node::FromImport { id, import } => Core::FromImport {
            from: Box::from(convert_node(id, imp, state)?),
            import: Box::from(convert_node(import, imp, state)?),
        },

        Node::VariableDef { .. } | Node::FunDef { .. } | Node::FunArg { .. } => {
            convert_def(ast, imp, state)?
        }
        Node::Reassign { left, right, op } => Core::Assign {
            left: Box::from(convert_node(left, imp, state)?),
            right: Box::from(convert_node(right, imp, state)?),
            op: match op {
                NodeOp::Add => CoreOp::AddAssign,
                NodeOp::Sub => CoreOp::SubAssign,
                NodeOp::Mul => CoreOp::MulAssign,
                NodeOp::Div => CoreOp::DivAssign,
                NodeOp::Pow => CoreOp::PowAssign,
                NodeOp::BLShift => CoreOp::BLShiftAssign,
                NodeOp::BRShift => CoreOp::BRShiftAssign,
                NodeOp::Assign => CoreOp::Assign,
                op => {
                    return Err(UnimplementedErr::new(ast, &format!("Reassign with {}", op)));
                }
            },
        },

        Node::File { statements, .. } => {
            let mut modules = convert_vec(statements, imp, state)?;
            let mut statements = imp.imports.clone();
            statements.append(&mut modules);
            Core::Block { statements }
        }
        Node::Block { statements } => Core::Block {
            statements: convert_stmts(statements, imp, &state.assign_to(assign_to.as_ref()))?,
        },

        Node::Int { lit } => Core::Int { int: lit.clone() },
        Node::Real { lit } => Core::Float { float: lit.clone() },
        Node::ENum { num, exp } => Core::ENum {
            num: num.clone(),
            exp: if exp.is_empty() { String::from("0") } else { exp.clone() },
        },
        Node::DocStr { lit } => Core::DocStr { string: lit.clone() },
        Node::Str { lit, expressions } if expressions.is_empty() => {
            Core::Str { string: lit.clone() }
        }
        Node::Str { lit, .. } => Core::FStr { string: lit.clone() },

        Node::QuestionOp { .. } => convert_ty(ast, imp, state)?,

        Node::Undefined => Core::None,
        Node::ExpressionType { .. } => convert_ty(ast, imp, state)?,
        Node::Id { lit } => Core::Id { lit: concrete_to_python(lit) },
        Node::Bool { lit } => Core::Bool { boolean: *lit },

        Node::Tuple { elements } if state.tup_lit => {
            Core::TupleLiteral { elements: convert_vec(elements, imp, state)? }
        }
        Node::Tuple { elements } => Core::Tuple { elements: convert_vec(elements, imp, state)? },
        Node::List { elements } => Core::List { elements: convert_vec(elements, imp, state)? },
        Node::Set { elements } => Core::Set { elements: convert_vec(elements, imp, state)? },
        Node::Index { item, range } => Core::Index {
            item: Box::from(convert_node(item, imp, state)?),
            range: Box::from(convert_node(range, imp, state)?),
        },

        Node::ListBuilder { .. } => return Err(UnimplementedErr::new(ast, "list builder")),
        Node::SetBuilder { .. } => return Err(UnimplementedErr::new(ast, "set builder")),

        Node::ReturnEmpty => Core::Return { expr: Box::from(Core::None) },
        Node::Return { expr } => Core::Return { expr: Box::from(convert_node(expr, imp, state)?) },

        Node::IfElse { .. }
        | Node::While { .. }
        | Node::For { .. }
        | Node::Break
        | Node::Continue => convert_cntrl_flow(ast, imp, state)?,
        Node::Match { .. } => convert_cntrl_flow(ast, imp, &state.expand_ty(false))?,

        Node::Not { expr } => Core::Not { expr: Box::from(convert_node(expr, imp, state)?) },
        Node::And { left, right } => Core::And {
            left: Box::from(convert_node(left, imp, state)?),
            right: Box::from(convert_node(right, imp, state)?),
        },
        Node::Or { left, right } => Core::Or {
            left: Box::from(convert_node(left, imp, state)?),
            right: Box::from(convert_node(right, imp, state)?),
        },
        Node::Is { left, right } => Core::Is {
            left: Box::from(convert_node(left, imp, state)?),
            right: Box::from(convert_node(right, imp, state)?),
        },
        Node::IsN { left, right } => Core::IsN {
            left: Box::from(convert_node(left, imp, state)?),
            right: Box::from(convert_node(right, imp, state)?),
        },
        Node::Eq { left, right } => Core::Eq {
            left: Box::from(convert_node(left, imp, state)?),
            right: Box::from(convert_node(right, imp, state)?),
        },
        Node::Neq { left, right } => Core::Neq {
            left: Box::from(convert_node(left, imp, state)?),
            right: Box::from(convert_node(right, imp, state)?),
        },
        Node::IsA { left, right } => Core::IsA {
            left: Box::from(convert_node(left, imp, state)?),
            right: Box::from(convert_node(right, imp, state)?),
        },
        Node::IsNA { left, right } => Core::Not {
            expr: Box::from(Core::IsA {
                left: Box::from(convert_node(left, imp, state)?),
                right: Box::from(convert_node(right, imp, state)?),
            }),
        },

        Node::Add { left, right } => Core::Add {
            left: Box::from(convert_node(left, imp, state)?),
            right: Box::from(convert_node(right, imp, state)?),
        },
        Node::Sub { left, right } => Core::Sub {
            left: Box::from(convert_node(left, imp, state)?),
            right: Box::from(convert_node(right, imp, state)?),
        },
        Node::Mul { left, right } => Core::Mul {
            left: Box::from(convert_node(left, imp, state)?),
            right: Box::from(convert_node(right, imp, state)?),
        },
        Node::Div { left, right } => Core::Div {
            left: Box::from(convert_node(left, imp, state)?),
            right: Box::from(convert_node(right, imp, state)?),
        },
        Node::FDiv { left, right } => Core::FDiv {
            left: Box::from(convert_node(left, imp, state)?),
            right: Box::from(convert_node(right, imp, state)?),
        },
        Node::Mod { left, right } => Core::Mod {
            left: Box::from(convert_node(left, imp, state)?),
            right: Box::from(convert_node(right, imp, state)?),
        },
        Node::Pow { left, right } => Core::Pow {
            left: Box::from(convert_node(left, imp, state)?),
            right: Box::from(convert_node(right, imp, state)?),
        },

        Node::BAnd { left, right } => Core::BAnd {
            left: Box::from(convert_node(left, imp, state)?),
            right: Box::from(convert_node(right, imp, state)?),
        },
        Node::BOr { left, right } => Core::BOr {
            left: Box::from(convert_node(left, imp, state)?),
            right: Box::from(convert_node(right, imp, state)?),
        },
        Node::BXOr { left, right } => Core::BXOr {
            left: Box::from(convert_node(left, imp, state)?),
            right: Box::from(convert_node(right, imp, state)?),
        },
        Node::BOneCmpl { expr } => {
            Core::BOneCmpl { expr: Box::from(convert_node(expr, imp, state)?) }
        }
        Node::BLShift { left, right } => Core::BLShift {
            left: Box::from(convert_node(left, imp, state)?),
            right: Box::from(convert_node(right, imp, state)?),
        },
        Node::BRShift { left, right } => Core::BRShift {
            left: Box::from(convert_node(left, imp, state)?),
            right: Box::from(convert_node(right, imp, state)?),
        },

        Node::AddU { expr } => Core::AddU { expr: Box::from(convert_node(expr, imp, state)?) },
        Node::SubU { expr } => Core::SubU { expr: Box::from(convert_node(expr, imp, state)?) },
        Node::Sqrt { expr } => {
            imp.add_import("math");
            Core::Sqrt { expr: Box::from(convert_node(expr, imp, state)?) }
        }

        Node::Le { left, right } => Core::Le {
            left: Box::from(convert_node(left, imp, state)?),
            right: Box::from(convert_node(right, imp, state)?),
        },
        Node::Leq { left, right } => Core::Leq {
            left: Box::from(convert_node(left, imp, state)?),
            right: Box::from(convert_node(right, imp, state)?),
        },
        Node::Ge { left, right } => Core::Ge {
            left: Box::from(convert_node(left, imp, state)?),
            right: Box::from(convert_node(right, imp, state)?),
        },
        Node::Geq { left, right } => Core::Geq {
            left: Box::from(convert_node(left, imp, state)?),
            right: Box::from(convert_node(right, imp, state)?),
        },

        Node::FunctionCall { .. } | Node::PropertyCall { .. } => convert_call(ast, imp, state)?,
        Node::AnonFun { args, body } => Core::AnonFun {
            args: convert_vec(args, imp, &state.expand_ty(false))?,
            body: Box::from(convert_node(body, imp, state)?),
        },

        Node::In { left, right } => Core::In {
            left: Box::from(convert_node(left, imp, state)?),
            right: Box::from(convert_node(right, imp, state)?),
        },
        Node::Range { from, to, inclusive, step } => Core::FunctionCall {
            function: Box::from(Core::Id { lit: String::from(clss::python::RANGE) }),
            args: vec![
                convert_node(from, imp, state)?,
                if *inclusive {
                    Core::Add {
                        left: Box::from(convert_node(to, imp, state)?),
                        right: Box::from(Core::Int { int: String::from("1") }),
                    }
                } else {
                    convert_node(to, imp, state)?
                },
                if let Some(step) = step {
                    convert_node(step, imp, state)?
                } else {
                    Core::Int { int: String::from("1") }
                },
            ],
        },
        Node::Slice { from, to, inclusive, step } => Core::FunctionCall {
            function: Box::from(Core::Id { lit: String::from(clss::python::SLICE) }),
            args: vec![
                convert_node(from, imp, state)?,
                if !inclusive {
                    Core::Sub {
                        left: Box::from(convert_node(to, imp, state)?),
                        right: Box::from(Core::Int { int: String::from("1") }),
                    }
                } else {
                    convert_node(to, imp, state)?
                },
                if let Some(step) = step {
                    convert_node(step, imp, state)?
                } else {
                    Core::Int { int: String::from("1") }
                },
            ],
        },
        Node::Underscore => Core::UnderScore,
        Node::Question { left, right } => Core::Or {
            left: Box::from(convert_node(left, imp, state)?),
            right: Box::from(convert_node(right, imp, state)?),
        },

        Node::TypeTup { .. }
        | Node::Type { .. }
        | Node::TypeFun { .. }
        | Node::TypeUnion { .. } => convert_ty(ast, imp, state)?,

        Node::TypeDef { .. } | Node::TypeAlias { .. } => convert_class(ast, imp, state)?,
        Node::Class { .. } => convert_class(ast, imp, state)?,
        Node::Generic { .. } => Core::Empty,
        Node::Parent { .. } => convert_class(ast, imp, state)?,

        Node::Condition { .. } => return Err(UnimplementedErr::new(ast, "condition")),

        Node::Comment { comment } => Core::Comment { comment: comment.clone() },
        Node::Pass => Core::Pass,

        Node::With { resource, alias: Some((alias, ..)), expr } => Core::WithAs {
            resource: Box::from(convert_node(resource, imp, state)?),
            alias: Box::from(convert_node(alias, imp, &state.expand_ty(false))?),
            expr: Box::from(convert_node(expr, imp, state)?),
        },
        Node::With { resource, expr, .. } => Core::With {
            resource: Box::from(convert_node(resource, imp, state)?),
            expr: Box::from(convert_node(expr, imp, state)?),
        },

        Node::Raises { .. } | Node::Raise { .. } | Node::Handle { .. } => {
            convert_handle(ast, imp, state)?
        }

        _ => {
            let msg = format!("Cannot have top level: {}", ast.node);
            return Err(UnimplementedErr::new(ast, &msg));
        }
    };

    let core = if let Some(assign_to) = assign_to {
        match core {
            Core::Block { .. } | Core::Return { .. } => core,
            expr => Core::Assign {
                left: Box::from(assign_to),
                right: Box::from(expr),
                op: CoreOp::Assign,
            },
        }
    } else {
        core
    };

    Ok(core)
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
        assert_eq!(gen(&ASTTy::from(&print_stmt)).unwrap(), Core::Return { expr: Box::from(Core::None) });
    }

    #[test]
    fn import_verify() {
        let _break = to_pos!(Node::Import {
            import: vec![to_pos_unboxed!(Node::Id { lit: String::from("a") })],
            aliases: vec![to_pos_unboxed!(Node::Id { lit: String::from("b") })]
        });

        assert_eq!(
            gen(&ASTTy::from(&_break)).unwrap(),
            Core::ImportAs {
                imports: vec![Core::Id { lit: String::from("a") }],
                aliases: vec![Core::Id { lit: String::from("b") }],
            }
        );
    }

    #[test]
    fn from_import_as_verify() {
        let _break = to_pos!(Node::FromImport {
            id: to_pos!(Node::Id { lit: String::from("f") }),
            import: to_pos!(Node::Import {
                import: vec![to_pos_unboxed!(Node::Id { lit: String::from("a") })],
                aliases: vec![to_pos_unboxed!(Node::Id { lit: String::from("b") })]
            })
        });

        assert_eq!(
            gen(&ASTTy::from(&_break)).unwrap(),
            Core::FromImport {
                from: Box::from(Core::Id { lit: String::from("f") }),
                import: Box::from(Core::ImportAs {
                    imports: vec![Core::Id { lit: String::from("a") }],
                    aliases: vec![Core::Id { lit: String::from("b") }],
                }),
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
        verify_unary!(Sqrt);
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

        let (resource, alias, expr) = match gen(&ASTTy::from(&with)) {
            Ok(Core::WithAs { resource, alias, expr }) => (resource, alias, expr),
            other => panic!("Expected with as but was {:?}", other),
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

        let (setup, _try, except) = match gen(&ASTTy::from(&handle)) {
            Ok(Core::TryExcept { setup, attempt, except }) => {
                (setup.clone(), attempt.clone(), except.clone())
            }
            other => panic!("Expected try except but was {:?}", other),
        };

        assert_eq!(setup, None);
        assert_eq!(*_try, Core::Id { lit: String::from("my_fun") });
        assert_eq!(except.len(), 1);
        match &except[0] {
            Core::Except { id, class, body } => {
                assert_eq!(*id, Box::from(Core::Id { lit: String::from("err") }));
                assert_eq!(
                    *class,
                    Some(Box::from(Core::Type { lit: String::from("my_type"), generics: vec![] }))
                );
                assert_eq!(*body, Box::from(Core::Int { int: String::from("9999") }));
            }
            other => panic!("Expected except case but was {:?}", other),
        }
    }
}

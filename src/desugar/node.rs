use crate::check::context::clss::concrete_to_python;
use crate::core::construct::Core;
use crate::desugar::call::desugar_call;
use crate::desugar::class::desugar_class;
use crate::desugar::common::{desugar_stmts, desugar_vec};
use crate::desugar::control_flow::desugar_control_flow;
use crate::desugar::definition::desugar_definition;
use crate::desugar::result::DesugarResult;
use crate::desugar::result::UnimplementedErr;
use crate::desugar::state::Imports;
use crate::desugar::state::State;
use crate::desugar::ty::desugar_type;
use crate::parse::ast::AST;
use crate::parse::ast::Node;

// TODO return imports instead of modifying mutable reference
pub fn desugar_node(ast: &AST, imp: &mut Imports, state: &State) -> DesugarResult {
    let assign_to = state.assign_to.clone();
    let state = &state.assign_to(None);

    let core = match &ast.node {
        Node::Import { import, aliases: _as } => {
            if _as.is_empty() {
                Core::Import { imports: desugar_vec(import, imp, state)? }
            } else {
                Core::ImportAs {
                    imports: desugar_vec(import, imp, state)?,
                    alias: desugar_vec(_as, imp, state)?,
                }
            }
        }
        Node::FromImport { id, import } => Core::FromImport {
            from: Box::from(desugar_node(id, imp, state)?),
            import: Box::from(desugar_node(import, imp, state)?),
        },

        Node::VariableDef { .. } | Node::FunDef { .. } => desugar_definition(ast, imp, state)?,
        Node::Reassign { left, right } => Core::Assign {
            left: Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?),
        },

        Node::File { statements, .. } => {
            let mut modules = desugar_vec(statements, imp, state)?;
            let mut statements = imp.imports.clone();
            statements.append(&mut modules);
            Core::Block { statements }
        }
        Node::Block { statements } => Core::Block {
            statements: desugar_stmts(statements, imp, &state.assign_to(assign_to.as_ref()))?,
        },

        Node::Int { lit } => Core::Int { int: lit.clone() },
        Node::Real { lit } => Core::Float { float: lit.clone() },
        Node::ENum { num, exp } => Core::ENum {
            num: num.clone(),
            exp: if exp.is_empty() { String::from("0") } else { exp.clone() },
        },
        Node::DocStr { lit } => Core::DocStr { string: lit.clone() },
        Node::Str { lit, expressions } => {
            if expressions.is_empty() {
                Core::Str { string: lit.clone() }
            } else {
                Core::FStr { string: lit.clone() }
            }
        }

        Node::AddOp => Core::AddOp,
        Node::SubOp => Core::SubOp,
        Node::SqrtOp => Core::Id { lit: String::from("sqrt") },
        Node::MulOp => Core::MulOp,
        Node::FDivOp => Core::FDivOp,
        Node::DivOp => Core::DivOp,
        Node::PowOp => Core::PowOp,
        Node::ModOp => Core::ModOp,
        Node::EqOp => Core::EqOp,
        Node::LeOp => Core::LeOp,
        Node::GeOp => Core::GeOp,
        Node::QuestionOp { .. } => desugar_type(ast, imp, state)?,

        Node::Undefined => Core::None,
        Node::ExpressionType { .. } => desugar_type(ast, imp, state)?,
        Node::Id { lit } => Core::Id { lit: concrete_to_python(lit) },
        Node::_Self => Core::Id { lit: String::from("self") },
        Node::Init => Core::Id { lit: String::from("init") },
        Node::Bool { lit } => Core::Bool { boolean: *lit },

        Node::Tuple { elements } => if state.tup_lit {
            Core::TupleLiteral { elements: desugar_vec(elements, imp, state)? }
        } else {
            Core::Tuple { elements: desugar_vec(elements, imp, state)? }
        },
        Node::List { elements } => Core::List { elements: desugar_vec(elements, imp, state)? },
        Node::Set { elements } => Core::Set { elements: desugar_vec(elements, imp, state)? },
        Node::Index { item, range } => Core::Index {
            item: Box::from(desugar_node(item, imp, state)?),
            range: Box::from(desugar_node(range, imp, state)?),
        },

        Node::ListBuilder { .. } => return Err(UnimplementedErr::new(ast, "list builder")),
        Node::SetBuilder { .. } => return Err(UnimplementedErr::new(ast, "set builder")),

        Node::ReturnEmpty => Core::Return { expr: Box::from(Core::None) },
        Node::Return { expr } => Core::Return { expr: Box::from(desugar_node(expr, imp, state)?) },
        Node::Print { expr } => Core::Print { expr: Box::from(desugar_node(expr, imp, state)?) },

        Node::IfElse { .. }
        | Node::While { .. }
        | Node::For { .. }
        | Node::Break
        | Node::Continue => desugar_control_flow(ast, imp, state)?,
        Node::Match { .. } => desugar_control_flow(ast, imp, &state.expand_ty(false))?,
        Node::Case { .. } => panic!("Case cannot be top-level"),

        Node::Not { expr } => Core::Not { expr: Box::from(desugar_node(expr, imp, state)?) },
        Node::And { left, right } => Core::And {
            left: Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?),
        },
        Node::Or { left, right } => Core::Or {
            left: Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?),
        },
        Node::Is { left, right } => Core::Is {
            left: Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?),
        },
        Node::IsN { left, right } => Core::IsN {
            left: Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?),
        },
        Node::Eq { left, right } => Core::Eq {
            left: Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?),
        },
        Node::Neq { left, right } => Core::Neq {
            left: Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?),
        },
        Node::IsA { left, right } => Core::IsA {
            left: Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?),
        },
        Node::IsNA { left, right } => Core::Not {
            expr: Box::from(Core::IsA {
                left: Box::from(desugar_node(left, imp, state)?),
                right: Box::from(desugar_node(right, imp, state)?),
            }),
        },

        Node::Add { left, right } => Core::Add {
            left: Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?),
        },
        Node::Sub { left, right } => Core::Sub {
            left: Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?),
        },
        Node::Mul { left, right } => Core::Mul {
            left: Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?),
        },
        Node::Div { left, right } => Core::Div {
            left: Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?),
        },
        Node::FDiv { left, right } => Core::FDiv {
            left: Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?),
        },
        Node::Mod { left, right } => Core::Mod {
            left: Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?),
        },
        Node::Pow { left, right } => Core::Pow {
            left: Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?),
        },

        Node::BAnd { left, right } => Core::BAnd {
            left: Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?),
        },
        Node::BOr { left, right } => Core::BOr {
            left: Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?),
        },
        Node::BXOr { left, right } => Core::BXOr {
            left: Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?),
        },
        Node::BOneCmpl { expr } => {
            Core::BOneCmpl { expr: Box::from(desugar_node(expr, imp, state)?) }
        }
        Node::BLShift { left, right } => Core::BLShift {
            left: Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?),
        },
        Node::BRShift { left, right } => Core::BRShift {
            left: Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?),
        },

        Node::AddU { expr } => Core::AddU { expr: Box::from(desugar_node(expr, imp, state)?) },
        Node::SubU { expr } => Core::SubU { expr: Box::from(desugar_node(expr, imp, state)?) },
        Node::Sqrt { expr } => {
            imp.add_import("math");
            Core::Sqrt { expr: Box::from(desugar_node(expr, imp, state)?) }
        }

        Node::Le { left, right } => Core::Le {
            left: Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?),
        },
        Node::Leq { left, right } => Core::Leq {
            left: Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?),
        },
        Node::Ge { left, right } => Core::Ge {
            left: Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?),
        },
        Node::Geq { left, right } => Core::Geq {
            left: Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?),
        },

        Node::FunArg { vararg, var, ty, default, .. } => Core::FunArg {
            vararg: *vararg,
            var: Box::from(desugar_node(var, imp, state)?),
            ty: if state.expand_ty {
                match ty {
                    Some(ty) => match &var.node {
                        Node::_Self => None,
                        _ => Some(Box::from(desugar_node(ty, imp, state)?)),
                    },
                    None => None,
                }
            } else {
                None
            },
            default: match default {
                Some(default) => Some(Box::from(desugar_node(default, imp, state)?)),
                None => None,
            },
        },

        Node::FunctionCall { .. } | Node::PropertyCall { .. } => desugar_call(ast, imp, state)?,
        Node::AnonFun { args, body } => Core::AnonFun {
            args: desugar_vec(args, imp, &state.expand_ty(false))?,
            body: Box::from(desugar_node(body, imp, state)?),
        },

        Node::In { left, right } => Core::In {
            left: Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?),
        },
        Node::Range { from, to, inclusive, step } => {
            let from = Box::from(desugar_node(from, imp, state)?);
            let to = Box::from(if *inclusive {
                Core::Add {
                    left: Box::from(desugar_node(to, imp, state)?),
                    right: Box::from(Core::Int { int: String::from("1") }),
                }
            } else {
                desugar_node(to, imp, state)?
            });
            let step = Box::from(if let Some(step) = step {
                desugar_node(step, imp, state)?
            } else {
                Core::Int { int: String::from("1") }
            });

            Core::Range { from, to, step }
        }
        Node::Slice { from, to, inclusive, step } => {
            let from = Box::from(desugar_node(from, imp, state)?);
            let to = Box::from(if !inclusive {
                Core::Sub {
                    left: Box::from(desugar_node(to, imp, state)?),
                    right: Box::from(Core::Int { int: String::from("1") }),
                }
            } else {
                desugar_node(to, imp, state)?
            });
            let step = Box::from(if let Some(step) = step {
                desugar_node(step, imp, state)?
            } else {
                Core::Int { int: String::from("1") }
            });

            Core::Slice { from, to, step }
        }
        Node::Underscore => Core::UnderScore,
        Node::Question { left, right } => Core::Or {
            left: Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?),
        },

        Node::TypeTup { .. }
        | Node::Type { .. }
        | Node::TypeFun { .. }
        | Node::TypeUnion { .. } => desugar_type(ast, imp, state)?,

        Node::TypeDef { .. } | Node::TypeAlias { .. } => desugar_class(ast, imp, state)?,
        Node::Class { .. } => desugar_class(ast, imp, state)?,
        Node::Generic { .. } => Core::Empty,
        Node::Parent { .. } => desugar_class(ast, imp, state)?,

        Node::Condition { .. } => return Err(UnimplementedErr::new(ast, "condition")),

        Node::Comment { comment } => Core::Comment { comment: comment.clone() },
        Node::Pass => Core::Pass,

        Node::With { resource, alias: Some((alias, ..)), expr } => Core::WithAs {
            resource: Box::from(desugar_node(resource, imp, state)?),
            alias: Box::from(desugar_node(alias, imp, &state.expand_ty(false))?),
            expr: Box::from(desugar_node(expr, imp, state)?),
        },
        Node::With { resource, expr, .. } => Core::With {
            resource: Box::from(desugar_node(resource, imp, state)?),
            expr: Box::from(desugar_node(expr, imp, state)?),
        },

        Node::Step { .. } => panic!("Step cannot be top level."),
        Node::Raises { expr_or_stmt, .. } => desugar_node(expr_or_stmt, imp, state)?,
        Node::Raise { error } => Core::Raise { error: Box::from(desugar_node(error, imp, state)?) },

        Node::Handle { expr_or_stmt, cases } => {
            let (var, ty) = if let Node::VariableDef { var, ty, .. } = &expr_or_stmt.node {
                (
                    Some(Box::from(desugar_node(var, imp, state)?)),
                    if let Some(ty) = ty {
                        Some(Box::from(desugar_node(ty, imp, state)?))
                    } else {
                        None
                    },
                )
            } else {
                (None, None)
            };
            let assign_state = state.assign_to(var.as_deref());

            Core::TryExcept {
                setup: var.map(|var| Box::from(Core::VarDef { var, ty, expr: None })),
                attempt: Box::from(desugar_node(&expr_or_stmt.clone(), imp, state)?),
                except: {
                    let mut except = Vec::new();
                    for case in cases {
                        let (cond, body) = match &case.node {
                            Node::Case { cond, body } => (cond, body),
                            other => panic!("Expected case but was {:?}", other),
                        };

                        match &cond.node {
                            Node::ExpressionType { expr, ty, .. } => except.push(Core::Except {
                                id: Box::from(desugar_node(expr, imp, state)?),
                                class: if let Some(ty) = ty {
                                    Some(Box::from(desugar_node(ty, imp, state)?))
                                } else {
                                    None
                                },
                                body: Box::from(desugar_node(body, imp, &assign_state)?),
                            }),
                            other => panic!("Expected id type but was {:?}", other),
                        };
                    }
                    except
                },
            }
        }
    };

    let core = if let Some(assign_to) = assign_to {
        match core {
            Core::Block { .. } | Core::Return { .. } => core,
            expr => Core::Assign { left: Box::from(assign_to), right: Box::from(expr) },
        }
    } else {
        core
    };

    Ok(core)
}

#[cfg(test)]
mod tests {
    use crate::common::position::Position;
    use crate::core::construct::Core;
    use crate::desugar::desugar;
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
        assert_eq!(desugar(&_break).unwrap(), Core::Break);
    }

    #[test]
    fn continue_verify() {
        let _continue = to_pos!(Node::Continue);
        assert_eq!(desugar(&_continue).unwrap(), Core::Continue);
    }

    #[test]
    fn pass_verify() {
        let pass = to_pos!(Node::Pass);
        assert_eq!(desugar(&pass).unwrap(), Core::Pass);
    }

    #[test]
    fn print_verify() {
        let expr = to_pos!(Node::Str { lit: String::from("a"), expressions: vec![] });
        let print_stmt = to_pos!(Node::Print { expr });
        assert_eq!(
            desugar(&print_stmt).unwrap(),
            Core::Print { expr: Box::from(Core::Str { string: String::from("a") }) }
        );
    }

    #[test]
    fn return_verify() {
        let expr = to_pos!(Node::Str { lit: String::from("a"), expressions: vec![] });
        let print_stmt = to_pos!(Node::Return { expr });

        assert_eq!(
            desugar(&print_stmt).unwrap(),
            Core::Return { expr: Box::from(Core::Str { string: String::from("a") }) }
        );
    }

    #[test]
    fn return_empty_verify() {
        let print_stmt = to_pos!(Node::ReturnEmpty);
        assert_eq!(desugar(&print_stmt).unwrap(), Core::Return { expr: Box::from(Core::None) });
    }

    #[test]
    fn init_verify() {
        let _break = to_pos!(Node::Init);
        assert_eq!(desugar(&_break).unwrap(), Core::Id { lit: String::from("init") });
    }

    #[test]
    fn self_verify() {
        let _break = to_pos!(Node::_Self);
        assert_eq!(desugar(&_break).unwrap(), Core::Id { lit: String::from("self") });
    }

    #[test]
    fn import_verify() {
        let _break = to_pos!(Node::Import {
            import: vec![to_pos_unboxed!(Node::Id { lit: String::from("a") })],
            aliases: vec![to_pos_unboxed!(Node::Id { lit: String::from("b") })]
        });

        assert_eq!(
            desugar(&_break).unwrap(),
            Core::ImportAs {
                imports: vec![Core::Id { lit: String::from("a") }],
                alias: vec![Core::Id { lit: String::from("b") }],
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
            desugar(&_break).unwrap(),
            Core::FromImport {
                from: Box::from(Core::Id { lit: String::from("f") }),
                import: Box::from(Core::ImportAs {
                    imports: vec![Core::Id { lit: String::from("a") }],
                    alias: vec![Core::Id { lit: String::from("b") }],
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
        assert_eq!(desugar(&type_def).unwrap(), Core::Id { lit: String::from("a") });
    }

    macro_rules! verify_op {
        ($op:ident) => {{
            let add_op = to_pos!(Node::$op);
            let core = desugar(&add_op).unwrap();
            assert_eq!(core, Core::$op);
        }};
    }

    macro_rules! verify {
        ($ast:ident) => {{
            let left = Node::Id { lit: String::from("left") };
            let right = Node::Id { lit: String::from("right") };
            let add_node = to_pos!(Node::$ast { left: to_pos!(left), right: to_pos!(right) });

            let (left, right) = match desugar(&add_node) {
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

            let expr_des = match desugar(&add_node) {
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
    fn add_op_verify() {
        verify_op!(AddOp);
    }

    #[test]
    fn sub_op_verify() {
        verify_op!(SubOp);
    }

    #[test]
    fn sqrt_op_verify() {
        let sqrt_node = to_pos!(Node::SqrtOp);
        let result = desugar(&sqrt_node);
        assert!(result.is_ok());
    }

    #[test]
    fn mul_op_verify() {
        verify_op!(MulOp);
    }

    #[test]
    fn div_op_verify() {
        verify_op!(DivOp);
    }

    #[test]
    fn pow_op_verify() {
        verify_op!(PowOp);
    }

    #[test]
    fn mod_op_verify() {
        verify_op!(ModOp);
    }

    #[test]
    fn eq_op_verify() {
        verify_op!(EqOp);
    }

    #[test]
    fn le_op_verify() {
        verify_op!(LeOp);
    }

    #[test]
    fn ge_op_verify() {
        verify_op!(GeOp);
    }

    #[test]
    fn tuple_verify() {
        let elements = vec![
            to_pos_unboxed!(Node::ENum { num: String::from("a"), exp: String::from("100") }),
            to_pos_unboxed!(Node::Real { lit: String::from("3000.5") }),
        ];
        let tuple = to_pos!(Node::Tuple { elements });
        let core = desugar(&tuple);

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
        let core = desugar(&set);

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
        let core = desugar(&tuple);

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

        let desugar_result = desugar(&list_builder);
        assert!(desugar_result.is_err());
    }

    #[test]
    fn list_builder_verify() {
        let item = to_pos!(Node::Id { lit: String::from("a") });
        let conditions = vec![];
        let list_builder = to_pos!(Node::ListBuilder { item, conditions });

        let desugar_result = desugar(&list_builder);
        assert!(desugar_result.is_err());
    }

    #[test]
    fn with_verify() {
        let resource = to_pos!(Node::Id { lit: String::from("my_resource") });
        let alias = Some((to_pos!(Node::Id { lit: String::from("other") }), false, None));
        let expr = to_pos!(Node::Int { lit: String::from("9") });
        let with = to_pos!(Node::With { resource, alias, expr });

        let (resource, alias, expr) = match desugar(&with) {
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

        let (resource, expr) = match desugar(&with) {
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

        let (setup, _try, except) = match desugar(&handle) {
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

        let (setup, _try, except) = match desugar(&handle) {
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

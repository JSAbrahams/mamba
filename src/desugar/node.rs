use crate::core::construct::Core;
use crate::desugar::call::desugar_call;
use crate::desugar::common::desugar_vec;
use crate::desugar::context::Context;
use crate::desugar::context::State;
use crate::desugar::control_flow::desugar_control_flow;
use crate::desugar::definition::desugar_definition;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use std::ops::Deref;

pub fn desugar_node(node_pos: &ASTNodePos, ctx: &Context, state: &State) -> Core {
    match &node_pos.node {
        ASTNode::Import { import, _as } => match _as.len() {
            0 => Core::Import { import: desugar_vec(import, ctx, state) },
            _ => Core::ImportAs {
                import: desugar_vec(import, ctx, state),
                _as:    desugar_vec(_as, ctx, state)
            }
        },
        ASTNode::FromImport { id, import } => Core::FromImport {
            from:   Box::from(desugar_node(id, ctx, state)),
            import: Box::from(desugar_node(import, ctx, state))
        },

        definition @ ASTNode::Def { .. } => desugar_definition(definition, ctx, state),
        ASTNode::Reassign { left, right } => Core::Assign {
            left:  Box::from(desugar_node(left, ctx, state)),
            right: Box::from(desugar_node(right, ctx, state))
        },

        ASTNode::Block { statements } =>
            Core::Block { statements: desugar_vec(statements, ctx, state) },

        ASTNode::Int { lit } => Core::Int { int: lit.clone() },
        ASTNode::Real { lit } => Core::Float { float: lit.clone() },
        ASTNode::ENum { num, exp } => Core::ENum {
            num: num.clone(),
            exp: if exp.is_empty() { String::from("0") } else { exp.clone() }
        },
        ASTNode::Str { lit } => Core::Str { _str: lit.clone() },

        ASTNode::AddOp => Core::AddOp,
        ASTNode::SubOp => Core::SubOp,
        ASTNode::SqrtOp => unimplemented!("sqrt"),
        ASTNode::MulOp => Core::MulOp,
        ASTNode::DivOp => Core::DivOp,
        ASTNode::PowOp => Core::PowOp,
        ASTNode::ModOp => Core::ModOp,
        ASTNode::EqOp => Core::EqOp,
        ASTNode::LeOp => Core::LeOp,
        ASTNode::GeOp => Core::GeOp,

        ASTNode::IdType { id, .. } => desugar_node(id, ctx, state),
        ASTNode::Id { lit } => Core::Id { lit: lit.clone() },
        ASTNode::_Self => Core::Id { lit: String::from("self") },
        ASTNode::Init => Core::Id { lit: String::from("init") },
        ASTNode::Bool { lit } => Core::Bool { _bool: *lit },

        ASTNode::Tuple { elements } => Core::Tuple { elements: desugar_vec(elements, ctx, state) },
        ASTNode::List { elements } => Core::List { elements: desugar_vec(elements, ctx, state) },
        ASTNode::Set { elements } => Core::Set { elements: desugar_vec(elements, ctx, state) },

        ASTNode::ListBuilder { .. } => unimplemented!("list builder"),
        ASTNode::SetBuilder { .. } => unimplemented!("set builder"),

        ASTNode::ReturnEmpty => Core::Return { expr: Box::from(Core::None) },
        ASTNode::Return { expr } =>
            Core::Return { expr: Box::from(desugar_node(expr, ctx, state)) },
        ASTNode::Print { expr } => Core::Print { expr: Box::from(desugar_node(expr, ctx, state)) },

        control_flow @ ASTNode::IfElse { .. }
        | control_flow @ ASTNode::Match { .. }
        | control_flow @ ASTNode::Case { .. }
        | control_flow @ ASTNode::While { .. }
        | control_flow @ ASTNode::For { .. }
        | control_flow @ ASTNode::Break
        | control_flow @ ASTNode::Continue => desugar_control_flow(control_flow, ctx, state),

        ASTNode::Not { expr } => Core::Not { expr: Box::from(desugar_node(expr, ctx, state)) },
        ASTNode::And { left, right } => Core::And {
            left:  Box::from(desugar_node(left, ctx, state)),
            right: Box::from(desugar_node(right, ctx, state))
        },
        ASTNode::Or { left, right } => Core::Or {
            left:  Box::from(desugar_node(left, ctx, state)),
            right: Box::from(desugar_node(right, ctx, state))
        },
        ASTNode::Is { left, right } => Core::Is {
            left:  Box::from(desugar_node(left, ctx, state)),
            right: Box::from(desugar_node(right, ctx, state))
        },
        ASTNode::IsN { left, right } => Core::IsN {
            left:  Box::from(desugar_node(left, ctx, state)),
            right: Box::from(desugar_node(right, ctx, state))
        },
        ASTNode::Eq { left, right } => Core::Eq {
            left:  Box::from(desugar_node(left, ctx, state)),
            right: Box::from(desugar_node(right, ctx, state))
        },
        ASTNode::Neq { left, right } => Core::Neq {
            left:  Box::from(desugar_node(left, ctx, state)),
            right: Box::from(desugar_node(right, ctx, state))
        },
        ASTNode::IsA { left, right } => Core::IsA {
            left:  Box::from(desugar_node(left, ctx, state)),
            right: Box::from(desugar_node(right, ctx, state))
        },
        ASTNode::IsNA { left, right } => Core::Not {
            expr: Box::from(Core::IsA {
                left:  Box::from(desugar_node(left, ctx, state)),
                right: Box::from(desugar_node(right, ctx, state))
            })
        },

        ASTNode::Add { left, right } => Core::Add {
            left:  Box::from(desugar_node(left, ctx, state)),
            right: Box::from(desugar_node(right, ctx, state))
        },
        ASTNode::Sub { left, right } => Core::Sub {
            left:  Box::from(desugar_node(left, ctx, state)),
            right: Box::from(desugar_node(right, ctx, state))
        },
        ASTNode::Mul { left, right } => Core::Mul {
            left:  Box::from(desugar_node(left, ctx, state)),
            right: Box::from(desugar_node(right, ctx, state))
        },
        ASTNode::Div { left, right } => Core::Div {
            left:  Box::from(desugar_node(left, ctx, state)),
            right: Box::from(desugar_node(right, ctx, state))
        },
        ASTNode::FDiv { left, right } => Core::FDiv {
            left:  Box::from(desugar_node(left, ctx, state)),
            right: Box::from(desugar_node(right, ctx, state))
        },
        ASTNode::Mod { left, right } => Core::Mod {
            left:  Box::from(desugar_node(left, ctx, state)),
            right: Box::from(desugar_node(right, ctx, state))
        },
        ASTNode::Pow { left, right } => Core::Pow {
            left:  Box::from(desugar_node(left, ctx, state)),
            right: Box::from(desugar_node(right, ctx, state))
        },

        ASTNode::BAnd { left, right } => Core::BAnd {
            left:  Box::from(desugar_node(left, ctx, state)),
            right: Box::from(desugar_node(right, ctx, state))
        },
        ASTNode::BOr { left, right } => Core::BOr {
            left:  Box::from(desugar_node(left, ctx, state)),
            right: Box::from(desugar_node(right, ctx, state))
        },
        ASTNode::BXOr { left, right } => Core::BXOr {
            left:  Box::from(desugar_node(left, ctx, state)),
            right: Box::from(desugar_node(right, ctx, state))
        },
        ASTNode::BOneCmpl { expr } =>
            Core::BOneCmpl { expr: Box::from(desugar_node(expr, ctx, state)) },
        ASTNode::BLShift { left, right } => Core::BLShift {
            left:  Box::from(desugar_node(left, ctx, state)),
            right: Box::from(desugar_node(right, ctx, state))
        },
        ASTNode::BRShift { left, right } => Core::BRShift {
            left:  Box::from(desugar_node(left, ctx, state)),
            right: Box::from(desugar_node(right, ctx, state))
        },

        ASTNode::AddU { expr } => Core::AddU { expr: Box::from(desugar_node(expr, ctx, state)) },
        ASTNode::SubU { expr } => Core::SubU { expr: Box::from(desugar_node(expr, ctx, state)) },
        ASTNode::Sqrt { expr } => Core::Sqrt { expr: Box::from(desugar_node(expr, ctx, state)) },

        ASTNode::Le { left, right } => Core::Le {
            left:  Box::from(desugar_node(left, ctx, state)),
            right: Box::from(desugar_node(right, ctx, state))
        },
        ASTNode::Leq { left, right } => Core::Leq {
            left:  Box::from(desugar_node(left, ctx, state)),
            right: Box::from(desugar_node(right, ctx, state))
        },
        ASTNode::Ge { left, right } => Core::Ge {
            left:  Box::from(desugar_node(left, ctx, state)),
            right: Box::from(desugar_node(right, ctx, state))
        },
        ASTNode::Geq { left, right } => Core::Geq {
            left:  Box::from(desugar_node(left, ctx, state)),
            right: Box::from(desugar_node(right, ctx, state))
        },

        ASTNode::FunArg { vararg, id_maybe_type, default } => Core::FunArg {
            vararg:  *vararg,
            id:      Box::from(desugar_node(id_maybe_type, ctx, state)),
            default: match default {
                Some(default) => Box::from(desugar_node(default, ctx, state)),
                None => Box::from(Core::Empty)
            }
        },

        call @ ASTNode::Call { .. } => desugar_call(call, ctx, state),
        ASTNode::DirectCall { name, args } => match &name.deref().node {
            ASTNode::Id { lit } => Core::MethodCall {
                object: Box::from(Core::Empty),
                method: lit.clone(),
                args:   desugar_vec(args, ctx, state)
            },
            call => panic!("invalid function call format: {:?}", call)
        },
        ASTNode::MethodCall { instance, name, args } => match &name.deref().node {
            ASTNode::Id { lit } => Core::MethodCall {
                object: Box::from(desugar_node(instance, ctx, state)),
                method: lit.clone(),
                args:   desugar_vec(args, ctx, state)
            },
            call => panic!("invalid function call format: {:?}", call)
        },
        ASTNode::AnonFun { args, body } => Core::AnonFun {
            args: desugar_vec(args, ctx, state),
            body: Box::from(desugar_node(body, ctx, state))
        },

        ASTNode::In { left, right } => Core::In {
            left:  Box::from(desugar_node(left, ctx, state)),
            right: Box::from(desugar_node(right, ctx, state))
        },
        ASTNode::Range { from, to, inclusive, step } => Core::Range {
            from: Box::from(desugar_node(from, ctx, state)),
            to:   Box::from(if *inclusive {
                Core::Add {
                    left:  Box::from(desugar_node(to, ctx, state)),
                    right: Box::from(Core::Int { int: String::from("1") })
                }
            } else {
                desugar_node(to, ctx, state)
            }),
            step: Box::from(if let Some(step) = step {
                desugar_node(step, ctx, state)
            } else {
                Core::Int { int: String::from("1") }
            })
        },
        ASTNode::Underscore => Core::UnderScore,
        ASTNode::QuestOr { _do, _default } => Core::Block {
            statements: vec![
                Core::VarDef {
                    private: true,
                    id:      Box::from(Core::Id { lit: String::from("$temp") }),
                    right:   Box::from(desugar_node(_do, ctx, state))
                },
                Core::IfElse {
                    cond:  vec![Core::Not {
                        expr: Box::from(Core::Eq {
                            left:  Box::from(Core::Id { lit: String::from("$temp") }),
                            right: Box::from(Core::None)
                        })
                    }],
                    then:  Box::from(Core::Id { lit: String::from("$temp") }),
                    _else: Box::from(desugar_node(_default, ctx, state))
                },
            ]
        },
        ASTNode::Script { statements } =>
            Core::Block { statements: desugar_vec(statements, ctx, state) },
        ASTNode::File { modules, type_defs, .. } => {
            let mut statements: Vec<Core> = desugar_vec(type_defs, ctx, state);
            statements.append(desugar_vec(modules, ctx, state).as_mut());
            Core::Block { statements }
        }

        ASTNode::Class { _type, body } => match (&_type.deref().node, &body.deref().node) {
            (ASTNode::Type { id, generics }, ASTNode::Body { isa, definitions }) =>
                Core::ClassDef {
                    name:        Box::from(desugar_node(id, ctx, state)),
                    generics:    desugar_vec(generics, ctx, state),
                    parents:     desugar_vec(isa, ctx, state),
                    definitions: desugar_vec(definitions, ctx, state)
                },
            other => panic!("desugarer didn't recognize while making class: {:?}.", other)
        },

        ASTNode::TypeDef { .. }
        | ASTNode::TypeAlias { .. }
        | ASTNode::TypeTup { .. }
        | ASTNode::Type { .. }
        | ASTNode::TypeFun { .. } => Core::Empty,

        ASTNode::Condition { .. } => unimplemented!("Condition has not yet been implemented."),

        ASTNode::Comment { comment } => Core::Comment { comment: comment.clone() },
        ASTNode::Pass => Core::Pass,

        ASTNode::With { resource, _as, expr } => Core::With {
            resource: Box::from(desugar_node(resource, ctx, state)),
            _as:      match _as {
                Some(_as) => Box::from(desugar_node(_as, ctx, state)),
                _ => Box::from(Core::Empty)
            },
            expr:     Box::from(desugar_node(expr, ctx, state))
        },

        ASTNode::Raises { .. } => Core::Empty,
        ASTNode::Retry { .. } => unimplemented!("Retry has not yet been implemented."),
        ASTNode::Handle { expr_or_stmt, cases } => Core::TryExcept {
            _try:   Box::from(desugar_node(expr_or_stmt, ctx, state)),
            except: {
                let mut except = Vec::new();
                for case in cases {
                    let (cond, body) = match &case.node {
                        ASTNode::Case { cond, body } => (cond, body),
                        other => panic!("Expected case but was {:?}", other)
                    };

                    let (id, class) = match &cond.node {
                        ASTNode::IdType { id, _type: Some(ty), .. } => match &ty.node {
                            ASTNode::Type { id: ty, .. } => (id, ty),
                            other => panic!("Expected type but was {:?}", other)
                        },
                        other => panic!("Expected id type but was {:?}", other)
                    };

                    except.push(Core::Except {
                        id:    Box::from(desugar_node(id, ctx, state)),
                        class: Box::from(desugar_node(class, ctx, state)),
                        body:  Box::from(desugar_node(body, ctx, state))
                    });
                }
                except
            }
        },

        ASTNode::Body { .. } => panic!("Body cannot be top level."),
        ASTNode::VariableDef { .. } => panic!("Variable definition cannot be top level."),
        ASTNode::FunDef { .. } => panic!("Function definition cannot be top level."),
        ASTNode::Step { .. } => panic!("Step cannot be top level.")
    }
}

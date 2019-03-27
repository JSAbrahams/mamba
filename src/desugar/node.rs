use crate::core::construct::Core;
use crate::desugar::call::desugar_call;
use crate::desugar::control_flow::desugar_control_flow;
use crate::desugar::definition::desugar_definition;
use crate::desugar::util::desugar_vec;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use std::ops::Deref;

pub fn desugar_node(node_pos: &ASTNodePos) -> Core {
    match &node_pos.node {
        definition @ ASTNode::Def { .. } => desugar_definition(definition),
        ASTNode::Reassign { left, right } => Core::Assign {
            left:  Box::from(desugar_node(left)),
            right: Box::from(desugar_node(right))
        },

        ASTNode::Block { statements } => Core::Block { statements: desugar_vec(statements) },

        ASTNode::Int { lit } => Core::Int { int: lit.clone() },
        ASTNode::Real { lit } => Core::Float { float: lit.clone() },
        ASTNode::ENum { num, exp } => Core::ENum { num: num.clone(), exp: exp.clone() },
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

        ASTNode::IdType { id, _type } => desugar_node(id),
        ASTNode::Id { lit } => Core::Id { lit: lit.clone() },
        ASTNode::_Self => Core::Id { lit: String::from("self") },
        ASTNode::Init => Core::Id { lit: String::from("init") },
        ASTNode::Bool { lit } => Core::Bool { _bool: *lit },

        ASTNode::Tuple { elements } => Core::Tuple { elements: desugar_vec(elements) },
        ASTNode::List { elements } => Core::List { elements: desugar_vec(elements) },
        ASTNode::Set { elements } => Core::Set { elements: desugar_vec(elements) },

        ASTNode::ListBuilder { .. } => unimplemented!("list builder"),
        ASTNode::SetBuilder { .. } => unimplemented!("set builder"),

        ASTNode::ReturnEmpty => Core::Return { expr: Box::from(Core::Empty) },
        ASTNode::Return { expr } => Core::Return { expr: Box::from(desugar_node(expr)) },
        ASTNode::Print { expr } => Core::Print { expr: Box::from(desugar_node(expr)) },

        control_flow @ ASTNode::IfElse { .. }
        | control_flow @ ASTNode::Match { .. }
        | control_flow @ ASTNode::Case { .. }
        | control_flow @ ASTNode::While { .. }
        | control_flow @ ASTNode::For { .. }
        | control_flow @ ASTNode::Break
        | control_flow @ ASTNode::Continue => desugar_control_flow(control_flow),

        ASTNode::Not { expr } => Core::Not { expr: Box::from(desugar_node(expr)) },
        ASTNode::And { left, right } => Core::And {
            left:  Box::from(desugar_node(left)),
            right: Box::from(desugar_node(right))
        },
        ASTNode::Or { left, right } =>
            Core::Or { left: Box::from(desugar_node(left)), right: Box::from(desugar_node(right)) },
        ASTNode::Is { left, right } =>
            Core::Is { left: Box::from(desugar_node(left)), right: Box::from(desugar_node(right)) },
        ASTNode::IsN { left, right } => Core::IsN {
            left:  Box::from(desugar_node(left)),
            right: Box::from(desugar_node(right))
        },
        ASTNode::Eq { left, right } =>
            Core::Eq { left: Box::from(desugar_node(left)), right: Box::from(desugar_node(right)) },
        ASTNode::Neq { left, right } => Core::Neq {
            left:  Box::from(desugar_node(left)),
            right: Box::from(desugar_node(right))
        },
        ASTNode::IsA { left, right } => Core::IsA {
            left:  Box::from(desugar_node(left)),
            right: Box::from(desugar_node(right))
        },
        ASTNode::IsNA { left, right } => Core::Not {
            expr: Box::from(Core::IsA {
                left:  Box::from(desugar_node(left)),
                right: Box::from(desugar_node(right))
            })
        },

        ASTNode::Add { left, right } => Core::Add {
            left:  Box::from(desugar_node(left)),
            right: Box::from(desugar_node(right))
        },
        ASTNode::Sub { left, right } => Core::Sub {
            left:  Box::from(desugar_node(left)),
            right: Box::from(desugar_node(right))
        },
        ASTNode::Mul { left, right } => Core::Mul {
            left:  Box::from(desugar_node(left)),
            right: Box::from(desugar_node(right))
        },
        ASTNode::Div { left, right } => Core::Div {
            left:  Box::from(desugar_node(left)),
            right: Box::from(desugar_node(right))
        },
        ASTNode::Mod { left, right } => Core::Mod {
            left:  Box::from(desugar_node(left)),
            right: Box::from(desugar_node(right))
        },
        ASTNode::Pow { left, right } => Core::Pow {
            left:  Box::from(desugar_node(left)),
            right: Box::from(desugar_node(right))
        },

        ASTNode::AddU { expr } => Core::AddU { expr: Box::from(desugar_node(expr)) },
        ASTNode::SubU { expr } => Core::SubU { expr: Box::from(desugar_node(expr)) },
        ASTNode::Sqrt { expr } => Core::Sqrt { expr: Box::from(desugar_node(expr)) },

        ASTNode::Le { left, right } =>
            Core::Le { left: Box::from(desugar_node(left)), right: Box::from(desugar_node(right)) },
        ASTNode::Leq { left, right } => Core::Leq {
            left:  Box::from(desugar_node(left)),
            right: Box::from(desugar_node(right))
        },
        ASTNode::Ge { left, right } =>
            Core::Ge { left: Box::from(desugar_node(left)), right: Box::from(desugar_node(right)) },
        ASTNode::Geq { left, right } => Core::Geq {
            left:  Box::from(desugar_node(left)),
            right: Box::from(desugar_node(right))
        },

        ASTNode::FunArg { vararg, id_maybe_type, default } => Core::FunArg {
            vararg:  *vararg,
            id:      Box::from(desugar_node(id_maybe_type)),
            default: match default {
                Some(default) => Box::from(desugar_node(default)),
                None => Box::from(Core::Empty)
            }
        },

        call @ ASTNode::Call { .. } => desugar_call(call),

        ASTNode::DirectCall { name, args } => match &name.deref().node {
            ASTNode::Id { lit } => Core::MethodCall {
                object: Box::from(Core::Empty),
                method: lit.clone(),
                args:   desugar_vec(args)
            },
            call => panic!("invalid function call format: {:?}", call)
        },
        ASTNode::MethodCall { instance, name, args } => match &name.deref().node {
            ASTNode::Id { lit } => Core::MethodCall {
                object: Box::from(desugar_node(instance)),
                method: lit.clone(),
                args:   desugar_vec(args)
            },
            call => panic!("invalid function call format: {:?}", call)
        },
        ASTNode::AnonFun { args, body } =>
            Core::AnonFun { args: desugar_vec(args), body: Box::from(desugar_node(body)) },

        ASTNode::Range { from, to } => Core::MethodCall {
            object: Box::from(desugar_node(from)),
            method: String::from("range"),
            args:   vec![desugar_node(to)]
        },
        ASTNode::RangeIncl { from, to } => Core::MethodCall {
            object: Box::from(desugar_node(from)),
            method: String::from("range_incl"),
            args:   vec![desugar_node(to)]
        },
        ASTNode::Underscore => Core::UnderScore,
        ASTNode::QuestOr { _do, _default } => Core::Block {
            statements: vec![
                Core::VarDef {
                    private: true,
                    id:      Box::from(Core::Id { lit: String::from("$temp") }),
                    right:   Box::from(desugar_node(_do))
                },
                Core::IfElse {
                    cond:  vec![Core::Not {
                        expr: Box::from(Core::Eq {
                            left:  Box::from(Core::Id { lit: String::from("$temp") }),
                            right: Box::from(Core::Undefined)
                        })
                    }],
                    then:  Box::from(Core::Id { lit: String::from("$temp") }),
                    _else: Box::from(desugar_node(_default))
                },
            ]
        },
        ASTNode::Script { statements } => Core::Block { statements: desugar_vec(statements) },
        ASTNode::File { modules, type_defs, .. } => {
            let mut statements: Vec<Core> = desugar_vec(type_defs);
            statements.append(desugar_vec(modules).as_mut());
            Core::Block { statements }
        }

        ASTNode::Stateful { _type, body } | ASTNode::Stateless { _type, body } =>
            match (&_type.deref().node, &body.deref().node) {
                (ASTNode::Type { id, generics }, ASTNode::Body { isa, definitions }) =>
                    Core::ClassDef {
                        name:        Box::from(desugar_node(id)),
                        generics:    desugar_vec(generics),
                        parents:     desugar_vec(isa),
                        definitions: desugar_vec(definitions)
                    },
                other => panic!("desugarer didn't recognize while making class: {:?}.", other)
            },

        ASTNode::TypeDef { .. } => Core::Empty,
        ASTNode::TypeAlias { .. } => Core::Empty,

        ASTNode::Pass => Core::Pass,

        other => panic!("desugarer didn't recognize {:?}.", other)
    }
}

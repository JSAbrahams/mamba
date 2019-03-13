use crate::core::core::Core;
use crate::parser::ASTNode;
use crate::parser::ASTNodePos;
use std::ops::Deref;

fn desugar_vec(node_pos: &Vec<ASTNodePos>) -> Vec<Core> {
    node_pos.iter().map(|node_pos| desugar_expression(node_pos)).collect()
}

// TODO use private of definition

pub fn desugar_expression(node_pos: &ASTNodePos) -> Core {
    match &node_pos.node {
        ASTNode::Def { private: _, definition } => match &definition.deref().node {
            ASTNode::VariableDef { mutable: _, ofmut: _, id_maybe_type, expression, forward } =>
                match (id_maybe_type, expression) {
                    (id, Some(expr)) => Core::VarDef {
                        id: Box::from(desugar_expression(id)),
                        right: Box::from(desugar_expression(expr)),
                    },
                    (id, None) => desugar_expression(id)
                },
            ASTNode::FunDef { id, fun_args, body: expression, .. } => Core::FunDef {
                id: Box::from(desugar_expression(id)),
                args: desugar_vec(fun_args),
                body: Box::from(match expression {
                    Some(expr) => desugar_expression(expr),
                    None => Core::Empty
                }),
            },
            ASTNode::Init { args, body } => Core::Init {
                args: desugar_vec(args.as_ref()),
                body: Box::from(match body {
                    Some(body) => desugar_expression(body.as_ref()),
                    None => Core::Empty
                }),
            },
            definition => panic!("invalid definition format: {:?}", definition),
        }

        ASTNode::InitArg { vararg, id_maybe_type } => match &id_maybe_type.deref().node {
            ASTNode::IdType { id, .. } => Core::FunArg { vararg: *vararg, id: Box::from(desugar_expression(id.as_ref())) },
            id_maybe_type => panic!("invalid init format: {:?}", id_maybe_type),
        }

        ASTNode::ReAssign { left, right } =>
            Core::Assign { left: Box::from(desugar_expression(left)), right: Box::from(desugar_expression(right)) },

        ASTNode::Block { statements } => Core::Block {
            statements: statements.into_iter().map(|stmt| desugar_expression(stmt)).collect()
        },

        ASTNode::Int { lit } => Core::Int { int: lit.clone() },
        ASTNode::Real { lit } => Core::Float { float: lit.clone() },
        ASTNode::ENum { num, exp } => Core::ENum { num: num.clone(), exp: exp.clone() },
        ASTNode::Str { lit } => Core::Str { _str: lit.clone() },

        ASTNode::IdType { id, _type } => desugar_expression(id),
        ASTNode::Id { lit } => Core::Id { lit: lit.clone() },
        ASTNode::_Self => Core::Id { lit: String::from("self") },
        ASTNode::Bool { lit } => Core::Bool { _bool: lit.clone() },

        ASTNode::Tuple { elements } => Core::Tuple { elements: desugar_vec(elements.as_ref()) },
        ASTNode::List { elements } => Core::List { elements: desugar_vec(elements.as_ref()) },
        ASTNode::Set { elements } => Core::Set { elements: desugar_vec(elements.as_ref()) },

        ASTNode::ListBuilder { items, conditions } => Core::Block { statements: unimplemented!() },
        ASTNode::SetBuilder { items, conditions } => Core::Block { statements: unimplemented!() },

        ASTNode::ReturnEmpty => Core::Return { expr: Box::from(Core::Empty) },
        ASTNode::Return { expr } => Core::Return { expr: Box::from(desugar_expression(expr)) },
        ASTNode::Print { expr } => Core::Print { expr: Box::from(desugar_expression(expr)) },
        ASTNode::PrintLn { expr } => Core::Print { expr: Box::from(desugar_expression(expr)) },

        ASTNode::IfElse { cond, then, _else } => Core::IfElse {
            cond: Box::from(desugar_expression(cond)),
            then: Box::from(desugar_expression(then)),
            _else: Box::from(match _else {
                Some(el) => desugar_expression(el),
                None => Core::Empty
            }),
        },
        ASTNode::When { cond, cases } => Core::When {
            cond: Box::from(desugar_expression(cond)),
            cases: desugar_vec(cases.as_ref()),
        },
        ASTNode::Case { cond, expr_or_stmt } => Core::Case {
            cond: Box::from(desugar_expression(cond)),
            then: Box::from(desugar_expression(expr_or_stmt)),
        },
        ASTNode::While { cond, body } => Core::While {
            cond: Box::from(desugar_expression(cond)),
            body: Box::from(desugar_expression(body)),
        },

        ASTNode::Break => Core::Break,
        ASTNode::Continue => Core::Continue,

        ASTNode::Not { expr } => Core::Not { expr: Box::from(desugar_expression(expr)) },
        ASTNode::And { left, right } => Core::And { left: Box::from(desugar_expression(left)), right: Box::from(desugar_expression(right)) },
        ASTNode::Or { left, right } => Core::Or { left: Box::from(desugar_expression(left)), right: Box::from(desugar_expression(right)) },

        ASTNode::Is { left, right } => Core::Is { left: Box::from(desugar_expression(left)), right: Box::from(desugar_expression(right)) },
        ASTNode::IsN { left, right } => Core::Not {
            expr: Box::from(Core::Is { left: Box::from(desugar_expression(left)), right: Box::from(desugar_expression(right)) })
        },
        ASTNode::Eq { left, right } => Core::Eq { left: Box::from(desugar_expression(left)), right: Box::from(desugar_expression(right)) },
        ASTNode::Neq { left, right } => Core::Not {
            expr: Box::from(Core::Eq { left: Box::from(desugar_expression(left)), right: Box::from(desugar_expression(right)) })
        },
        ASTNode::IsA { left, right } => Core::IsA { left: Box::from(desugar_expression(left)), right: Box::from(desugar_expression(right)) },
        ASTNode::IsNA { left, right } => Core::Not {
            expr: Box::from(Core::IsA { left: Box::from(desugar_expression(left)), right: Box::from(desugar_expression(right)) })
        },

        ASTNode::Add { left, right } => Core::Add { left: Box::from(desugar_expression(left)), right: Box::from(desugar_expression(right)) },
        ASTNode::Sub { left, right } => Core::Sub { left: Box::from(desugar_expression(left)), right: Box::from(desugar_expression(right)) },
        ASTNode::Mul { left, right } => Core::Mul { left: Box::from(desugar_expression(left)), right: Box::from(desugar_expression(right)) },
        ASTNode::Div { left, right } => Core::Div { left: Box::from(desugar_expression(left)), right: Box::from(desugar_expression(right)) },
        ASTNode::Mod { left, right } => Core::Mod { left: Box::from(desugar_expression(left)), right: Box::from(desugar_expression(right)) },
        ASTNode::Pow { left, right } => Core::Pow { left: Box::from(desugar_expression(left)), right: Box::from(desugar_expression(right)) },

        ASTNode::AddU { expr } => Core::AddU { expr: Box::from(desugar_expression(expr)) },
        ASTNode::SubU { expr } => Core::SubU { expr: Box::from(desugar_expression(expr)) },
        ASTNode::Sqrt { expr } => Core::Sqrt { expr: Box::from(desugar_expression(expr)) },

        ASTNode::Le { left, right } =>
            Core::Le { left: Box::from(desugar_expression(left)), right: Box::from(desugar_expression(right)) },
        ASTNode::Leq { left, right } =>
            Core::Leq { left: Box::from(desugar_expression(left)), right: Box::from(desugar_expression(right)) },
        ASTNode::Ge { left, right } =>
            Core::Ge { left: Box::from(desugar_expression(left)), right: Box::from(desugar_expression(right)) },
        ASTNode::Geq { left, right } =>
            Core::Geq { left: Box::from(desugar_expression(left)), right: Box::from(desugar_expression(right)) },

        // TODO do something with default
        ASTNode::FunArg { vararg, id_maybe_type, default } => Core::FunArg {
            vararg: vararg.clone(),
            id: Box::from(desugar_expression(id_maybe_type)),
        },
        ASTNode::Call { instance_or_met, met_or_arg } => unimplemented!(),

        ASTNode::FunCall { namespace, name, args } => match &name.deref().node {
            ASTNode::Id { lit } => Core::MethodCall {
                object: Box::from(desugar_expression(namespace)),
                method: lit.clone(),
                args: desugar_vec(args),
            },
            call => panic!("invalid function call format: {:?}", call),
        }
        ASTNode::MetCall { instance, name, args } => match &name.deref().node {
            ASTNode::Id { lit } => Core::MethodCall {
                object: Box::from(desugar_expression(instance)),
                method: lit.clone(),
                args: desugar_vec(args),
            },
            call => panic!("invalid function call format: {:?}", call),
        }
        ASTNode::Range { from, to } => Core::MethodCall {
            object: Box::from(desugar_expression(from)),
            method: String::from("range"),
            args: vec![desugar_expression(to)],
        },
        ASTNode::RangeIncl { from, to } => Core::MethodCall {
            object: Box::from(desugar_expression(from)),
            method: String::from("range_incl"),
            args: vec![desugar_expression(to)],
        },

        ASTNode::UnderScore => Core::UnderScore,
        ASTNode::QuestOr { _do, _default } => Core::Block {
            statements: vec![
                Core::VarDef { id: Box::from(Core::Id { lit: String::from("$temp") }), right: Box::from(desugar_expression(_do)) },
                Core::IfElse {
                    cond: Box::from(Core::Not {
                        expr: Box::from(Core::Eq {
                            left: Box::from(Core::Id { lit: String::from("$temp") }),
                            right: Box::from(Core::Undefined),
                        })
                    }),
                    then: Box::from(Core::Id { lit: String::from("$temp") }),
                    _else: Box::from(desugar_expression(_default)),
                }
            ]
        },

        ASTNode::Script { statements } => Core::Block { statements: desugar_vec(statements) },

        ASTNode::File { imports: _, modules, type_defs } => {
            let mut statements: Vec<Core> = desugar_vec(type_defs);
            statements.append(desugar_vec(modules).as_mut());
            Core::Block { statements }
        }

        ASTNode::Stateful { _type, body } |
        ASTNode::Stateless { _type, body } => match (&_type.deref().node, &body.deref().node) {
            (ASTNode::Type { id, generics }, ASTNode::Body { isa, definitions }) => Core::ClassDef {
                name: Box::from(desugar_expression(id)),
                generics: desugar_vec(generics),
                parents: desugar_vec(isa.as_ref()),
                definitions: desugar_vec(definitions.as_ref()),
            },
            other => panic!("desugarer didn't recognize while making class: {:?}.")
        }

        ASTNode::TypeDef { .. } => Core::Empty,
        ASTNode::TypeAlias { .. } => Core::Empty,

        other => panic!("desugarer didn't recognize {:?}.", other)
    }
}

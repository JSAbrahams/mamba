use crate::core::construct::Core;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use std::ops::Deref;

fn desugar_vec(node_pos: &[ASTNodePos]) -> Vec<Core> {
    node_pos.iter().map(|node_pos| desugar_node(node_pos)).collect()
}

pub fn desugar_node(node_pos: &ASTNodePos) -> Core {
    match &node_pos.node {
        ASTNode::Def { private, definition } => match &definition.deref().node {
            // TODO do something with forward
            ASTNode::VariableDef { id_maybe_type, expression, .. } =>
                match (id_maybe_type, expression) {
                    (id, Some(expr)) => Core::VarDef {
                        private: *private,
                        id:      Box::from(desugar_node(id)),
                        right:   Box::from(desugar_node(expr))
                    },
                    (id, None) => desugar_node(id)
                },
            ASTNode::FunDef { id, fun_args, body: expression, .. } => Core::FunDef {
                private: *private,
                id:      Box::from(desugar_node(id)),
                args:    desugar_vec(fun_args),
                body:    Box::from(match expression {
                    Some(expr) => desugar_node(expr),
                    None => Core::Empty
                })
            },
            definition => panic!("invalid definition format: {:?}", definition)
        },

        ASTNode::ReAssign { left, right } => Core::Assign {
            left:  Box::from(desugar_node(left)),
            right: Box::from(desugar_node(right))
        },

        ASTNode::Block { statements } =>
            Core::Block { statements: statements.iter().map(|stmt| desugar_node(stmt)).collect() },

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

        ASTNode::IdType { id, _type } => desugar_node(id),
        ASTNode::Id { lit } => Core::Id { lit: lit.clone() },
        ASTNode::_Self => Core::Id { lit: String::from("self") },
        ASTNode::Bool { lit } => Core::Bool { _bool: *lit },

        ASTNode::Tuple { elements } => Core::Tuple { elements: desugar_vec(elements) },
        ASTNode::List { elements } => Core::List { elements: desugar_vec(elements) },
        ASTNode::Set { elements } => Core::Set { elements: desugar_vec(elements) },

        ASTNode::ListBuilder { .. } => unimplemented!("list builder"),
        ASTNode::SetBuilder { .. } => unimplemented!("set builder"),

        ASTNode::ReturnEmpty => Core::Return { expr: Box::from(Core::Empty) },
        ASTNode::Return { expr } => Core::Return { expr: Box::from(desugar_node(expr)) },
        ASTNode::Print { expr } => Core::Print { expr: Box::from(desugar_node(expr)) },

        ASTNode::IfElse { cond, then, _else } => match _else {
            Some(_else) => Core::IfElse {
                cond:  desugar_vec(cond),
                then:  Box::from(desugar_node(then)),
                _else: Box::from(desugar_node(_else))
            },
            None => Core::If { cond: desugar_vec(cond), then: Box::from(desugar_node(then)) }
        },
        ASTNode::Match { cond, cases } =>
            Core::When { cond: desugar_vec(cond), cases: desugar_vec(cases) },
        ASTNode::Case { cond, expr } =>
            Core::Case { cond: Box::from(desugar_node(cond)), then: Box::from(desugar_node(expr)) },
        ASTNode::While { cond, body } =>
            Core::While { cond: desugar_vec(cond), body: Box::from(desugar_node(body)) },
        ASTNode::For { expr, collection, body } => Core::For {
            expr:       desugar_vec(expr),
            collection: Box::from(desugar_node(collection)),
            body:       Box::from(desugar_node(body))
        },

        ASTNode::Break => Core::Break,
        ASTNode::Continue => Core::Continue,

        ASTNode::Not { expr } => Core::Not { expr: Box::from(desugar_node(expr)) },
        ASTNode::And { left, right } => Core::And {
            left:  Box::from(desugar_node(left)),
            right: Box::from(desugar_node(right))
        },
        ASTNode::Or { left, right } =>
            Core::Or { left: Box::from(desugar_node(left)), right: Box::from(desugar_node(right)) },

        ASTNode::Is { left, right } =>
            Core::Is { left: Box::from(desugar_node(left)), right: Box::from(desugar_node(right)) },
        ASTNode::IsN { left, right } => Core::Not {
            expr: Box::from(Core::Is {
                left:  Box::from(desugar_node(left)),
                right: Box::from(desugar_node(right))
            })
        },
        ASTNode::Eq { left, right } =>
            Core::Eq { left: Box::from(desugar_node(left)), right: Box::from(desugar_node(right)) },
        ASTNode::Neq { left, right } => Core::Not {
            expr: Box::from(Core::Eq {
                left:  Box::from(desugar_node(left)),
                right: Box::from(desugar_node(right))
            })
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

        // TODO do something with default
        ASTNode::FunArg { vararg, id_maybe_type, .. } =>
            Core::FunArg { vararg: *vararg, id: Box::from(desugar_node(id_maybe_type)) },

        // TODO use context to check whether identifier is function or property
        // Currently:
        // a b => a.b , where a may be expression, b must be id
        // a b c => a.b(c), where and c may be expression, b must be id
        // a b c d => a.b(c.d) etc.
        ASTNode::Call { instance_or_met, met_or_arg } => match &met_or_arg.deref().node {
            ASTNode::Call { instance_or_met: method, met_or_arg } => match &method.deref().node {
                ASTNode::Id { lit: method } => Core::MethodCall {
                    object: Box::from(desugar_node(instance_or_met)),
                    method: method.clone(),
                    args:   vec![desugar_node(met_or_arg)]
                },
                other => panic!("Chained method call must have identifier, was {:?}", other)
            },
            ASTNode::Id { lit } => Core::PropertyCall {
                object:   Box::from(desugar_node(instance_or_met)),
                property: lit.clone()
            },
            _ => match &instance_or_met.deref().node {
                ASTNode::Id { lit: method } => Core::MethodCall {
                    object: Box::from(Core::Empty),
                    method: method.clone(),
                    args:   vec![desugar_node(met_or_arg)]
                },
                other => panic!("desugaring calls not that advanced yet: {:?}.", other)
            }
        },

        ASTNode::FunctionCall { namespace, name, args } => match &name.deref().node {
            ASTNode::Id { lit } => Core::MethodCall {
                object: Box::from(desugar_node(namespace)),
                method: lit.clone(),
                args:   desugar_vec(args)
            },
            call => panic!("invalid function call format: {:?}", call)
        },
        ASTNode::FunctionCallDirect { name, args } => match &name.deref().node {
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

        other => panic!("desugarer didn't recognize {:?}.", other)
    }
}

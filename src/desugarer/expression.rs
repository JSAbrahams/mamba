#![feature(box_syntax, box_patterns)]

use crate::core::core::Core;
use crate::desugarer::context::Context;
use crate::desugarer::desugar;
use crate::parser::ASTNode;
use crate::parser::ASTNodePos;
use std::cmp;

const LONG_MAX: &str = "18446744073709551615";

#[macro_use]
macro_rules! operator {
    ($left:expr, $op:expr, $right:expr) => {{ Core::MethodCall {
        object: box desugar($left),
        method: $op,
        args: vec![box desugar($right)],
    }}};
    ($left:expr, $op:expr) => {{ Core::MethodCall {
        object: box desugar($left),
        method: $op,
        args: vec![],
    }}}
}

pub fn desugar_expression(node_pos: ASTNodePos, context: Context) -> Core {
    match node_pos.node {
        ASTNode::Def {
            definition: box ASTNode::VariableDef {
                id_maybe_type: box ASTNode::TypeId { box id, .. },
                expression: box expression, ..
            }, ..
        } => Core::VarDef { id: box desugar(box id), right: box desugar(expression) },
        ASTNode::Def {
            definition: box ASTNode::FunDef {
                id: box id,
                fun_args: box fun_args,
                body: box expression,
                raises: box raises, ..
            }, ..
        } => Core::FunDef {
            id: box desugar(id),
            args: box desugar(fun_args),
            body: box desugar(expression),
        },
        ASTNode::Def { definition, .. } => panic!("invalid definition format: {:?}", definition),

        ASTNode::Init { box args, box body } => Core::Init {
            id: String::from("init"),
            args: box desugar(box args),
            body: box desugar(box body),
        },
        ASTNode::InitArg { vararg, def, id_maybe_type: box ASTNode::TypeId { id, .. } } =>
            Core::FunArg { vararg, id },

        ASTNode::ReAssign { left, right } =>
            Core::Assign { left: box desugar(box left), right: box desugar(box right) },

        ASTNode::Block { statements } => Core::Block {
            statements: statements.into_iter().map(|stmt| box desugar(stmt))
        },

        ASTNode::Int { .. } | ASTNode::Real { .. } | ASTNode::ENum { .. } =>
            desugar_num(node_pos.node),
        ASTNode::Id { lit } => Core::Id { lit },
        ASTNode::_Self => Core::Id { lit: String::from("self") },
        ASTNode::Bool { _bool } => Core::Bool { _bool },

        ASTNode::Tuple { elements } => Core::Tuple { elements: box desugar(box elements) },
        ASTNode::List { elements } => Core::List { elements: box desugar(box elements) },
        ASTNode::EmptyList { elements } => Core::List { elements: vec![] },
        ASTNode::Set { elements } => Core::Set { elements: box desugar(box elements) },
        ASTNode::EmptySet { elements } => Core::List { elements: vec![] },

        ASTNode::ListBuilder { item, conditions } => Core::Block { statements: unimplemented!() },
        ASTNode::SetBuilder { item, conditions } => Core::Block { statements: unimplemented!() },

        ASTNode::ReturnEmpty => Core::Return { expr: box Core::Empty },
        ASTNode::Return { expr } => Core::Return { expr: box desugar(box expr) },
        ASTNode::Print { expr } => Core::Print { expr: box desugar(box expr) },
        ASTNode::PrintLn { expr } => Core::Print { expr: box desugar(box expr) },

        ASTNode::IfElse { cond, then, _else } => Core::IfElse {
            cond: box desugar(box cond),
            then: box desugar(box then),
            _else: if _else.is_some() { box desugar(box _else) } else { box Core::Empty },
        },
        ASTNode::When { cond, cases } => Core::When {
            cond: box desugar(box cond),
            cases: box desugar(box cases),
        },
        ASTNode::Case { cond, expr_or_stmt } => Core::Case {
            cond: box desugar(box cond),
            then: box desugar(box expr_or_stmt),
        },
        ASTNode::While { cond, body } => Core::While {
            cond: box desugar(box cond),
            body: box desugar(box body),
        },

        ASTNode::Break => Core::Break,
        ASTNode::Continue => Core::Continue,

        ASTNode::Not { expr } => Core::Not { expr: box desugar(box expr) },
        ASTNode::And { left, right } => Core::And { left: box desugar(box left), right: box desugar(box right) },
        ASTNode::Or { left, right } => Core::Or { left: box desugar(box left), right: box desugar(box right) },

        ASTNode::Is { left, right } => Core::Is { left: box desugar(box left), right: box desugar(box right) },
        ASTNode::IsN { left, right } => Core::Not {
            expr: box Core::Is { left: box desugar(box left), right: box desugar(box right) }
        },
        ASTNode::Eq { left, right } => Core::Eq { left: box desugar(box left), right: box desugar(box right) },
        ASTNode::Neq { left, right } => Core::Not {
            expr: box Core::Eq { left: box desugar(box left), right: box desugar(box right) }
        },
        ASTNode::IsA { left, right } => Core::IsA { left: box desugar(box left), right: box desugar(box right) },
        ASTNode::IsNA { left, right } => Core::Not {
            expr: box Core::IsA { left: box desugar(box left), right: box desugar(box right) }
        },

        ASTNode::Add { left, right } => operator!(left, "+", right),
        ASTNode::Sub { left, right } => operator!(left, "-", right),
        ASTNode::Mul { left, right } => operator!(left, "*", right),
        ASTNode::Div { left, right } => operator!(left, "/", right),
        ASTNode::Mod { left, right } => operator!(left, "mod", right),
        ASTNode::Pow { left, right } => operator!(left, "^", right),

        ASTNode::AddU { expr } => operator!(expr, "+"),
        ASTNode::SubU { expr } => operator!(expr, "-"),
        ASTNode::Sqrt { expr } => operator!(expr, "sqrt"),

        ASTNode::Le { lleft, right: box ASTNodePos { node: ASTNode::Le { rleft, rright }, .. } } =>
            Core::And {
                left: box Core::Le { left: box desugar(lleft), right: box desugar(rleft) },
                right: box Core::Le { left: box desugar(rleft), right: box desugar(rright) },
            },
        ASTNode::Le { lleft, right: box ASTNodePos { node: ASTNode::Leq { rleft, rright }, .. } } =>
            Core::And {
                left: box Core::Le { left: box desugar(lleft), right: box desugar(rleft) },
                right: box Core::Leq { left: box desugar(rleft), right: box desugar(rright) },
            },
        ASTNode::Le { left, right } => Core::Le { left: box desugar(box left), right: box desugar(box right) },

        ASTNode::Ge { lleft, right: box ASTNodePos { node: ASTNode::Ge { rleft, rright }, .. } } =>
            Core::And {
                left: box Core::Ge { left: box desugar(lleft), right: box desugar(rleft) },
                right: box Core::Ge { left: box desugar(rleft), right: box desugar(rright) },
            },
        ASTNode::Ge { lleft, right: box ASTNodePos { node: ASTNode::Geq { rleft, rright }, .. } } =>
            Core::And {
                left: box Core::Ge { left: box desugar(lleft), right: box desugar(rleft) },
                right: box Core::Geq { left: box desugar(rleft), right: box desugar(rright) },
            },
        ASTNode::Ge { left, right } => Core::Ge { left: box desugar(box left), right: box desugar(box right) },

        ASTNode::Leq { lleft, right: box ASTNodePos { node: ASTNode::Le { rleft, rright }, .. } } =>
            Core::And {
                left: box Core::Leq { left: box desugar(lleft), right: box desugar(rleft) },
                right: box Core::Le { left: box desugar(rleft), right: box desugar(rright) },
            },
        ASTNode::Leq { lleft, right: box ASTNodePos { node: ASTNode::Leq { rleft, rright }, .. } } =>
            Core::And {
                left: box Core::Leq { left: box desugar(lleft), right: box desugar(rleft) },
                right: box Core::Leq { left: box desugar(rleft), right: box desugar(rright) },
            },
        ASTNode::Leq { left, right } =>
            Core::Leq { left: box desugar(box left), right: box desugar(box right) },

        ASTNode::Geq { lleft, right: box ASTNodePos { node: ASTNode::Ge { rleft, rright }, .. } } =>
            Core::And {
                left: box Core::Geq { left: box desugar(lleft), right: box desugar(rleft) },
                right: box Core::Ge { left: box desugar(rleft), right: box desugar(rright) },
            },
        ASTNode::Geq { ll, right: box ASTNodePos { node: ASTNode::Geq { rl, rr }, .. } } =>
            Core::And {
                left: box Core::Geq { left: box desugar(ll), right: box desugar(rl) },
                right: box Core::Geq { left: box desugar(rl), right: box desugar(rr) },
            },
        ASTNode::Geq { left, right } =>
            Core::Geq { left: box desugar(box left), right: box desugar(box right) },

        ASTNode::Range { from, to } => Core::MethodCall {
            object: box desugar(box from),
            method: String::from("range"),
            args: vec![box desugar(box to)],
        },
        ASTNode::RangeIncl { from, to } => Core::MethodCall {
            object: box desugar(box from),
            method: String::from("range_incl"),
            args: vec![box desugar(box to)],
        },

        ASTNode::UnderScore => Core::UnderScore,
        ASTNode::QuestOr { _do, default } => Core::Block {
            statements: vec![
                Core::VarDef { id: String::from("$temp"), right: box desugar(box _do) },
                Core::IfElse {
                    cond: box Core::Not {
                        expr: box Core::Eq {
                            left: box Core::Id { lit: String::from("$temp") },
                            right: box Core::Undefined,
                        }
                    },
                    then: box Core::Id { lit: String::from("$temp") },
                    _else: box desugar(default),
                }
            ]
        },

        _ => panic!("")
    }
}

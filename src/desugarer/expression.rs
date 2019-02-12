#![feature(box_syntax, box_patterns)]

use crate::desugarer::Core;
use crate::desugarer::desugar;
use crate::parser::ASTNode;
use crate::parser::ASTNodePos;
use std::cmp;

const LONG_MAX: &str = "18446744073709551615";

#[macro_use]
macro_rules! operator {
    ($left:expr, $op:expr, $right:expr) => {{ Core::MethodCall {
        object: desugar!($left),
        method: $op,
        args: vec![desugar!($right)],
    }}};
    ($left:expr, $op:expr) => {{ Core::MethodCall {
        object: desugar!($left),
        method: $op,
        args: vec![],
    }}}
}

pub fn desugar_expression(node_pos: ASTNodePos) -> Core {
    match node_pos.node {
        ASTNode::Def {
            definition: box ASTNode::VariableDef {
                id_maybe_type: box ASTNode::TypeId { box id, .. },
                expression: box expression, ..
            }, ..
        } => Core::VarDef { id: desugar!(id), right: desugar!(expression) },
        ASTNode::Def {
            definition: box ASTNode::FunDef {
                id: box id,
                fun_args: box fun_args,
                body: box expression,
                raises: box raises, ..
            }, ..
        } => Core::FunDef {
            id: desugar!(id),
            args: desugar!(fun_args),
            raises: desugar!(raises),
            right: desugar!(expression),
        },
        ASTNode::Def { definition, .. } => panic!("invalid definition format: {:?}", definition),

        ASTNode::Init { box args, box body } => Core::FunDef {
            id: "init",
            args: desugar!(args),
            raises: vec![],
            right: desugar!(body),
        },
        ASTNode::InitArg { vararg, def, id_maybe_type: box ASTNode::TypeId { id, .. } } =>
            Core::FunArg { vararg, id },

        ASTNode::ReAssign { left, right } =>
            Core::Assign { left: desugar!(left), right: desugar!(right) },

        ASTNode::Block { statements } => Core::Block {
            statements: statements.into_iter().map(|stmt| desugar!(stmt))
        },

        ASTNode::Int { .. } | ASTNode::Real { .. } | ASTNode::ENum { .. } =>
            desugar_num(node_pos.node),
        ASTNode::Id { lit } => Core::Id { lit },
        ASTNode::_Self => Core::Id { lit: String::from("self") },
        ASTNode::Bool { _bool } => Core::Bool { _bool },

        ASTNode::Tuple { elements } => Core::Tuple { elements: desugar!(elements) },
        ASTNode::List { elements } => Core::List { elements: desugar!(elements) },
        ASTNode::EmptyList { elements } => Core::List { elements: vec![] },
        ASTNode::Set { elements } => Core::Set { elements: desugar!(elements) },
        ASTNode::EmptySet { elements } => Core::List { elements: vec![] },

        ASTNode::ListBuilder { item, conditions } => Core::Block { statements: unimplemented!() },
        ASTNode::SetBuilder { item, conditions } => Core::Block { statements: unimplemented!() },

        ASTNode::ReturnEmpty => Core::Return { expr: box Core::Empty },
        ASTNode::Return { box expr } => Core::Return { expr: desugar!(expr) },
        ASTNode::Print { box expr } => Core::Print { expr: desugar!(expr) },
        ASTNode::PrintLn { box expr } => Core::Print { expr: desugar!(expr) },

        ASTNode::IfElse { cond, then, _else } => Core::IfElse {
            cond: desugar!(cond),
            then: desugar!(then),
            _else: if _else.is_some() { desugar!(_else) } else { box Core::Empty },
        },
        ASTNode::When { cond, cases } => Core::When {
            cond: desugar!(cond),
            cases: desugar!(cases),
        },
        ASTNode::Case { cond, expr_or_stmt } => Core::Case {
            cond: desugar!(cond),
            then: desugar!(expr_or_stmt),
        },
        ASTNode::While { cond, body } => Core::While {
            cond: desugar!(cond),
            body: desugar!(body),
        },

        ASTNode::Break => Core::Break,
        ASTNode::Continue => Core::Continue,

        ASTNode::Not { expr } => Core::Not { expr: desugar!(expr) },
        ASTNode::And { left, right } => Core::And { left: desugar!(left), right: desugar!(right) },
        ASTNode::Or { left, right } => Core::Or { left: desugar!(left), right: desugar!(right) },

        ASTNode::Is { left, right } => Core::Is { left: desugar!(left), right: desugar!(right) },
        ASTNode::IsN { left, right } => Core::Not {
            expr: box Core::Is { left: desugar!(left), right: desugar!(right) }
        },
        ASTNode::Eq { left, right } => Core::Eq { left: desugar!(left), right: desugar!(right) },
        ASTNode::Neq { left, right } => Core::Not {
            expr: box Core::Eq { left: desugar!(left), right: desugar!(right) }
        },
        ASTNode::IsA { left, right } => Core::IsA { left: desugar!(left), right: desugar!(right) },
        ASTNode::IsNA { left, right } => Core::Not {
            expr: box Core::IsA { left: desugar!(left), right: desugar!(right) }
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
                left: box Core::Le { left: desugar!(lleft), right: desugar!(rleft) },
                right: box Core::Le { left: desugar!(rleft), right: desugar!(rright) },
            },
        ASTNode::Le { lleft, right: box ASTNodePos { node: ASTNode::Leq { rleft, rright }, .. } } =>
            Core::And {
                left: box Core::Le { left: desugar!(lleft), right: desugar!(rleft) },
                right: box Core::Leq { left: desugar!(rleft), right: desugar!(rright) },
            },
        ASTNode::Le { left, right } => Core::Le { left: desugar!(left), right: desugar!(right) },

        ASTNode::Ge { lleft, right: box ASTNodePos { node: ASTNode::Ge { rleft, rright }, .. } } =>
            Core::And {
                left: box Core::Ge { left: desugar!(lleft), right: desugar!(rleft) },
                right: box Core::Ge { left: desugar!(rleft), right: desugar!(rright) },
            },
        ASTNode::Ge { lleft, right: box ASTNodePos { node: ASTNode::Geq { rleft, rright }, .. } } =>
            Core::And {
                left: box Core::Ge { left: desugar!(lleft), right: desugar!(rleft) },
                right: box Core::Geq { left: desugar!(rleft), right: desugar!(rright) },
            },
        ASTNode::Ge { left, right } => Core::Ge { left: desugar!(left), right: desugar!(right) },

        ASTNode::Leq { lleft, right: box ASTNodePos { node: ASTNode::Le { rleft, rright }, .. } } =>
            Core::And {
                left: box Core::Leq { left: desugar!(lleft), right: desugar!(rleft) },
                right: box Core::Le { left: desugar!(rleft), right: desugar!(rright) },
            },
        ASTNode::Leq { lleft, right: box ASTNodePos { node: ASTNode::Leq { rleft, rright }, .. } } =>
            Core::And {
                left: box Core::Leq { left: desugar!(lleft), right: desugar!(rleft) },
                right: box Core::Leq { left: desugar!(rleft), right: desugar!(rright) },
            },
        ASTNode::Leq { left, right } => Core::Leq { left: desugar!(left), right: desugar!(right) },

        ASTNode::Geq { lleft, right: box ASTNodePos { node: ASTNode::Ge { rleft, rright }, .. } } =>
            Core::And {
                left: box Core::Geq { left: desugar!(lleft), right: desugar!(rleft) },
                right: box Core::Ge { left: desugar!(rleft), right: desugar!(rright) },
            },
        ASTNode::Geq { ll, right: box ASTNodePos { node: ASTNode::Geq { rl, rr }, .. } } =>
            Core::And {
                left: box Core::Geq { left: desugar!(ll), right: desugar!(rl) },
                right: box Core::Geq { left: desugar!(rl), right: desugar!(rr) },
            },
        ASTNode::Geq { left, right } => Core::Geq { left: desugar!(left), right: desugar!(right) },

        ASTNode::Range { from, to } => Core::MethodCall {
            object: desugar!(from),
            method: "range",
            args: vec![desugar!(to)],
        },
        ASTNode::RangeIncl { from, to } => Core::MethodCall {
            object: desugar!(from),
            method: "range_incl",
            args: vec![desugar!(to)],
        },

        ASTNode::UnderScore => Core::UnderScore,
        ASTNode::QuestOr { _do, default } => Core::Block {
            statements: vec![
                Core::VarDef { id: "$temp", right: desugar!(_do) },
                Core::IfElse {
                    cond: Core::Not {
                        expr: Core::Eq { left: Core::Id { lit: "$temp" }, right: Core::Undefined }
                    },
                    then: Core::Id { lit: "$temp" },
                    _else: desugar!(default),
                }
            ]
        },

        _ => panic!("")
    }
}

fn desugar_num(node: ASTNode) -> Core {
    match node {
        ASTNode::ENum { num, exp } => unimplemented!(),
        ASTNode::Int { mut lit } => to_bit_int(&mut lit),
        ASTNode::Real { mut lit } => unimplemented!(),
        _ => panic!("Tried to desugar {:?} as number", node)
    }
}

fn to_big_float(lit: &mut String) -> Core {
    let mut integers: Vec<i64> = Vec::new();
    for i in (0..lit.len()).step_by(LONG_MAX.len() - 1) {
        let num: String =
            lit.drain(..cmp::min(lit.len(), LONG_MAX.len())).collect();
        integers.push(num.parse().unwrap());
    }

    return Core::BigFloat { int_digits: Vec::new(), frac_digits: Vec::new() };
}

fn to_bit_int(lit: &mut String) -> Core {
    let mut integers: Vec<i64> = Vec::new();
    for i in (0..lit.len()).step_by(LONG_MAX.len() - 1) {
        let num: String =
            lit.drain(..cmp::min(lit.len(), LONG_MAX.len())).collect();
        integers.push(num.parse().unwrap());
    }

    return Core::BigInt { integers };
}

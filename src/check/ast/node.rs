use std::convert::TryFrom;
use std::ops::Deref;

use crate::{AST, ASTTy};
use crate::check::ast::NodeTy;
use crate::check::constrain::unify::finished::Finished;
use crate::check::name::{Empty, Name};
use crate::check::name::string_name::StringName;
use crate::parse::ast::Node;

impl From<(&Box<AST>, &Finished)> for NodeTy {
    fn from((ast, finished): (&Box<AST>, &Finished)) -> Self {
        NodeTy::from((ast.deref(), finished))
    }
}

impl From<(&AST, &Finished)> for NodeTy {
    fn from((ast, finished): (&AST, &Finished)) -> Self {
        NodeTy::from((&ast.node, finished))
    }
}

impl From<(&Box<Node>, &Finished)> for NodeTy {
    fn from((node, finished): (&Box<Node>, &Finished)) -> Self {
        NodeTy::from((node.deref(), finished))
    }
}

impl From<(&Node, &Finished)> for NodeTy {
    fn from((node, finished): (&Node, &Finished)) -> Self {
        let pos_to_name = &finished.pos_to_name;

        match node {
            Node::Import { from, import, alias } => NodeTy::Import {
                from: from.clone().map(|from| ASTTy::from((from, finished))).map(Box::from),
                import: import.iter().map(|ast| ASTTy::from((ast, finished))).collect(),
                alias: alias.iter().map(|ast| ASTTy::from((ast, finished))).collect(),
            },
            Node::Class { ty, args, parents, body } => NodeTy::Class {
                ty: StringName::try_from(ty).unwrap_or_else(|_| StringName::empty()),
                args: args.iter().map(|ast| ASTTy::from((ast, finished))).collect(),
                parents: parents.iter().map(|ast| ASTTy::from((ast, finished))).collect(),
                body: body.clone().map(|ast| ASTTy::from((ast, finished))).map(Box::from),
            },
            Node::Parent { ty, args } => NodeTy::Parent {
                ty: StringName::try_from(ty).unwrap_or_else(|_| StringName::empty()),
                args: args.iter().map(|ast| ASTTy::from((ast, finished))).collect(),
            },
            Node::Reassign { left, right, op } => NodeTy::Reassign {
                left: Box::from(ASTTy::from((left, finished))),
                right: Box::from(ASTTy::from((right, finished))),
                op: op.clone(),
            },
            Node::VariableDef { mutable, var, expr, forward, ty, .. } => NodeTy::VariableDef {
                mutable: *mutable,
                var: Box::from(ASTTy::from((var, finished))),
                ty: if let Some(ty) = ty {
                    Name::try_from(ty).ok()
                } else {
                    pos_to_name.get(&var.pos).cloned()
                },
                expr: expr.clone().map(|ast| ASTTy::from((ast, finished))).map(Box::from),
                forward: forward.iter().map(|ast| ASTTy::from((ast, finished))).collect(),
            },
            Node::FunDef { pure, id, args, ret, raises, body } => NodeTy::FunDef {
                pure: *pure,
                id: Box::from(ASTTy::from((id, finished))),
                args: args.iter().map(|ast| ASTTy::from((ast, finished))).collect(),
                ret: ret.as_ref().and_then(|ret_ty| Name::try_from(ret_ty).ok()),
                raises: raises.iter().map(|ast| ASTTy::from((ast, finished))).collect(),
                body: body.clone().map(|ast| ASTTy::from((ast, finished))).map(Box::from),
            },
            Node::AnonFun { args, body } => NodeTy::AnonFun {
                args: args.iter().map(|ast| ASTTy::from((ast, finished))).collect(),
                body: Box::from(ASTTy::from((body, finished))),
            },
            Node::Raise { error } => NodeTy::Raise {
                error: Box::from(ASTTy::from((error, finished)))
            },
            Node::Handle { expr_or_stmt, cases } => NodeTy::Handle {
                expr_or_stmt: Box::from(ASTTy::from((expr_or_stmt, finished))),
                cases: cases.iter().map(|ast| ASTTy::from((ast, finished))).collect(),
            },
            Node::With { resource, alias, expr } => NodeTy::With {
                resource: Box::from(ASTTy::from((resource, finished))),
                alias: alias.clone().map(|(resource, alias, expr)| {
                    let resource = Box::from(ASTTy::from((resource, finished)));
                    let expr = expr.and_then(|expr| Name::try_from(&expr).ok());
                    (resource, alias, expr)
                }),
                expr: Box::from(ASTTy::from((expr, finished))),
            },
            Node::FunctionCall { name, args } => NodeTy::FunctionCall {
                name: StringName::try_from(name).unwrap_or_else(|_| StringName::empty()),
                args: args.iter().map(|ast| ASTTy::from((ast, finished))).collect(),
            },
            Node::PropertyCall { instance, property } => NodeTy::PropertyCall {
                instance: Box::from(ASTTy::from((instance, finished))),
                property: Box::from(ASTTy::from((property, finished))),
            },
            Node::ExpressionType { expr, mutable, ty } => NodeTy::ExpressionType {
                expr: Box::from(ASTTy::from((expr, finished))),
                mutable: *mutable,
                ty: if let Some(ty) = ty {
                    Name::try_from(ty).ok()
                } else {
                    pos_to_name.get(&expr.pos).cloned()
                },
            },
            Node::TypeDef { ty, isa, body } => NodeTy::TypeDef {
                ty: StringName::try_from(ty).ok().unwrap_or_else(StringName::empty),
                isa: isa.as_ref().and_then(|isa| Name::try_from(isa).ok()),
                body: body.clone().map(|ast| ASTTy::from((ast, finished))).map(Box::from),
            },
            Node::TypeAlias { ty, isa, conditions } => NodeTy::TypeAlias {
                ty: StringName::try_from(ty).unwrap_or_else(|_| StringName::empty()),
                isa: Name::try_from(isa).unwrap_or_else(|_| Name::empty()),
                conditions: conditions.iter().map(|ast| ASTTy::from((ast, finished))).collect(),
            },
            Node::Condition { cond, el } => NodeTy::Condition {
                cond: Box::from(ASTTy::from((cond, finished))),
                el: el.clone().map(|ast| ASTTy::from((ast, finished))).map(Box::from),
            },
            Node::FunArg { vararg, mutable, var, default, ty } => NodeTy::FunArg {
                vararg: *vararg,
                mutable: *mutable,
                var: Box::from(ASTTy::from((var, finished))),
                ty: ty.as_ref().and_then(|ty| Name::try_from(ty).ok()),
                default: default.clone().map(|ast| ASTTy::from((ast, finished))).map(Box::from),
            },
            Node::Dict { elements } => NodeTy::Dict {
                elements: elements.iter().map(|(from, to)|
                    (ASTTy::from(from), ASTTy::from(to))
                ).collect()
            },
            Node::DictBuilder { from, to, conditions } => NodeTy::DictBuilder {
                from: Box::from(ASTTy::from(from)),
                to: Box::from(ASTTy::from(to)),
                conditions: conditions.iter().map(ASTTy::from).collect(),
            },
            Node::Set { elements } => {
                NodeTy::Set { elements: elements.iter().map(|ast| ASTTy::from((ast, finished))).collect() }
            }
            Node::SetBuilder { item, conditions } => NodeTy::SetBuilder {
                item: Box::from(ASTTy::from((item, finished))),
                conditions: conditions.iter().map(|ast| ASTTy::from((ast, finished))).collect(),
            },
            Node::List { elements } => {
                NodeTy::List { elements: elements.iter().map(|ast| ASTTy::from((ast, finished))).collect() }
            }
            Node::ListBuilder { item, conditions } => NodeTy::ListBuilder {
                item: Box::from(ASTTy::from((item, finished))),
                conditions: conditions.iter().map(|ast| ASTTy::from((ast, finished))).collect(),
            },
            Node::Tuple { elements } => {
                NodeTy::Tuple { elements: elements.iter().map(|ast| ASTTy::from((ast, finished))).collect() }
            }
            Node::Range { from, to, inclusive, step } => NodeTy::Range {
                from: Box::from(ASTTy::from((from, finished))),
                to: Box::from(ASTTy::from((to, finished))),
                inclusive: *inclusive,
                step: step.clone().map(|ast| ASTTy::from((ast, finished))).map(Box::from),
            },
            Node::Block { statements } => {
                NodeTy::Block { statements: statements.iter().map(|ast| ASTTy::from((ast, finished))).collect() }
            }
            Node::Add { left, right } => NodeTy::Add {
                left: Box::from(ASTTy::from((left, finished))),
                right: Box::from(ASTTy::from((right, finished))),
            },
            Node::AddU { expr } => NodeTy::AddU {
                expr: Box::from(ASTTy::from((expr, finished)))
            },
            Node::Sub { left, right } => NodeTy::Sub {
                left: Box::from(ASTTy::from((left, finished))),
                right: Box::from(ASTTy::from((right, finished))),
            },
            Node::SubU { expr } => NodeTy::SubU {
                expr: Box::from(ASTTy::from((expr, finished)))
            },
            Node::Mul { left, right } => NodeTy::Mul {
                left: Box::from(ASTTy::from((left, finished))),
                right: Box::from(ASTTy::from((right, finished))),
            },
            Node::Div { left, right } => NodeTy::Div {
                left: Box::from(ASTTy::from((left, finished))),
                right: Box::from(ASTTy::from((right, finished))),
            },
            Node::FDiv { left, right } => NodeTy::FDiv {
                left: Box::from(ASTTy::from((left, finished))),
                right: Box::from(ASTTy::from((right, finished))),
            },
            Node::Mod { left, right } => NodeTy::Mod {
                left: Box::from(ASTTy::from((left, finished))),
                right: Box::from(ASTTy::from((right, finished))),
            },
            Node::Pow { left, right } => NodeTy::Pow {
                left: Box::from(ASTTy::from((left, finished))),
                right: Box::from(ASTTy::from((right, finished))),
            },
            Node::Sqrt { expr } => NodeTy::Sqrt {
                expr: Box::from(ASTTy::from((expr, finished)))
            },
            Node::BAnd { left, right } => NodeTy::BAnd {
                left: Box::from(ASTTy::from((left, finished))),
                right: Box::from(ASTTy::from((right, finished))),
            },
            Node::BOr { left, right } => NodeTy::BOr {
                left: Box::from(ASTTy::from((left, finished))),
                right: Box::from(ASTTy::from((right, finished))),
            },
            Node::BXOr { left, right } => NodeTy::BXOr {
                left: Box::from(ASTTy::from((left, finished))),
                right: Box::from(ASTTy::from((right, finished))),
            },
            Node::BOneCmpl { expr } => NodeTy::BOneCmpl {
                expr: Box::from(ASTTy::from((expr, finished)))
            },
            Node::BLShift { left, right } => NodeTy::BLShift {
                left: Box::from(ASTTy::from((left, finished))),
                right: Box::from(ASTTy::from((right, finished))),
            },
            Node::BRShift { left, right } => NodeTy::BRShift {
                left: Box::from(ASTTy::from((left, finished))),
                right: Box::from(ASTTy::from((right, finished))),
            },
            Node::Le { left, right } => NodeTy::Le {
                left: Box::from(ASTTy::from((left, finished))),
                right: Box::from(ASTTy::from((right, finished))),
            },
            Node::Ge { left, right } => NodeTy::Ge {
                left: Box::from(ASTTy::from((left, finished))),
                right: Box::from(ASTTy::from((right, finished))),
            },
            Node::Leq { left, right } => NodeTy::Leq {
                left: Box::from(ASTTy::from((left, finished))),
                right: Box::from(ASTTy::from((right, finished))),
            },
            Node::Geq { left, right } => NodeTy::Geq {
                left: Box::from(ASTTy::from((left, finished))),
                right: Box::from(ASTTy::from((right, finished))),
            },
            Node::Is { left, right } => NodeTy::Is {
                left: Box::from(ASTTy::from((left, finished))),
                right: Box::from(ASTTy::from((right, finished))),
            },
            Node::IsN { left, right } => NodeTy::IsN {
                left: Box::from(ASTTy::from((left, finished))),
                right: Box::from(ASTTy::from((right, finished))),
            },
            Node::Eq { left, right } => NodeTy::Eq {
                left: Box::from(ASTTy::from((left, finished))),
                right: Box::from(ASTTy::from((right, finished))),
            },
            Node::Neq { left, right } => NodeTy::Neq {
                left: Box::from(ASTTy::from((left, finished))),
                right: Box::from(ASTTy::from((right, finished))),
            },
            Node::IsA { left, right } => NodeTy::IsA {
                left: Box::from(ASTTy::from((left, finished))),
                right: Box::from(ASTTy::from((right, finished))),
            },
            Node::IsNA { left, right } => NodeTy::IsNA {
                left: Box::from(ASTTy::from((left, finished))),
                right: Box::from(ASTTy::from((right, finished))),
            },
            Node::Not { expr } => NodeTy::Not {
                expr: Box::from(ASTTy::from((expr, finished)))
            },
            Node::And { left, right } => NodeTy::And {
                left: Box::from(ASTTy::from((left, finished))),
                right: Box::from(ASTTy::from((right, finished))),
            },
            Node::Or { left, right } => NodeTy::Or {
                left: Box::from(ASTTy::from((left, finished))),
                right: Box::from(ASTTy::from((right, finished))),
            },
            Node::IfElse { cond, then, el } => NodeTy::IfElse {
                cond: Box::from(ASTTy::from((cond, finished))),
                then: Box::from(ASTTy::from((then, finished))),
                el: el.clone().map(|ast| ASTTy::from((ast, finished))).map(Box::from),
            },
            Node::Match { cond, cases } => NodeTy::Match {
                cond: Box::from(ASTTy::from((cond, finished))),
                cases: cases.iter().map(|ast| ASTTy::from((ast, finished))).collect(),
            },
            Node::Case { cond, body } => NodeTy::Case {
                cond: Box::from(ASTTy::from((cond, finished))),
                body: Box::from(ASTTy::from((body, finished))),
            },
            Node::For { expr, col, body } => NodeTy::For {
                expr: Box::from(ASTTy::from((expr, finished))),
                col: Box::from(ASTTy::from((col, finished))),
                body: Box::from(ASTTy::from((body, finished))),
            },
            Node::In { left, right } => NodeTy::In {
                left: Box::from(ASTTy::from((left, finished))),
                right: Box::from(ASTTy::from((right, finished))),
            },
            Node::While { cond, body } => NodeTy::While {
                cond: Box::from(ASTTy::from((cond, finished))),
                body: Box::from(ASTTy::from((body, finished))),
            },
            Node::Return { expr } => NodeTy::Return {
                expr: Box::from(ASTTy::from((expr, finished)))
            },
            Node::Question { left, right } => NodeTy::Question {
                left: Box::from(ASTTy::from((left, finished))),
                right: Box::from(ASTTy::from((right, finished))),
            },
            Node::QuestionOp { expr } => NodeTy::QuestionOp {
                expr: Box::from(ASTTy::from((expr, finished)))
            },
            Node::Id { lit } => NodeTy::Id { lit: lit.clone() },
            Node::Slice { from, to, inclusive, step } => NodeTy::Slice {
                from: Box::from(ASTTy::from((from, finished))),
                to: Box::from(ASTTy::from((to, finished))),
                inclusive: *inclusive,
                step: step.clone().map(|ast| ASTTy::from((ast, finished))).map(Box::from),
            },
            Node::Index { item, range } => NodeTy::Index {
                item: Box::from(ASTTy::from((item, finished))),
                range: if let Some(range) = range.first() {
                    Box::from(ASTTy::from((range, finished)))
                } else {
                    Box::from(ASTTy { pos: item.pos, node: NodeTy::Empty, ty: None })
                },
            },
            Node::Real { lit } => NodeTy::Real { lit: lit.clone() },
            Node::Int { lit } => NodeTy::Int { lit: lit.clone() },
            Node::ENum { num, exp } => NodeTy::ENum { num: num.clone(), exp: exp.clone() },
            Node::Str { lit, expressions } => NodeTy::Str {
                lit: lit.clone(),
                expressions: expressions.iter().map(|ast| ASTTy::from((ast, finished))).collect(),
            },
            Node::DocStr { lit } => NodeTy::DocStr { lit: lit.clone() },
            Node::Bool { lit } => NodeTy::Bool { lit: *lit },
            Node::Break => NodeTy::Break,
            Node::Continue => NodeTy::Continue,
            Node::ReturnEmpty => NodeTy::ReturnEmpty,
            Node::Underscore => NodeTy::Underscore,
            Node::Undefined => NodeTy::Undefined,
            Node::Pass => NodeTy::Pass,
            _ => NodeTy::Empty
        }
    }
}

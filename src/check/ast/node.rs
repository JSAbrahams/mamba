use std::ops::Deref;

use crate::{AST, ASTTy};
use crate::check::ast::NodeTy;
use crate::parse::ast::Node;

impl From<&Box<AST>> for NodeTy {
    fn from(ast: &Box<AST>) -> Self {
        NodeTy::from(ast.deref())
    }
}

impl From<&AST> for NodeTy {
    fn from(ast: &AST) -> Self {
        NodeTy::from(&ast.node)
    }
}

impl From<&Box<Node>> for NodeTy {
    fn from(node: &Box<Node>) -> Self {
        NodeTy::from(node.deref())
    }
}

impl From<&Node> for NodeTy {
    fn from(node: &Node) -> Self {
        match node {
            Node::Import { from, import, alias } => NodeTy::Import {
                from: from.clone().map(ASTTy::from).map(Box::from),
                import: import.iter().map(ASTTy::from).collect(),
                alias: alias.iter().map(ASTTy::from).collect(),
            },
            Node::Class { ty, args, parents, body } => NodeTy::Class {
                ty: Box::from(ASTTy::from(ty)),
                args: args.iter().map(ASTTy::from).collect(),
                parents: parents.iter().map(ASTTy::from).collect(),
                body: body.clone().map(ASTTy::from).map(Box::from),
            },
            Node::Generic { id, isa } => NodeTy::Generic {
                id: Box::from(ASTTy::from(id)),
                isa: isa.clone().map(ASTTy::from).map(Box::from),
            },
            Node::Parent { ty, args } => NodeTy::Parent {
                ty: Box::from(ASTTy::from(ty)),
                args: args.iter().map(ASTTy::from).collect(),
            },
            Node::Reassign { left, right, op } => NodeTy::Reassign {
                left: Box::from(ASTTy::from(left)),
                right: Box::from(ASTTy::from(right)),
                op: op.clone(),
            },
            Node::VariableDef { mutable, var, ty, expr: expression, forward } => {
                NodeTy::VariableDef {
                    mutable: *mutable,
                    var: Box::from(ASTTy::from(var)),
                    ty: ty.clone().map(ASTTy::from).map(Box::from),
                    expr: expression.clone().map(ASTTy::from).map(Box::from),
                    forward: forward.iter().map(ASTTy::from).collect(),
                }
            }
            Node::FunDef { pure, id, args: fun_args, ret: ret_ty, raises, body } => {
                NodeTy::FunDef {
                    pure: *pure,
                    id: Box::from(ASTTy::from(id)),
                    args: fun_args.iter().map(ASTTy::from).collect(),
                    ret: ret_ty.clone().map(ASTTy::from).map(Box::from),
                    raises: raises.iter().map(ASTTy::from).collect(),
                    body: body.clone().map(ASTTy::from).map(Box::from),
                }
            }
            Node::AnonFun { args, body } => NodeTy::AnonFun {
                args: args.iter().map(ASTTy::from).collect(),
                body: Box::from(ASTTy::from(body)),
            },
            Node::Raises { expr_or_stmt, errors } => NodeTy::Raises {
                expr_or_stmt: Box::from(ASTTy::from(expr_or_stmt)),
                errors: errors.iter().map(ASTTy::from).collect(),
            },
            Node::Raise { error } => NodeTy::Raise { error: Box::from(ASTTy::from(error)) },
            Node::Handle { expr_or_stmt, cases } => NodeTy::Handle {
                expr_or_stmt: Box::from(ASTTy::from(expr_or_stmt)),
                cases: cases.iter().map(ASTTy::from).collect(),
            },
            Node::With { resource, alias, expr } => NodeTy::With {
                resource: Box::from(ASTTy::from(resource)),
                alias: alias.clone().map(|(resource, alias, expr)| {
                    (Box::from(ASTTy::from(resource)), alias, expr.map(ASTTy::from).map(Box::from))
                }),
                expr: Box::from(ASTTy::from(expr)),
            },
            Node::FunctionCall { name, args } => NodeTy::FunctionCall {
                name: Box::from(ASTTy::from(name)),
                args: args.iter().map(ASTTy::from).collect(),
            },
            Node::PropertyCall { instance, property } => NodeTy::PropertyCall {
                instance: Box::from(ASTTy::from(instance)),
                property: Box::from(ASTTy::from(property)),
            },
            Node::ExpressionType { expr, mutable, ty } => NodeTy::ExpressionType {
                expr: Box::from(ASTTy::from(expr)),
                mutable: *mutable,
                ty: ty.clone().map(ASTTy::from).map(Box::from),
            },
            Node::TypeDef { ty, isa, body } => NodeTy::TypeDef {
                ty: Box::from(ASTTy::from(ty)),
                isa: isa.clone().map(ASTTy::from).map(Box::from),
                body: body.clone().map(ASTTy::from).map(Box::from),
            },
            Node::TypeAlias { ty, isa, conditions } => NodeTy::TypeAlias {
                ty: Box::from(ASTTy::from(ty)),
                isa: Box::from(ASTTy::from(isa)),
                conditions: conditions.iter().map(ASTTy::from).collect(),
            },
            Node::TypeTup { types } => {
                NodeTy::TypeTup { types: types.iter().map(ASTTy::from).collect() }
            }
            Node::TypeUnion { types } => {
                NodeTy::TypeUnion { types: types.iter().map(ASTTy::from).collect() }
            }
            Node::Type { id, generics } => NodeTy::Type {
                id: Box::from(ASTTy::from(id)),
                generics: generics.iter().map(ASTTy::from).collect(),
            },
            Node::TypeFun { args, ret_ty } => NodeTy::TypeFun {
                args: args.iter().map(ASTTy::from).collect(),
                ret_ty: Box::from(ASTTy::from(ret_ty)),
            },
            Node::Condition { cond, el } => NodeTy::Condition {
                cond: Box::from(ASTTy::from(cond)),
                el: el.clone().map(ASTTy::from).map(Box::from),
            },
            Node::FunArg { vararg, mutable, var, ty, default } => NodeTy::FunArg {
                vararg: *vararg,
                mutable: *mutable,
                var: Box::from(ASTTy::from(var)),
                ty: ty.clone().map(ASTTy::from).map(Box::from),
                default: default.clone().map(ASTTy::from).map(Box::from),
            },
            Node::Set { elements } => {
                NodeTy::Set { elements: elements.iter().map(ASTTy::from).collect() }
            }
            Node::SetBuilder { item, conditions } => NodeTy::SetBuilder {
                item: Box::from(ASTTy::from(item)),
                conditions: conditions.iter().map(ASTTy::from).collect(),
            },
            Node::List { elements } => {
                NodeTy::List { elements: elements.iter().map(ASTTy::from).collect() }
            }
            Node::ListBuilder { item, conditions } => NodeTy::ListBuilder {
                item: Box::from(ASTTy::from(item)),
                conditions: conditions.iter().map(ASTTy::from).collect(),
            },
            Node::Tuple { elements } => {
                NodeTy::Tuple { elements: elements.iter().map(ASTTy::from).collect() }
            }
            Node::Range { from, to, inclusive, step } => NodeTy::Range {
                from: Box::from(ASTTy::from(from)),
                to: Box::from(ASTTy::from(to)),
                inclusive: *inclusive,
                step: step.clone().map(ASTTy::from).map(Box::from),
            },
            Node::Block { statements } => {
                NodeTy::Block { statements: statements.iter().map(ASTTy::from).collect() }
            }
            Node::Add { left, right } => NodeTy::Add {
                left: Box::from(ASTTy::from(left)),
                right: Box::from(ASTTy::from(right)),
            },
            Node::AddU { expr } => NodeTy::AddU { expr: Box::from(ASTTy::from(expr)) },
            Node::Sub { left, right } => NodeTy::Sub {
                left: Box::from(ASTTy::from(left)),
                right: Box::from(ASTTy::from(right)),
            },
            Node::SubU { expr } => NodeTy::SubU { expr: Box::from(ASTTy::from(expr)) },
            Node::Mul { left, right } => NodeTy::Mul {
                left: Box::from(ASTTy::from(left)),
                right: Box::from(ASTTy::from(right)),
            },
            Node::Div { left, right } => NodeTy::Div {
                left: Box::from(ASTTy::from(left)),
                right: Box::from(ASTTy::from(right)),
            },
            Node::FDiv { left, right } => NodeTy::FDiv {
                left: Box::from(ASTTy::from(left)),
                right: Box::from(ASTTy::from(right)),
            },
            Node::Mod { left, right } => NodeTy::Mod {
                left: Box::from(ASTTy::from(left)),
                right: Box::from(ASTTy::from(right)),
            },
            Node::Pow { left, right } => NodeTy::Pow {
                left: Box::from(ASTTy::from(left)),
                right: Box::from(ASTTy::from(right)),
            },
            Node::Sqrt { expr } => NodeTy::Sqrt { expr: Box::from(ASTTy::from(expr)) },
            Node::BAnd { left, right } => NodeTy::BAnd {
                left: Box::from(ASTTy::from(left)),
                right: Box::from(ASTTy::from(right)),
            },
            Node::BOr { left, right } => NodeTy::BOr {
                left: Box::from(ASTTy::from(left)),
                right: Box::from(ASTTy::from(right)),
            },
            Node::BXOr { left, right } => NodeTy::BXOr {
                left: Box::from(ASTTy::from(left)),
                right: Box::from(ASTTy::from(right)),
            },
            Node::BOneCmpl { expr } => NodeTy::BOneCmpl { expr: Box::from(ASTTy::from(expr)) },
            Node::BLShift { left, right } => NodeTy::BLShift {
                left: Box::from(ASTTy::from(left)),
                right: Box::from(ASTTy::from(right)),
            },
            Node::BRShift { left, right } => NodeTy::BRShift {
                left: Box::from(ASTTy::from(left)),
                right: Box::from(ASTTy::from(right)),
            },
            Node::Le { left, right } => NodeTy::Le {
                left: Box::from(ASTTy::from(left)),
                right: Box::from(ASTTy::from(right)),
            },
            Node::Ge { left, right } => NodeTy::Ge {
                left: Box::from(ASTTy::from(left)),
                right: Box::from(ASTTy::from(right)),
            },
            Node::Leq { left, right } => NodeTy::Leq {
                left: Box::from(ASTTy::from(left)),
                right: Box::from(ASTTy::from(right)),
            },
            Node::Geq { left, right } => NodeTy::Geq {
                left: Box::from(ASTTy::from(left)),
                right: Box::from(ASTTy::from(right)),
            },
            Node::Is { left, right } => NodeTy::Is {
                left: Box::from(ASTTy::from(left)),
                right: Box::from(ASTTy::from(right)),
            },
            Node::IsN { left, right } => NodeTy::IsN {
                left: Box::from(ASTTy::from(left)),
                right: Box::from(ASTTy::from(right)),
            },
            Node::Eq { left, right } => NodeTy::Eq {
                left: Box::from(ASTTy::from(left)),
                right: Box::from(ASTTy::from(right)),
            },
            Node::Neq { left, right } => NodeTy::Neq {
                left: Box::from(ASTTy::from(left)),
                right: Box::from(ASTTy::from(right)),
            },
            Node::IsA { left, right } => NodeTy::IsA {
                left: Box::from(ASTTy::from(left)),
                right: Box::from(ASTTy::from(right)),
            },
            Node::IsNA { left, right } => NodeTy::IsNA {
                left: Box::from(ASTTy::from(left)),
                right: Box::from(ASTTy::from(right)),
            },
            Node::Not { expr } => NodeTy::Not { expr: Box::from(ASTTy::from(expr)) },
            Node::And { left, right } => NodeTy::And {
                left: Box::from(ASTTy::from(left)),
                right: Box::from(ASTTy::from(right)),
            },
            Node::Or { left, right } => NodeTy::Or {
                left: Box::from(ASTTy::from(left)),
                right: Box::from(ASTTy::from(right)),
            },
            Node::IfElse { cond, then, el } => NodeTy::IfElse {
                cond: Box::from(ASTTy::from(cond)),
                then: Box::from(ASTTy::from(then)),
                el: el.clone().map(ASTTy::from).map(Box::from),
            },
            Node::Match { cond, cases } => NodeTy::Match {
                cond: Box::from(ASTTy::from(cond)),
                cases: cases.iter().map(ASTTy::from).collect(),
            },
            Node::Case { cond, body } => NodeTy::Case {
                cond: Box::from(ASTTy::from(cond)),
                body: Box::from(ASTTy::from(body)),
            },
            Node::For { expr, col, body } => NodeTy::For {
                expr: Box::from(ASTTy::from(expr)),
                col: Box::from(ASTTy::from(col)),
                body: Box::from(ASTTy::from(body)),
            },
            Node::In { left, right } => NodeTy::In {
                left: Box::from(ASTTy::from(left)),
                right: Box::from(ASTTy::from(right)),
            },
            Node::While { cond, body } => NodeTy::While {
                cond: Box::from(ASTTy::from(cond)),
                body: Box::from(ASTTy::from(body)),
            },
            Node::Return { expr } => NodeTy::Return { expr: Box::from(ASTTy::from(expr)) },
            Node::Question { left, right } => NodeTy::Question {
                left: Box::from(ASTTy::from(left)),
                right: Box::from(ASTTy::from(right)),
            },
            Node::QuestionOp { expr } => NodeTy::QuestionOp { expr: Box::from(ASTTy::from(expr)) },
            Node::Id { lit } => NodeTy::Id { lit: lit.clone() },
            Node::Slice { from, to, inclusive, step } => NodeTy::Slice {
                from: Box::from(ASTTy::from(from)),
                to: Box::from(ASTTy::from(to)),
                inclusive: *inclusive,
                step: step.clone().map(ASTTy::from).map(Box::from),
            },
            Node::Index { item, range } => NodeTy::Index {
                item: Box::from(ASTTy::from(item)),
                range: Box::from(ASTTy::from(range)),
            },
            Node::Real { lit } => NodeTy::Real { lit: lit.clone() },
            Node::Int { lit } => NodeTy::Int { lit: lit.clone() },
            Node::ENum { num, exp } => NodeTy::ENum { num: num.clone(), exp: exp.clone() },
            Node::Str { lit, expressions } => NodeTy::Str {
                lit: lit.clone(),
                expressions: expressions.iter().map(ASTTy::from).collect(),
            },
            Node::DocStr { lit } => NodeTy::DocStr { lit: lit.clone() },
            Node::Bool { lit } => NodeTy::Bool { lit: *lit },
            Node::Break => NodeTy::Break,
            Node::Continue => NodeTy::Continue,
            Node::ReturnEmpty => NodeTy::ReturnEmpty,
            Node::Underscore => NodeTy::Underscore,
            Node::Undefined => NodeTy::Undefined,
            Node::Pass => NodeTy::Pass,
            Node::Comment { comment } => NodeTy::Comment { comment: comment.clone() },
        }
    }
}

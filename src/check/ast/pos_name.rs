use std::collections::HashMap;

use crate::{AST, ASTTy};
use crate::check::ast::NodeTy;
use crate::check::name::Name;
use crate::common::position::Position;

type PosNameMap = HashMap<Position, Name>;

impl From<(&AST, &PosNameMap)> for ASTTy {
    fn from((ast, names): (&AST, &PosNameMap)) -> Self {
        let ast_ty = ASTTy { pos: ast.pos.clone(), node: NodeTy::from(&ast.node), ty: None };
        ast_ty.map_ty(names)
    }
}

impl ASTTy {
    fn map_ty(&self, names: &PosNameMap) -> Self {
        names.iter().fold(self.clone(), |acc, (pos, name)| {
            if pos == &acc.pos {
                acc.to_ty(name)
            } else if acc.pos.start <= pos.start && pos.end <= acc.pos.end {
                ASTTy { pos: acc.pos, node: NodeTy::from((&acc.node, names)), ty: acc.ty }
            } else {
                acc
            }
        })
    }
}

impl From<(&NodeTy, &PosNameMap)> for NodeTy {
    fn from((node_ty, names): (&NodeTy, &PosNameMap)) -> Self {
        match node_ty {
            NodeTy::File { pure, statements } => NodeTy::File {
                pure: *pure,
                statements: statements.iter().map(|stmt| stmt.map_ty(names)).collect(),
            },
            NodeTy::Import { import, aliases } => NodeTy::Import {
                import: import.iter().map(|i| i.map_ty(names)).collect(),
                aliases: aliases.iter().map(|a| a.map_ty(names)).collect(),
            },
            NodeTy::FromImport { id, import } => NodeTy::FromImport {
                id: Box::from(id.map_ty(names)),
                import: Box::from(import.map_ty(names)),
            },
            NodeTy::Class { ty, args, parents, body } => NodeTy::Class {
                ty: Box::from(ty.map_ty(names)),
                args: args.iter().map(|a| a.map_ty(names)).collect(),
                parents: parents.iter().map(|p| p.map_ty(names)).collect(),
                body: body.clone().map(|b| Box::from(b.map_ty(names))),
            },
            NodeTy::Generic { id, isa } => NodeTy::Generic {
                id: Box::from(id.map_ty(names)),
                isa: isa.clone().map(|isa| Box::from(isa.map_ty(names))),
            },
            NodeTy::Parent { ty, args } => NodeTy::Parent {
                ty: Box::from(ty.map_ty(names)),
                args: args.iter().map(|a| a.map_ty(names)).collect(),
            },
            NodeTy::Reassign { left, right, op } => NodeTy::Reassign {
                left: Box::from(left.map_ty(names)),
                right: Box::from(right.map_ty(names)),
                op: op.clone(),
            },
            NodeTy::VariableDef { mutable, var, ty, expr: expression, forward } => NodeTy::VariableDef {
                mutable: *mutable,
                var: Box::from(var.map_ty(names)),
                ty: ty.clone().map(|t| Box::from(t.map_ty(names))),
                expr: expression.clone().map(|e| Box::from(e.map_ty(names))),
                forward: forward.iter().map(|f| f.map_ty(names)).collect(),
            },
            NodeTy::FunDef { pure, id, args: fun_args, ret: ret_ty, raises, body } => NodeTy::FunDef {
                pure: *pure,
                id: Box::from(id.map_ty(names)),
                args: fun_args.iter().map(|a| a.map_ty(names)).collect(),
                ret: ret_ty.clone().map(|r| Box::from(r.map_ty(names))),
                raises: raises.iter().map(|r| r.map_ty(names)).collect(),
                body: body.clone().map(|b| Box::from(b.map_ty(names))),
            },
            NodeTy::AnonFun { args, body } => NodeTy::AnonFun {
                args: args.iter().map(|a| a.map_ty(names)).collect(),
                body: Box::from(body.map_ty(names)),
            },
            NodeTy::Raises { expr_or_stmt, errors } => NodeTy::Raises {
                expr_or_stmt: Box::from(expr_or_stmt.map_ty(names)),
                errors: errors.iter().map(|e| e.map_ty(names)).collect(),
            },
            NodeTy::Raise { error } => NodeTy::Raise { error: Box::from(error.map_ty(names)) },
            NodeTy::Handle { expr_or_stmt, cases } => NodeTy::Handle {
                expr_or_stmt: Box::from(expr_or_stmt.map_ty(names)),
                cases: cases.iter().map(|c| c.map_ty(names)).collect(),
            },
            NodeTy::With { resource, alias, expr } => NodeTy::With {
                resource: Box::from(resource.map_ty(names)),
                alias: alias.clone().map(|(resource, alias, expr)| (
                    Box::from(resource.map_ty(names)),
                    alias,
                    expr.map(|expr| Box::from(expr.map_ty(names)))
                )),
                expr: Box::from(expr.map_ty(names)),
            },
            NodeTy::FunctionCall { name, args } => NodeTy::FunctionCall {
                name: Box::from(name.map_ty(names)),
                args: args.iter().map(|a| a.map_ty(names)).collect(),
            },
            NodeTy::PropertyCall { instance, property } => NodeTy::PropertyCall {
                instance: Box::from(instance.map_ty(names)),
                property: Box::from(property.map_ty(names)),
            },
            NodeTy::ExpressionType { expr, mutable, ty } => NodeTy::ExpressionType {
                expr: Box::from(expr.map_ty(names)),
                mutable: *mutable,
                ty: ty.clone().map(|ty| Box::from(ty.map_ty(names))),
            },
            NodeTy::TypeDef { ty, isa, body } => NodeTy::TypeDef {
                ty: Box::from(ty.map_ty(names)),
                isa: isa.clone().map(|isa| Box::from(isa.map_ty(names))),
                body: body.clone().map(|body| Box::from(body.map_ty(names))),
            },
            NodeTy::TypeAlias { ty, isa, conditions } => NodeTy::TypeAlias {
                ty: Box::from(ty.map_ty(names)),
                isa: Box::from(isa.map_ty(names)),
                conditions: conditions.iter().map(|c| c.map_ty(names)).collect(),
            },
            NodeTy::TypeTup { types } => NodeTy::TypeTup {
                types: types.iter().map(|ty| ty.map_ty(names)).collect()
            },
            NodeTy::TypeUnion { types } => NodeTy::TypeUnion {
                types: types.iter().map(|ty| ty.map_ty(names)).collect()
            },
            NodeTy::Type { id, generics } => NodeTy::Type {
                id: Box::from(id.map_ty(names)),
                generics: generics.iter().map(|gen| gen.map_ty(names)).collect(),
            },
            NodeTy::TypeFun { args, ret_ty } => NodeTy::TypeFun {
                args: args.iter().map(|arg| arg.map_ty(names)).collect(),
                ret_ty: Box::from(ret_ty.map_ty(names)),
            },
            NodeTy::Condition { cond, el } => NodeTy::Condition {
                cond: Box::from(cond.map_ty(names)),
                el: el.clone().map(|el| Box::from(el.map_ty(names))),
            },
            NodeTy::FunArg { vararg, mutable, var, ty, default } => NodeTy::FunArg {
                vararg: *vararg,
                mutable: *mutable,
                var: Box::from(var.map_ty(names)),
                ty: ty.clone().map(|ty| Box::from(ty.map_ty(names))),
                default: default.clone().map(|d| Box::from(d.map_ty(names))),
            },
            NodeTy::Set { elements } => NodeTy::Set {
                elements: elements.iter().map(|e| e.map_ty(names)).collect()
            },
            NodeTy::SetBuilder { item, conditions } => NodeTy::SetBuilder {
                item: Box::from(item.map_ty(names)),
                conditions: conditions.iter().map(|cond| cond.map_ty(names)).collect(),
            },
            NodeTy::List { elements } => NodeTy::List {
                elements: elements.iter().map(|e| e.map_ty(names)).collect()
            },
            NodeTy::ListBuilder { item, conditions } => NodeTy::ListBuilder {
                item: Box::from(item.map_ty(names)),
                conditions: conditions.iter().map(|cond| cond.map_ty(names)).collect(),
            },
            NodeTy::Tuple { elements } => NodeTy::Tuple {
                elements: elements.iter().map(|e| e.map_ty(names)).collect()
            },
            NodeTy::Range { from, to, inclusive, step } => NodeTy::Range {
                from: Box::from(from.map_ty(names)),
                to: Box::from(to.map_ty(names)),
                inclusive: *inclusive,
                step: step.clone().map(|ast| Box::from(ast.map_ty(names))),
            },
            NodeTy::Block { statements } => NodeTy::Block {
                statements: statements.iter().map(|stmt| stmt.map_ty(names)).collect()
            },
            NodeTy::Add { left, right } => NodeTy::Add { left: Box::from(left.map_ty(names)), right: Box::from(right.map_ty(names)) },
            NodeTy::AddU { expr } => NodeTy::AddU {
                expr: Box::from(expr.map_ty(names))
            },
            NodeTy::Sub { left, right } => NodeTy::Sub {
                left: Box::from(left.map_ty(names)),
                right: Box::from(right.map_ty(names)),
            },
            NodeTy::SubU { expr } => NodeTy::SubU {
                expr: Box::from(expr.map_ty(names))
            },
            NodeTy::Mul { left, right } => NodeTy::Mul {
                left: Box::from(left.map_ty(names)),
                right: Box::from(right.map_ty(names)),
            },
            NodeTy::Div { left, right } => NodeTy::Div {
                left: Box::from(left.map_ty(names)),
                right: Box::from(right.map_ty(names)),
            },
            NodeTy::FDiv { left, right } => NodeTy::FDiv {
                left: Box::from(left.map_ty(names)),
                right: Box::from(right.map_ty(names)),
            },
            NodeTy::Mod { left, right } => NodeTy::Mod {
                left: Box::from(left.map_ty(names)),
                right: Box::from(right.map_ty(names)),
            },
            NodeTy::Pow { left, right } => NodeTy::Pow {
                left: Box::from(left.map_ty(names)),
                right: Box::from(right.map_ty(names)),
            },
            NodeTy::Sqrt { expr } => NodeTy::Sqrt { expr: Box::from(expr.map_ty(names)) },
            NodeTy::BAnd { left, right } => NodeTy::BAnd {
                left: Box::from(left.map_ty(names)),
                right: Box::from(right.map_ty(names)),
            },
            NodeTy::BOr { left, right } => NodeTy::BOr {
                left: Box::from(left.map_ty(names)),
                right: Box::from(right.map_ty(names)),
            },
            NodeTy::BXOr { left, right } => NodeTy::BXOr {
                left: Box::from(left.map_ty(names)),
                right: Box::from(right.map_ty(names)),
            },
            NodeTy::BOneCmpl { expr } => NodeTy::BOneCmpl { expr: Box::from(expr.map_ty(names)) },
            NodeTy::BLShift { left, right } => NodeTy::BLShift {
                left: Box::from(left.map_ty(names)),
                right: Box::from(right.map_ty(names)),
            },
            NodeTy::BRShift { left, right } => NodeTy::BRShift {
                left: Box::from(left.map_ty(names)),
                right: Box::from(right.map_ty(names)),
            },
            NodeTy::Le { left, right } => NodeTy::Le {
                left: Box::from(left.map_ty(names)),
                right: Box::from(right.map_ty(names)),
            },
            NodeTy::Ge { left, right } => NodeTy::Ge {
                left: Box::from(left.map_ty(names)),
                right: Box::from(right.map_ty(names)),
            },
            NodeTy::Leq { left, right } => NodeTy::Leq {
                left: Box::from(left.map_ty(names)),
                right: Box::from(right.map_ty(names)),
            },
            NodeTy::Geq { left, right } => NodeTy::Geq {
                left: Box::from(left.map_ty(names)),
                right: Box::from(right.map_ty(names)),
            },
            NodeTy::Is { left, right } => NodeTy::Is {
                left: Box::from(left.map_ty(names)),
                right: Box::from(right.map_ty(names)),
            },
            NodeTy::IsN { left, right } => NodeTy::IsN {
                left: Box::from(left.map_ty(names)),
                right: Box::from(right.map_ty(names)),
            },
            NodeTy::Eq { left, right } => NodeTy::Eq {
                left: Box::from(left.map_ty(names)),
                right: Box::from(right.map_ty(names)),
            },
            NodeTy::Neq { left, right } => NodeTy::Neq {
                left: Box::from(left.map_ty(names)),
                right: Box::from(right.map_ty(names)),
            },
            NodeTy::IsA { left, right } => NodeTy::IsA {
                left: Box::from(left.map_ty(names)),
                right: Box::from(right.map_ty(names)),
            },
            NodeTy::IsNA { left, right } => NodeTy::IsNA {
                left: Box::from(left.map_ty(names)),
                right: Box::from(right.map_ty(names)),
            },
            NodeTy::Not { expr } => NodeTy::Not { expr: Box::from(expr.map_ty(names)) },
            NodeTy::And { left, right } => NodeTy::And {
                left: Box::from(left.map_ty(names)),
                right: Box::from(right.map_ty(names)),
            },
            NodeTy::Or { left, right } => NodeTy::Or {
                left: Box::from(left.map_ty(names)),
                right: Box::from(right.map_ty(names)),
            },
            NodeTy::IfElse { cond, then, el } => NodeTy::IfElse {
                cond: Box::from(cond.map_ty(names)),
                then: Box::from(then.map_ty(names)),
                el: el.clone().map(|el| Box::from(el.map_ty(names))),
            },
            NodeTy::Match { cond, cases } => NodeTy::Match {
                cond: Box::from(cond.map_ty(names)),
                cases: cases.iter().map(|c| c.map_ty(names)).collect(),
            },
            NodeTy::Case { cond, body } => NodeTy::Case {
                cond: Box::from(cond.map_ty(names)),
                body: Box::from(body.map_ty(names)),
            },
            NodeTy::For { expr, col, body } => NodeTy::For {
                expr: Box::from(expr.map_ty(names)),
                col: Box::from(col.map_ty(names)),
                body: Box::from(body.map_ty(names)),
            },
            NodeTy::In { left, right } => NodeTy::In {
                left: Box::from(left.map_ty(names)),
                right: Box::from(right.map_ty(names)),
            },
            NodeTy::While { cond, body } => NodeTy::While {
                cond: Box::from(cond.map_ty(names)),
                body: Box::from(body.map_ty(names)),
            },
            NodeTy::Return { expr } => NodeTy::Return {
                expr: Box::from(expr.map_ty(names))
            },
            NodeTy::Question { left, right } => NodeTy::Question {
                left: Box::from(left.map_ty(names)),
                right: Box::from(right.map_ty(names)),
            },
            NodeTy::QuestionOp { expr } => NodeTy::QuestionOp { expr: Box::from(expr.map_ty(names)) },

            NodeTy::Id { lit } => NodeTy::Id { lit: lit.clone() },
            NodeTy::Slice { from, to, inclusive, step } => NodeTy::Slice {
                from: Box::from(from.map_ty(names)),
                to: Box::from(to.map_ty(names)),
                inclusive: *inclusive,
                step: step.clone().map(|step| step.map_ty(names)).map(Box::from),
            },
            NodeTy::Index { item, range } => NodeTy::Index {
                item: Box::from(item.map_ty(names)),
                range: Box::from(range.map_ty(names)),
            },
            NodeTy::Str { lit, expressions } => NodeTy::Str {
                lit: lit.clone(),
                expressions: expressions.iter().map(|expr| expr.map_ty(names)).collect(),
            },

            other => other.clone()
        }
    }
}

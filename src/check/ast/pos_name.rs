use std::collections::HashMap;

use crate::{AST, ASTTy};
use crate::check::ast::NodeTy;
use crate::check::name::Name;
use crate::common::position::Position;

pub type PosNameMap = HashMap<Position, Name>;

impl From<(&AST, &PosNameMap)> for ASTTy {
    fn from((ast, names): (&AST, &PosNameMap)) -> Self {
        let ast_ty = ASTTy { pos: ast.pos, node: NodeTy::from(&ast.node), ty: None };
        ast_ty.map(names)
    }
}

impl ASTTy {
    fn map(&self, names: &PosNameMap) -> Self {
        for (pos, name) in names {
            if pos == &self.pos {
                trace!("Annotated AST at {} with '{}'", self.pos, name);
                let mut new_names = names.clone();
                new_names.remove(pos);

                let node = NodeTy::from((&self.node, &new_names));
                return ASTTy { node, ty: Some(name.clone()), ..self.clone() };
            }
        }

        ASTTy { node: NodeTy::from((&self.node, names)), ..self.clone() }
    }
}

impl From<(&NodeTy, &PosNameMap)> for NodeTy {
    fn from((node_ty, names): (&NodeTy, &PosNameMap)) -> Self {
        match node_ty {
            NodeTy::Import { from, import, alias } => NodeTy::Import {
                from: from.clone().map(|ast| ast.map(names)).map(Box::from),
                import: import.iter().map(|i| i.map(names)).collect(),
                alias: alias.iter().map(|a| a.map(names)).collect(),
            },
            NodeTy::Class { ty, args, parents, body } => NodeTy::Class {
                ty: Box::from(ty.map(names)),
                args: args.iter().map(|a| a.map(names)).collect(),
                parents: parents.iter().map(|p| p.map(names)).collect(),
                body: body.clone().map(|b| Box::from(b.map(names))),
            },
            NodeTy::Parent { ty, args } => NodeTy::Parent {
                ty: Box::from(ty.map(names)),
                args: args.iter().map(|a| a.map(names)).collect(),
            },
            NodeTy::Reassign { left, right, op } => NodeTy::Reassign {
                left: Box::from(left.map(names)),
                right: Box::from(right.map(names)),
                op: op.clone(),
            },
            NodeTy::VariableDef { mutable, var, ty, expr: expression, forward } => NodeTy::VariableDef {
                mutable: *mutable,
                var: Box::from(var.map(names)),
                ty: ty.clone().map(|t| Box::from(t.map(names))),
                expr: expression.clone().map(|e| Box::from(e.map(names))),
                forward: forward.iter().map(|f| f.map(names)).collect(),
            },
            NodeTy::FunDef { pure, id, args: fun_args, ret: ret_ty, raises, body } => NodeTy::FunDef {
                pure: *pure,
                id: Box::from(id.map(names)),
                args: fun_args.iter().map(|a| a.map(names)).collect(),
                ret: ret_ty.clone().map(|r| Box::from(r.map(names))),
                raises: raises.iter().map(|r| r.map(names)).collect(),
                body: body.clone().map(|b| Box::from(b.map(names))),
            },
            NodeTy::AnonFun { args, body } => NodeTy::AnonFun {
                args: args.iter().map(|a| a.map(names)).collect(),
                body: Box::from(body.map(names)),
            },
            NodeTy::Raises { expr_or_stmt, errors } => NodeTy::Raises {
                expr_or_stmt: Box::from(expr_or_stmt.map(names)),
                errors: errors.iter().map(|e| e.map(names)).collect(),
            },
            NodeTy::Raise { error } => NodeTy::Raise { error: Box::from(error.map(names)) },
            NodeTy::Handle { expr_or_stmt, cases } => NodeTy::Handle {
                expr_or_stmt: Box::from(expr_or_stmt.map(names)),
                cases: cases.iter().map(|c| c.map(names)).collect(),
            },
            NodeTy::With { resource, alias, expr } => NodeTy::With {
                resource: Box::from(resource.map(names)),
                alias: alias.clone().map(|(resource, alias, expr)| (
                    Box::from(resource.map(names)),
                    alias,
                    expr.map(|expr| Box::from(expr.map(names)))
                )),
                expr: Box::from(expr.map(names)),
            },
            NodeTy::FunctionCall { name, args } => NodeTy::FunctionCall {
                name: Box::from(name.map(names)),
                args: args.iter().map(|a| a.map(names)).collect(),
            },
            NodeTy::PropertyCall { instance, property } => NodeTy::PropertyCall {
                instance: Box::from(instance.map(names)),
                property: Box::from(property.map(names)),
            },
            NodeTy::ExpressionType { expr, mutable, ty } => NodeTy::ExpressionType {
                expr: Box::from(expr.map(names)),
                mutable: *mutable,
                ty: ty.clone().map(|ty| Box::from(ty.map(names))),
            },
            NodeTy::TypeDef { ty, isa, body } => NodeTy::TypeDef {
                ty: Box::from(ty.map(names)),
                isa: isa.clone().map(|isa| Box::from(isa.map(names))),
                body: body.clone().map(|body| Box::from(body.map(names))),
            },
            NodeTy::TypeAlias { ty, isa, conditions } => NodeTy::TypeAlias {
                ty: Box::from(ty.map(names)),
                isa: Box::from(isa.map(names)),
                conditions: conditions.iter().map(|c| c.map(names)).collect(),
            },
            NodeTy::TypeTup { types } => NodeTy::TypeTup {
                types: types.iter().map(|ty| ty.map(names)).collect()
            },
            NodeTy::TypeUnion { types } => NodeTy::TypeUnion {
                types: types.iter().map(|ty| ty.map(names)).collect()
            },
            NodeTy::Type { id, generics } => NodeTy::Type {
                id: Box::from(id.map(names)),
                generics: generics.iter().map(|gen| gen.map(names)).collect(),
            },
            NodeTy::TypeFun { args, ret_ty } => NodeTy::TypeFun {
                args: args.iter().map(|arg| arg.map(names)).collect(),
                ret_ty: Box::from(ret_ty.map(names)),
            },
            NodeTy::Condition { cond, el } => NodeTy::Condition {
                cond: Box::from(cond.map(names)),
                el: el.clone().map(|el| Box::from(el.map(names))),
            },
            NodeTy::FunArg { vararg, mutable, var, ty, default } => NodeTy::FunArg {
                vararg: *vararg,
                mutable: *mutable,
                var: Box::from(var.map(names)),
                ty: ty.clone().map(|ty| Box::from(ty.map(names))),
                default: default.clone().map(|d| Box::from(d.map(names))),
            },
            NodeTy::Set { elements } => NodeTy::Set {
                elements: elements.iter().map(|e| e.map(names)).collect()
            },
            NodeTy::SetBuilder { item, conditions } => NodeTy::SetBuilder {
                item: Box::from(item.map(names)),
                conditions: conditions.iter().map(|cond| cond.map(names)).collect(),
            },
            NodeTy::List { elements } => NodeTy::List {
                elements: elements.iter().map(|e| e.map(names)).collect()
            },
            NodeTy::ListBuilder { item, conditions } => NodeTy::ListBuilder {
                item: Box::from(item.map(names)),
                conditions: conditions.iter().map(|cond| cond.map(names)).collect(),
            },
            NodeTy::Tuple { elements } => NodeTy::Tuple {
                elements: elements.iter().map(|e| e.map(names)).collect()
            },
            NodeTy::Range { from, to, inclusive, step } => NodeTy::Range {
                from: Box::from(from.map(names)),
                to: Box::from(to.map(names)),
                inclusive: *inclusive,
                step: step.clone().map(|ast| Box::from(ast.map(names))),
            },
            NodeTy::Block { statements } => NodeTy::Block {
                statements: statements.iter().map(|stmt| stmt.map(names)).collect()
            },
            NodeTy::Add { left, right } => NodeTy::Add { left: Box::from(left.map(names)), right: Box::from(right.map(names)) },
            NodeTy::AddU { expr } => NodeTy::AddU {
                expr: Box::from(expr.map(names))
            },
            NodeTy::Sub { left, right } => NodeTy::Sub {
                left: Box::from(left.map(names)),
                right: Box::from(right.map(names)),
            },
            NodeTy::SubU { expr } => NodeTy::SubU {
                expr: Box::from(expr.map(names))
            },
            NodeTy::Mul { left, right } => NodeTy::Mul {
                left: Box::from(left.map(names)),
                right: Box::from(right.map(names)),
            },
            NodeTy::Div { left, right } => NodeTy::Div {
                left: Box::from(left.map(names)),
                right: Box::from(right.map(names)),
            },
            NodeTy::FDiv { left, right } => NodeTy::FDiv {
                left: Box::from(left.map(names)),
                right: Box::from(right.map(names)),
            },
            NodeTy::Mod { left, right } => NodeTy::Mod {
                left: Box::from(left.map(names)),
                right: Box::from(right.map(names)),
            },
            NodeTy::Pow { left, right } => NodeTy::Pow {
                left: Box::from(left.map(names)),
                right: Box::from(right.map(names)),
            },
            NodeTy::Sqrt { expr } => NodeTy::Sqrt { expr: Box::from(expr.map(names)) },
            NodeTy::BAnd { left, right } => NodeTy::BAnd {
                left: Box::from(left.map(names)),
                right: Box::from(right.map(names)),
            },
            NodeTy::BOr { left, right } => NodeTy::BOr {
                left: Box::from(left.map(names)),
                right: Box::from(right.map(names)),
            },
            NodeTy::BXOr { left, right } => NodeTy::BXOr {
                left: Box::from(left.map(names)),
                right: Box::from(right.map(names)),
            },
            NodeTy::BOneCmpl { expr } => NodeTy::BOneCmpl { expr: Box::from(expr.map(names)) },
            NodeTy::BLShift { left, right } => NodeTy::BLShift {
                left: Box::from(left.map(names)),
                right: Box::from(right.map(names)),
            },
            NodeTy::BRShift { left, right } => NodeTy::BRShift {
                left: Box::from(left.map(names)),
                right: Box::from(right.map(names)),
            },
            NodeTy::Le { left, right } => NodeTy::Le {
                left: Box::from(left.map(names)),
                right: Box::from(right.map(names)),
            },
            NodeTy::Ge { left, right } => NodeTy::Ge {
                left: Box::from(left.map(names)),
                right: Box::from(right.map(names)),
            },
            NodeTy::Leq { left, right } => NodeTy::Leq {
                left: Box::from(left.map(names)),
                right: Box::from(right.map(names)),
            },
            NodeTy::Geq { left, right } => NodeTy::Geq {
                left: Box::from(left.map(names)),
                right: Box::from(right.map(names)),
            },
            NodeTy::Is { left, right } => NodeTy::Is {
                left: Box::from(left.map(names)),
                right: Box::from(right.map(names)),
            },
            NodeTy::IsN { left, right } => NodeTy::IsN {
                left: Box::from(left.map(names)),
                right: Box::from(right.map(names)),
            },
            NodeTy::Eq { left, right } => NodeTy::Eq {
                left: Box::from(left.map(names)),
                right: Box::from(right.map(names)),
            },
            NodeTy::Neq { left, right } => NodeTy::Neq {
                left: Box::from(left.map(names)),
                right: Box::from(right.map(names)),
            },
            NodeTy::IsA { left, right } => NodeTy::IsA {
                left: Box::from(left.map(names)),
                right: Box::from(right.map(names)),
            },
            NodeTy::IsNA { left, right } => NodeTy::IsNA {
                left: Box::from(left.map(names)),
                right: Box::from(right.map(names)),
            },
            NodeTy::Not { expr } => NodeTy::Not { expr: Box::from(expr.map(names)) },
            NodeTy::And { left, right } => NodeTy::And {
                left: Box::from(left.map(names)),
                right: Box::from(right.map(names)),
            },
            NodeTy::Or { left, right } => NodeTy::Or {
                left: Box::from(left.map(names)),
                right: Box::from(right.map(names)),
            },
            NodeTy::IfElse { cond, then, el } => NodeTy::IfElse {
                cond: Box::from(cond.map(names)),
                then: Box::from(then.map(names)),
                el: el.clone().map(|el| Box::from(el.map(names))),
            },
            NodeTy::Match { cond, cases } => NodeTy::Match {
                cond: Box::from(cond.map(names)),
                cases: cases.iter().map(|c| c.map(names)).collect(),
            },
            NodeTy::Case { cond, body } => NodeTy::Case {
                cond: Box::from(cond.map(names)),
                body: Box::from(body.map(names)),
            },
            NodeTy::For { expr, col, body } => NodeTy::For {
                expr: Box::from(expr.map(names)),
                col: Box::from(col.map(names)),
                body: Box::from(body.map(names)),
            },
            NodeTy::In { left, right } => NodeTy::In {
                left: Box::from(left.map(names)),
                right: Box::from(right.map(names)),
            },
            NodeTy::While { cond, body } => NodeTy::While {
                cond: Box::from(cond.map(names)),
                body: Box::from(body.map(names)),
            },
            NodeTy::Return { expr } => NodeTy::Return {
                expr: Box::from(expr.map(names))
            },
            NodeTy::Question { left, right } => NodeTy::Question {
                left: Box::from(left.map(names)),
                right: Box::from(right.map(names)),
            },
            NodeTy::QuestionOp { expr } => NodeTy::QuestionOp { expr: Box::from(expr.map(names)) },

            NodeTy::Id { lit } => NodeTy::Id { lit: lit.clone() },
            NodeTy::Slice { from, to, inclusive, step } => NodeTy::Slice {
                from: Box::from(from.map(names)),
                to: Box::from(to.map(names)),
                inclusive: *inclusive,
                step: step.clone().map(|step| step.map(names)).map(Box::from),
            },
            NodeTy::Index { item, range } => NodeTy::Index {
                item: Box::from(item.map(names)),
                range: Box::from(range.map(names)),
            },
            NodeTy::Str { lit, expressions } => NodeTy::Str {
                lit: lit.clone(),
                expressions: expressions.iter().map(|expr| expr.map(names)).collect(),
            },

            other => other.clone()
        }
    }
}

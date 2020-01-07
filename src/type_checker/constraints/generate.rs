use std::ops::Deref;

use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::cons::Constraints;
use crate::type_checker::constraints::cons::Expect;
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::ty;
use crate::type_checker::context::Context;
use crate::type_checker::environment::Environment;
use crate::type_checker::type_name::TypeName;

pub fn generate(ast: &AST, env: &Environment, ctx: &Context, constr: &Constraints) -> Constrained {
    match &ast.node {
        Node::File { .. } => unimplemented!(),
        Node::Import { .. } => unimplemented!(),
        Node::FromImport { .. } => unimplemented!(),
        Node::Class { .. } => unimplemented!(),
        Node::Generic { .. } => unimplemented!(),
        Node::Parent { .. } => unimplemented!(),
        Node::Script { .. } => unimplemented!(),
        Node::Init => unimplemented!(),
        Node::Reassign { .. } => unimplemented!(),
        Node::VariableDef { .. } => unimplemented!(),
        Node::FunDef { .. } => unimplemented!(),
        Node::AnonFun { .. } => unimplemented!(),
        Node::Raises { .. } => unimplemented!(),
        Node::Raise { .. } => unimplemented!(),
        Node::Handle { .. } => unimplemented!(),
        Node::With { .. } => unimplemented!(),
        Node::ConstructorCall { .. } => unimplemented!(),
        Node::FunctionCall { .. } => unimplemented!(),
        Node::PropertyCall { .. } => unimplemented!(),
        Node::Id { .. } => unimplemented!(),
        Node::IdType { .. } => unimplemented!(),
        Node::TypeDef { .. } => unimplemented!(),
        Node::TypeAlias { .. } => unimplemented!(),
        Node::TypeTup { .. } => unimplemented!(),
        Node::TypeUnion { .. } => unimplemented!(),
        Node::Type { .. } => unimplemented!(),
        Node::TypeFun { .. } => unimplemented!(),
        Node::Condition { .. } => unimplemented!(),
        Node::FunArg { .. } => unimplemented!(),
        Node::_Self => unimplemented!(),
        Node::AddOp => unimplemented!(),
        Node::SubOp => unimplemented!(),
        Node::SqrtOp => unimplemented!(),
        Node::MulOp => unimplemented!(),
        Node::FDivOp => unimplemented!(),
        Node::DivOp => unimplemented!(),
        Node::PowOp => unimplemented!(),
        Node::ModOp => unimplemented!(),
        Node::EqOp => unimplemented!(),
        Node::LeOp => unimplemented!(),
        Node::GeOp => unimplemented!(),
        Node::Set { .. } => unimplemented!(),
        Node::SetBuilder { .. } => unimplemented!(),
        Node::List { .. } => unimplemented!(),
        Node::ListBuilder { .. } => unimplemented!(),
        Node::Tuple { .. } => unimplemented!(),
        Node::Range { .. } => unimplemented!(),
        Node::Block { .. } => unimplemented!(),
        Node::Real { .. } => unimplemented!(),
        Node::Int { .. } => unimplemented!(),
        Node::ENum { .. } => unimplemented!(),
        Node::Str { .. } => unimplemented!(),
        Node::DocStr { .. } => unimplemented!(),
        Node::Bool { .. } => unimplemented!(),
        Node::Add { .. } => unimplemented!(),
        Node::AddU { .. } => unimplemented!(),
        Node::Sub { .. } => unimplemented!(),
        Node::SubU { .. } => unimplemented!(),
        Node::Mul { .. } => unimplemented!(),
        Node::Div { .. } => unimplemented!(),
        Node::FDiv { .. } => unimplemented!(),
        Node::Mod { .. } => unimplemented!(),
        Node::Pow { .. } => unimplemented!(),
        Node::Sqrt { .. } => unimplemented!(),
        Node::BAnd { .. } => unimplemented!(),
        Node::BOr { .. } => unimplemented!(),
        Node::BXOr { .. } => unimplemented!(),
        Node::BOneCmpl { .. } => unimplemented!(),
        Node::BLShift { .. } => unimplemented!(),
        Node::BRShift { .. } => unimplemented!(),
        Node::Le { .. } => unimplemented!(),
        Node::Ge { .. } => unimplemented!(),
        Node::Leq { .. } => unimplemented!(),
        Node::Geq { .. } => unimplemented!(),
        Node::Is { .. } => unimplemented!(),
        Node::IsN { .. } => unimplemented!(),
        Node::Eq { .. } => unimplemented!(),
        Node::Neq { .. } => unimplemented!(),
        Node::IsA { .. } => unimplemented!(),
        Node::IsNA { .. } => unimplemented!(),
        Node::Not { .. } => unimplemented!(),
        Node::And { .. } => unimplemented!(),
        Node::Or { .. } => unimplemented!(),
        Node::IfElse { cond, then, _else } => {
            let type_name = TypeName::from(ty::concrete::BOOL_PRIMITIVE);
            let constr = constr
                .add(&Expect::Expression { ast: cond.deref().clone() }, &Expect::Type {
                    type_name
                });
            if let Some(_else) = _else {
                // TODO change constraint depending on whether we expect an expression or not
                let constr = constr
                    .add(&Expect::Expression { ast: then.deref().clone() }, &Expect::Expression {
                        ast: _else.deref().clone()
                    });
                let (constr, env) = generate(cond, env, ctx, &constr)?;
                let (constr, env) = generate(then, &env, ctx, &constr)?;
                let (constr, env) = generate(_else, &env, ctx, &constr)?;
                Ok((constr, env.clone()))
            } else {
                let constr =
                    constr.add(&Expect::Statement { ast: then.deref().clone() }, &Expect::Any);
                let (constr, env) = generate(cond, env, ctx, &constr)?;
                let (constr, env) = generate(then, &env, ctx, &constr)?;
                Ok((constr, env.clone()))
            }
        }
        Node::Match { .. } => unimplemented!(),
        Node::Case { .. } => unimplemented!(),
        Node::For { .. } => unimplemented!(),
        Node::In { .. } => unimplemented!(),
        Node::Step { .. } => unimplemented!(),
        Node::While { .. } => unimplemented!(),
        Node::Break => unimplemented!(),
        Node::Continue => unimplemented!(),
        Node::Return { .. } => unimplemented!(),
        Node::ReturnEmpty => unimplemented!(),
        Node::Underscore => unimplemented!(),
        Node::Undefined => unimplemented!(),
        Node::Pass => unimplemented!(),
        Node::Question { .. } => unimplemented!(),
        Node::QuestionOp { .. } => unimplemented!(),
        Node::Print { expr } => {
            let constr = constr
                .add(&Expect::Expression { ast: expr.deref().clone() }, &Expect::AnyExpression);
            let constr = constr.add(&Expect::Statement { ast: ast.clone() }, &Expect::AnyStatement);
            let (constr, env) = generate(expr, env, ctx, &constr)?;
            Ok((constr, env.clone()))
        }
        Node::Comment { .. } => Ok((constr.clone(), env.clone()))
    }
}

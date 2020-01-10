use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::cons::Constraints;
use crate::type_checker::constraints::generate::call::generate_call;
use crate::type_checker::constraints::generate::class::gen_class;
use crate::type_checker::constraints::generate::collection::gen_collection;
use crate::type_checker::constraints::generate::common::gen_vec;
use crate::type_checker::constraints::generate::control_flow::gen_cntrl_flow;
use crate::type_checker::constraints::generate::definition::gen_definition;
use crate::type_checker::constraints::generate::expression::gen_expression;
use crate::type_checker::constraints::generate::operation::gen_operation;
use crate::type_checker::constraints::generate::resources::gen_resources;
use crate::type_checker::constraints::generate::statement::gen_statement;
use crate::type_checker::constraints::generate::ty::gen_ty;
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::Context;
use crate::type_checker::environment::Environment;

mod call;
mod class;
mod collection;
mod control_flow;
mod definition;
mod expression;
mod operation;
mod resources;
mod statement;
mod ty;

mod common;

pub fn generate(ast: &AST, env: &Environment, ctx: &Context, constr: &Constraints) -> Constrained {
    match &ast.node {
        Node::File { modules, .. } => gen_vec(modules, env, ctx, constr),
        Node::Block { statements } => gen_vec(statements, env, ctx, constr),
        Node::Script { statements } => gen_vec(statements, env, ctx, constr),

        Node::Class { .. } | Node::TypeDef { .. } => gen_class(ast, env, ctx, constr),
        Node::TypeAlias { .. } | Node::Condition { .. } => gen_class(ast, env, ctx, constr),

        Node::VariableDef { .. } | Node::FunDef { .. } => gen_definition(ast, env, ctx, constr),

        Node::Reassign { .. } => generate_call(ast, env, ctx, constr),
        Node::ConstructorCall { .. } => generate_call(ast, env, ctx, constr),
        Node::FunctionCall { .. } => generate_call(ast, env, ctx, constr),
        Node::PropertyCall { .. } => generate_call(ast, env, ctx, constr),

        Node::TypeTup { .. } => gen_ty(ast, env, ctx, constr),
        Node::TypeUnion { .. } | Node::Type { .. } => gen_ty(ast, env, ctx, constr),
        Node::TypeFun { .. } => gen_ty(ast, env, ctx, constr),
        Node::QuestionOp { .. } => gen_ty(ast, env, ctx, constr),

        Node::Id { .. } => gen_expression(ast, env, ctx, constr),
        Node::AnonFun { .. } => gen_expression(ast, env, ctx, constr),
        Node::Question { .. } => gen_expression(ast, env, ctx, constr),

        Node::Raises { .. } => gen_resources(ast, env, ctx, constr),
        Node::With { .. } => gen_resources(ast, env, ctx, constr),

        Node::SetBuilder { .. } | Node::ListBuilder { .. } => gen_collection(ast, env, ctx, constr),
        Node::Set { .. } | Node::List { .. } => gen_collection(ast, env, ctx, constr),
        Node::Tuple { .. } => gen_collection(ast, env, ctx, constr),

        Node::Range { .. } => gen_operation(ast, env, ctx, constr),
        Node::Real { .. } => gen_operation(ast, env, ctx, constr),
        Node::Int { .. } => gen_operation(ast, env, ctx, constr),
        Node::ENum { .. } => gen_operation(ast, env, ctx, constr),
        Node::Str { .. } => gen_operation(ast, env, ctx, constr),
        Node::Bool { .. } => gen_operation(ast, env, ctx, constr),

        Node::Add { .. } | Node::Sub { .. } => gen_operation(ast, env, ctx, constr),
        Node::Mul { .. } | Node::Div { .. } => gen_operation(ast, env, ctx, constr),
        Node::FDiv { .. } => gen_operation(ast, env, ctx, constr),
        Node::Pow { .. } => gen_operation(ast, env, ctx, constr),
        Node::Le { .. } | Node::Ge { .. } => gen_operation(ast, env, ctx, constr),
        Node::Leq { .. } | Node::Geq { .. } => gen_operation(ast, env, ctx, constr),
        Node::Eq { .. } | Node::Neq { .. } => gen_operation(ast, env, ctx, constr),
        Node::Mod { .. } => gen_operation(ast, env, ctx, constr),
        Node::AddU { .. } | Node::SubU { .. } => gen_operation(ast, env, ctx, constr),
        Node::Sqrt { .. } => gen_operation(ast, env, ctx, constr),

        Node::BOneCmpl { .. } => gen_operation(ast, env, ctx, constr),
        Node::BAnd { .. } => gen_operation(ast, env, ctx, constr),
        Node::BOr { .. } | Node::BXOr { .. } => gen_operation(ast, env, ctx, constr),
        Node::BLShift { .. } | Node::BRShift { .. } => gen_operation(ast, env, ctx, constr),

        Node::Is { .. } | Node::IsN { .. } => gen_operation(ast, env, ctx, constr),
        Node::IsA { .. } | Node::IsNA { .. } => gen_operation(ast, env, ctx, constr),
        Node::And { .. } | Node::Or { .. } => gen_operation(ast, env, ctx, constr),
        Node::Not { .. } => gen_operation(ast, env, ctx, constr),

        Node::Handle { .. } => gen_cntrl_flow(ast, env, ctx, constr),
        Node::IfElse { .. } => gen_cntrl_flow(ast, env, ctx, constr),
        Node::Case { .. } => gen_cntrl_flow(ast, env, ctx, constr),
        Node::Match { .. } => gen_cntrl_flow(ast, env, ctx, constr),
        Node::For { .. } => gen_cntrl_flow(ast, env, ctx, constr),
        Node::In { .. } => gen_cntrl_flow(ast, env, ctx, constr),
        Node::Step { .. } => gen_cntrl_flow(ast, env, ctx, constr),
        Node::While { .. } => gen_cntrl_flow(ast, env, ctx, constr),

        Node::Return { .. } => gen_statement(ast, env, ctx, constr),
        Node::Print { .. } => gen_statement(ast, env, ctx, constr),
        Node::Raise { .. } => gen_statement(ast, env, ctx, constr),

        _ => Ok((constr.clone(), env.clone()))
    }
}

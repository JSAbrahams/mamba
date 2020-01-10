use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::cons::Constraints;
use crate::type_checker::constraints::generate::call::gen_call;
use crate::type_checker::constraints::generate::class::gen_class;
use crate::type_checker::constraints::generate::collection::gen_coll;
use crate::type_checker::constraints::generate::control_flow::gen_flow;
use crate::type_checker::constraints::generate::definition::gen_def;
use crate::type_checker::constraints::generate::expression::gen_expr;
use crate::type_checker::constraints::generate::operation::gen_op;
use crate::type_checker::constraints::generate::resources::gen_resources;
use crate::type_checker::constraints::generate::statement::gen_stmt;
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

pub fn generate(ast: &AST, env: &Environment, ctx: &Context, constr: &Constraints) -> Constrained {
    match &ast.node {
        Node::File { modules, .. } => gen_vec(modules, env, ctx, constr),
        Node::Block { statements } => gen_vec(statements, env, ctx, constr),
        Node::Script { statements } => gen_vec(statements, env, ctx, constr),

        Node::Class { .. } | Node::TypeDef { .. } => gen_class(ast, env, ctx, constr),
        Node::TypeAlias { .. } | Node::Condition { .. } => gen_class(ast, env, ctx, constr),

        Node::VariableDef { .. } | Node::FunDef { .. } => gen_def(ast, env, ctx, constr),

        Node::Reassign { .. } => gen_call(ast, env, ctx, constr),
        Node::ConstructorCall { .. } => gen_call(ast, env, ctx, constr),
        Node::FunctionCall { .. } => gen_call(ast, env, ctx, constr),
        Node::PropertyCall { .. } => gen_call(ast, env, ctx, constr),

        Node::TypeTup { .. } => gen_ty(ast, env, ctx, constr),
        Node::TypeUnion { .. } | Node::Type { .. } => gen_ty(ast, env, ctx, constr),
        Node::TypeFun { .. } => gen_ty(ast, env, ctx, constr),
        Node::QuestionOp { .. } => gen_ty(ast, env, ctx, constr),

        Node::Id { .. } | Node::Question { .. } => gen_expr(ast, env, ctx, constr),
        Node::AnonFun { .. } => gen_expr(ast, env, ctx, constr),

        Node::Raises { .. } => gen_resources(ast, env, ctx, constr),
        Node::With { .. } => gen_resources(ast, env, ctx, constr),

        Node::SetBuilder { .. } | Node::ListBuilder { .. } => gen_coll(ast, env, ctx, constr),
        Node::Set { .. } | Node::List { .. } => gen_coll(ast, env, ctx, constr),
        Node::Tuple { .. } => gen_coll(ast, env, ctx, constr),

        Node::Range { .. } => gen_op(ast, env, ctx, constr),
        Node::Real { .. } | Node::Int { .. } => gen_op(ast, env, ctx, constr),
        Node::ENum { .. } => gen_op(ast, env, ctx, constr),
        Node::Str { .. } => gen_op(ast, env, ctx, constr),
        Node::Bool { .. } => gen_op(ast, env, ctx, constr),

        Node::In { .. } => gen_op(ast, env, ctx, constr),
        Node::Add { .. } | Node::Sub { .. } => gen_op(ast, env, ctx, constr),
        Node::Mul { .. } | Node::Div { .. } => gen_op(ast, env, ctx, constr),
        Node::FDiv { .. } => gen_op(ast, env, ctx, constr),
        Node::Pow { .. } => gen_op(ast, env, ctx, constr),
        Node::Le { .. } | Node::Ge { .. } => gen_op(ast, env, ctx, constr),
        Node::Leq { .. } | Node::Geq { .. } => gen_op(ast, env, ctx, constr),
        Node::Eq { .. } | Node::Neq { .. } => gen_op(ast, env, ctx, constr),
        Node::Mod { .. } => gen_op(ast, env, ctx, constr),
        Node::AddU { .. } | Node::SubU { .. } => gen_op(ast, env, ctx, constr),
        Node::Sqrt { .. } => gen_op(ast, env, ctx, constr),

        Node::BOneCmpl { .. } => gen_op(ast, env, ctx, constr),
        Node::BAnd { .. } => gen_op(ast, env, ctx, constr),
        Node::BOr { .. } | Node::BXOr { .. } => gen_op(ast, env, ctx, constr),
        Node::BLShift { .. } | Node::BRShift { .. } => gen_op(ast, env, ctx, constr),

        Node::Is { .. } | Node::IsN { .. } => gen_op(ast, env, ctx, constr),
        Node::IsA { .. } | Node::IsNA { .. } => gen_op(ast, env, ctx, constr),
        Node::And { .. } | Node::Or { .. } => gen_op(ast, env, ctx, constr),
        Node::Not { .. } => gen_op(ast, env, ctx, constr),

        Node::IfElse { .. } => gen_flow(ast, env, ctx, constr),
        Node::Match { .. } | Node::Handle { .. } => gen_flow(ast, env, ctx, constr),
        Node::Case { .. } => gen_flow(ast, env, ctx, constr),
        Node::For { .. } | Node::Step { .. } => gen_flow(ast, env, ctx, constr),
        Node::While { .. } => gen_flow(ast, env, ctx, constr),

        Node::Return { .. } => gen_stmt(ast, env, ctx, constr),
        Node::Print { .. } => gen_stmt(ast, env, ctx, constr),
        Node::Raise { .. } => gen_stmt(ast, env, ctx, constr),

        _ => Ok((constr.clone(), env.clone()))
    }
}

pub fn gen_vec(
    asts: &Vec<AST>,
    env: &Environment,
    ctx: &Context,
    constr: &Constraints
) -> Constrained {
    let mut constr_env = (constr.clone(), env.clone());
    for ast in asts {
        constr_env = generate(ast, &constr_env.1, ctx, &constr_env.0)?;
    }
    Ok(constr_env)
}

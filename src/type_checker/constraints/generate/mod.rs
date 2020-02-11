use crate::parser::ast::Node::*;
use crate::parser::ast::AST;
use crate::type_checker::constraints::constraint::builder::ConstrBuilder;
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

pub fn generate(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder
) -> Constrained {
    match &ast.node {
        File { modules, .. } => gen_vec(modules, env, ctx, constr),
        Block { statements } | Script { statements } => gen_vec(statements, env, ctx, constr),

        Class { .. } | TypeDef { .. } => gen_class(ast, env, ctx, constr),
        TypeAlias { .. } | Condition { .. } => gen_class(ast, env, ctx, constr),

        VariableDef { .. } | FunDef { .. } => gen_def(ast, env, ctx, constr),
        FunArg { .. } => gen_def(ast, env, ctx, constr),

        Reassign { .. } => gen_call(ast, env, ctx, constr),
        ConstructorCall { .. } => gen_call(ast, env, ctx, constr),
        FunctionCall { .. } => gen_call(ast, env, ctx, constr),
        PropertyCall { .. } => gen_call(ast, env, ctx, constr),

        TypeTup { .. } => gen_ty(ast, env, ctx, constr),
        TypeUnion { .. } | Type { .. } => gen_ty(ast, env, ctx, constr),
        TypeFun { .. } => gen_ty(ast, env, ctx, constr),
        QuestionOp { .. } => gen_ty(ast, env, ctx, constr),

        Id { .. } | Question { .. } => gen_expr(ast, env, ctx, constr),
        AnonFun { .. } => gen_expr(ast, env, ctx, constr),
        Pass | Undefined => gen_expr(ast, env, ctx, constr),

        Raises { .. } => gen_resources(ast, env, ctx, constr),
        With { .. } => gen_resources(ast, env, ctx, constr),

        SetBuilder { .. } | ListBuilder { .. } => gen_coll(ast, env, ctx, constr),
        Set { .. } | List { .. } => gen_coll(ast, env, ctx, constr),
        Tuple { .. } => gen_coll(ast, env, ctx, constr),

        Range { .. } => gen_op(ast, env, ctx, constr),
        Real { .. } | Int { .. } => gen_op(ast, env, ctx, constr),
        ENum { .. } => gen_op(ast, env, ctx, constr),
        Str { .. } => gen_op(ast, env, ctx, constr),
        Bool { .. } => gen_op(ast, env, ctx, constr),

        In { .. } => gen_op(ast, env, ctx, constr),
        Add { .. } | Sub { .. } => gen_op(ast, env, ctx, constr),
        Mul { .. } | Div { .. } => gen_op(ast, env, ctx, constr),
        FDiv { .. } => gen_op(ast, env, ctx, constr),
        Pow { .. } => gen_op(ast, env, ctx, constr),
        Le { .. } | Ge { .. } => gen_op(ast, env, ctx, constr),
        Leq { .. } | Geq { .. } => gen_op(ast, env, ctx, constr),
        Eq { .. } | Neq { .. } => gen_op(ast, env, ctx, constr),
        Mod { .. } => gen_op(ast, env, ctx, constr),
        AddU { .. } | SubU { .. } => gen_op(ast, env, ctx, constr),
        Sqrt { .. } => gen_op(ast, env, ctx, constr),

        BOneCmpl { .. } => gen_op(ast, env, ctx, constr),
        BAnd { .. } => gen_op(ast, env, ctx, constr),
        BOr { .. } | BXOr { .. } => gen_op(ast, env, ctx, constr),
        BLShift { .. } | BRShift { .. } => gen_op(ast, env, ctx, constr),

        Is { .. } | IsN { .. } => gen_op(ast, env, ctx, constr),
        IsA { .. } | IsNA { .. } => gen_op(ast, env, ctx, constr),
        And { .. } | Or { .. } => gen_op(ast, env, ctx, constr),
        Not { .. } => gen_op(ast, env, ctx, constr),

        IfElse { .. } => gen_flow(ast, env, ctx, constr),
        Match { .. } | Handle { .. } => gen_flow(ast, env, ctx, constr),
        Case { .. } => gen_flow(ast, env, ctx, constr),
        For { .. } | Step { .. } => gen_flow(ast, env, ctx, constr),
        While { .. } => gen_flow(ast, env, ctx, constr),
        Break | Continue => gen_flow(ast, env, ctx, constr),

        Return { .. } | ReturnEmpty => gen_stmt(ast, env, ctx, constr),
        Print { .. } => gen_stmt(ast, env, ctx, constr),
        Raise { .. } => gen_stmt(ast, env, ctx, constr),

        _ => Ok((constr.clone(), env.clone()))
    }
}

pub fn gen_vec(
    asts: &[AST],
    env: &Environment,
    ctx: &Context,
    constr: &ConstrBuilder
) -> Constrained {
    let mut constr_env = (constr.clone(), env.clone());
    let mut asts = Vec::from(asts);
    let last = asts.pop();

    for ast in asts {
        let mut env = constr_env.1;
        env.last_stmt_in_function = false;
        constr_env = generate(&ast, &env, ctx, &mut constr_env.0)?;
    }

    if let Some(last) = last {
        constr_env = generate(&last, &constr_env.1, ctx, &mut constr_env.0)?;
    }

    Ok(constr_env)
}

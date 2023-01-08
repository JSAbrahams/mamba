use env::Environment;

use crate::check::constrain::constraint::builder::ConstrBuilder;
use crate::check::constrain::constraint::iterator::Constraints;
use crate::check::constrain::generate::call::gen_call;
use crate::check::constrain::generate::class::gen_class;
use crate::check::constrain::generate::collection::gen_coll;
use crate::check::constrain::generate::control_flow::gen_flow;
use crate::check::constrain::generate::definition::gen_def;
use crate::check::constrain::generate::expression::gen_expr;
use crate::check::constrain::generate::operation::gen_op;
use crate::check::constrain::generate::resources::gen_resources;
use crate::check::constrain::generate::statement::gen_stmt;
use crate::check::constrain::generate::ty::gen_ty;
use crate::check::context::Context;
use crate::check::result::TypeErr;
use crate::parse::ast::AST;
use crate::parse::ast::Node::*;

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

pub(super) mod env;

pub type Constrained<T = Environment> = Result<T, Vec<TypeErr>>;

pub fn gen_all(ast: &AST, ctx: &Context) -> Constrained<Vec<Constraints>> {
    let mut builder = ConstrBuilder::new();

    generate(ast, &Environment::default(), ctx, &mut builder)?;
    Ok(builder.all_constr())
}

pub fn generate(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder,
) -> Constrained {
    match &ast.node {
        Block { statements } => gen_vec(statements, env, true, ctx, constr),

        Class { .. } | TypeDef { .. } => gen_class(ast, env, ctx, constr),
        TypeAlias { .. } | Condition { .. } => gen_class(ast, env, ctx, constr),

        VariableDef { .. } | FunDef { .. } | FunArg { .. } => gen_def(ast, env, ctx, constr),

        Reassign { .. } => gen_call(ast, env, ctx, constr),
        FunctionCall { .. } | PropertyCall { .. } => gen_call(ast, env, ctx, constr),
        Index { .. } => gen_call(ast, env, ctx, constr),

        TypeTup { .. } | TypeUnion { .. } | Type { .. } => gen_ty(ast, env, ctx, constr),
        TypeFun { .. } => gen_ty(ast, env, ctx, constr),
        QuestionOp { .. } => gen_ty(ast, env, ctx, constr),

        ExpressionType { .. } | Id { .. } | Question { .. } => gen_expr(ast, env, ctx, constr),
        AnonFun { .. } => gen_expr(ast, env, ctx, constr),
        Pass => gen_expr(ast, env, ctx, constr),

        With { .. } => gen_resources(ast, env, ctx, constr),

        SetBuilder { .. } | ListBuilder { .. } => gen_coll(ast, env, ctx, constr),
        Set { .. } | List { .. } | Tuple { .. } => gen_coll(ast, env, ctx, constr),

        Range { .. } | Slice { .. } => gen_op(ast, env, ctx, constr),
        Real { .. } | Int { .. } | ENum { .. } => gen_op(ast, env, ctx, constr),
        Str { .. } => gen_op(ast, env, ctx, constr),
        Bool { .. } => gen_op(ast, env, ctx, constr),

        In { .. } => gen_op(ast, env, ctx, constr),
        Add { .. } | Sub { .. } | Mul { .. } | Div { .. } => gen_op(ast, env, ctx, constr),
        FDiv { .. } => gen_op(ast, env, ctx, constr),
        Pow { .. } => gen_op(ast, env, ctx, constr),
        Le { .. } | Ge { .. } | Leq { .. } | Geq { .. } => gen_op(ast, env, ctx, constr),
        Eq { .. } | Neq { .. } => gen_op(ast, env, ctx, constr),
        Mod { .. } => gen_op(ast, env, ctx, constr),
        AddU { .. } | SubU { .. } => gen_op(ast, env, ctx, constr),
        Sqrt { .. } => gen_op(ast, env, ctx, constr),
        Undefined => gen_op(ast, env, ctx, constr),

        BOneCmpl { .. } => gen_op(ast, env, ctx, constr),
        BAnd { .. } | BOr { .. } | BXOr { .. } => gen_op(ast, env, ctx, constr),
        BLShift { .. } | BRShift { .. } => gen_op(ast, env, ctx, constr),

        Is { .. } | IsN { .. } | IsA { .. } | IsNA { .. } => gen_op(ast, env, ctx, constr),
        And { .. } | Or { .. } | Not { .. } => gen_op(ast, env, ctx, constr),

        IfElse { .. } => gen_flow(ast, env, ctx, constr),
        Match { .. } | Handle { .. } | Case { .. } => gen_flow(ast, env, ctx, constr),
        For { .. } | While { .. } | Break | Continue => gen_flow(ast, env, ctx, constr),

        Return { .. } | ReturnEmpty => gen_stmt(ast, env, ctx, constr),
        Raise { .. } => gen_stmt(ast, env, ctx, constr),

        Import { .. } | Generic { .. } | Parent { .. } | DocStr { .. } | Underscore => Ok(env.clone()),
    }
}

/// Generate constraint for vector of ASTs.
///
/// Original environment is returned.
/// If [carry_env] is true, then environment is used by each consecutive element.
/// This environment is then also returned.
pub fn gen_vec(
    asts: &[AST],
    env: &Environment,
    carry_env: bool,
    ctx: &Context,
    constr: &mut ConstrBuilder,
) -> Constrained {
    let (mut asts, mut inner_env) = (Vec::from(asts), env.clone());
    let last = asts.pop();

    for ast in asts {
        inner_env = generate(&ast, if carry_env { &inner_env } else { env }, ctx, constr)?;
    }
    if let Some(last) = last {
        let env = if carry_env { inner_env } else { env.clone() };
        inner_env = generate(&last, &env, ctx, constr)?;
    }

    Ok(if carry_env { inner_env } else { env.clone() })
}

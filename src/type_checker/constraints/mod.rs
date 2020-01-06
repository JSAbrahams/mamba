use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::constraint::Constraint;
use crate::type_checker::constraints::expected::Expected;
use crate::type_checker::context::ty;
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::context::Context;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::InferResult;

mod constraint;
mod expected;

pub type Inferred = InferResult<Vec<Constraint>>;

pub fn generate(ast: &AST, env: &Environment, ctx: &Context, constr: &[Constraint]) -> Inferred {
    macro_rules! gen {
        ($ast:expr, $constr:expr, $env:expr) => {{ generate($ast, &$env, ctx, &$constr) }};
        ($ast:expr, $constr:expr, $env:expr, $(($bind:expr, $exp:expr))*) => {{
            let mut constr = $constr.to_vec();
            constr.append(&mut vec![$(Constraint::new($bind, $exp)),*]);
            generate($ast, &$env, ctx, &constr)
        }};
        ($ast:expr, $(($bind:expr, $exp:expr))*) => {{ gen!($ast, constr, env, $(($bind, $exp))*) }};
        ($(($bind:expr, $exp:expr))*) => {{
            let mut constr = constr.to_vec();
            constr.append(&mut vec![$(Constraint::new($bind, $exp)),*]);
            Ok((constr, env.clone()))
         }};
    }

    match &ast.node {
        Node::File { .. } => unimplemented!(),
        Node::Import { .. } => Ok((constr.to_vec(), env.clone())),
        Node::FromImport { .. } => Ok((constr.to_vec(), env.clone())),
        Node::Class { .. } => unimplemented!(),
        Node::Generic { .. } => Ok((constr.to_vec(), env.clone())),
        Node::Parent { .. } => Ok((constr.to_vec(), env.clone())),
        Node::Script { .. } => unimplemented!(),
        Node::Init => Ok((constr.to_vec(), env.clone())),
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
        Node::Id { lit } => {
            let lit_expr_ty = env.lookup(lit, &ast.pos)?;
            gen!((ast, &Expected::Expression { type_name: TypeName::from(&lit_expr_ty) }))
        }
        Node::IdType { .. } => unimplemented!(),
        Node::TypeDef { .. } => unimplemented!(),
        Node::TypeAlias { .. } => unimplemented!(),
        Node::TypeTup { .. } => unimplemented!(),
        Node::TypeUnion { .. } => unimplemented!(),
        Node::Type { .. } => unimplemented!(),
        Node::TypeFun { .. } => unimplemented!(),
        Node::Condition { .. } => unimplemented!(),
        Node::FunArg { .. } => unimplemented!(),
        Node::_Self => Ok((constr.to_vec(), env.clone())),
        Node::AddOp => Ok((constr.to_vec(), env.clone())),
        Node::SubOp => Ok((constr.to_vec(), env.clone())),
        Node::SqrtOp => Ok((constr.to_vec(), env.clone())),
        Node::MulOp => Ok((constr.to_vec(), env.clone())),
        Node::FDivOp => Ok((constr.to_vec(), env.clone())),
        Node::DivOp => Ok((constr.to_vec(), env.clone())),
        Node::PowOp => Ok((constr.to_vec(), env.clone())),
        Node::ModOp => Ok((constr.to_vec(), env.clone())),
        Node::EqOp => Ok((constr.to_vec(), env.clone())),
        Node::LeOp => Ok((constr.to_vec(), env.clone())),
        Node::GeOp => Ok((constr.to_vec(), env.clone())),
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
        Node::DocStr { .. } => unimplemented!(),
        Node::Str { .. } => unimplemented!(),
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
        Node::IfElse { .. } => unimplemented!(),
        Node::Match { .. } => unimplemented!(),
        Node::Case { .. } => unimplemented!(),
        Node::For { .. } => unimplemented!(),
        Node::In { .. } => unimplemented!(),
        Node::Step { .. } => unimplemented!(),
        Node::While { cond, body } => {
            let bool_name = TypeName::from(ty::concrete::BOOL_PRIMITIVE);
            let (constr, env) = gen!(cond, (cond, &Expected::Expression { type_name: bool_name }))?;
            gen!(body, constr, env)
        }

        Node::Break => Ok((constr.to_vec(), env.clone())),
        Node::Continue => Ok((constr.to_vec(), env.clone())),

        Node::Return { expr } => gen!(expr, (expr, &Expected::AnyExpression {})),
        Node::ReturnEmpty => Ok((constr.to_vec(), env.clone())),
        Node::Underscore => Ok((constr.to_vec(), env.clone())),
        Node::Undefined => Ok((constr.to_vec(), env.clone())),
        Node::Pass => Ok((constr.to_vec(), env.clone())),

        Node::Question { .. } => unimplemented!(),
        Node::QuestionOp { expr } => {
            let (constr, env) = gen!(expr, (expr, &Expected::AnyExpression {}))?;
            // TODO bind to inner expression
            gen!(
                ast,
                constr,
                env,
                (ast, &Expected::NullableExpression {
                    expected: Box::from(Expected::AnyExpression {})
                })
            )
        }

        Node::Print { expr } => gen!(expr, (expr, &Expected::AnyExpression {})),

        Node::Comment { .. } => Ok((constr.to_vec(), env.clone()))
    }
}

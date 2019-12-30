use crate::parser::ast::{Node, AST};
use crate::type_checker::context::Context;
use crate::type_checker::type_result::TypeResult;

pub mod constructor;
pub mod forward;
pub mod retry;

pub trait Modification {
    fn modify(&self, ast: &AST, ctx: &Context) -> TypeResult<AST>;

    fn recursion(&self, ast: &AST, ctx: &Context) -> TypeResult<AST> {
        match &ast.node {
            Node::File { pure, comments, imports, modules } => {
                let comments =
                    comments.iter().map(|com| self.modify(com, ctx)).collect::<Result<_, _>>()?;
                let imports =
                    imports.iter().map(|imp| self.modify(imp, ctx)).collect::<Result<_, _>>()?;
                let modules = modules
                    .iter()
                    .map(|module| self.modify(module, ctx))
                    .collect::<Result<_, _>>()?;
                Ok(AST {
                    node: Node::File { comments, imports, modules, pure: pure.clone() },
                    ..ast.clone()
                })
            }
            Node::Import { import, _as } => {
                let import =
                    import.iter().map(|imp| self.modify(imp, ctx)).collect::<Result<_, _>>()?;
                let _as = _as.iter().map(|imp| self.modify(imp, ctx)).collect::<Result<_, _>>()?;
                Ok(AST { node: Node::Import { import, _as }, ..ast.clone() })
            }
            Node::FromImport { id, import } => {
                let id = Box::from(self.modify(id, ctx)?);
                let import = Box::from(self.modify(import, ctx)?);
                Ok(AST { node: Node::FromImport { id, import }, ..ast.clone() })
            }
            Node::Class { _type, args, parents, body } => {
                let _type = Box::from(self.modify(_type, ctx)?);
                let args =
                    args.iter().map(|arg| self.modify(arg, ctx)).collect::<Result<_, _>>()?;
                let parents = parents
                    .iter()
                    .map(|parent| self.modify(parent, ctx))
                    .collect::<Result<_, _>>()?;
                let body = if let Some(body) = body {
                    Some(Box::from(self.modify(body, ctx)?))
                } else {
                    None
                };
                Ok(AST { node: Node::Class { _type, args, parents, body }, ..ast.clone() })
            }
            Node::Generic { id, isa } => {
                let id = Box::from(self.modify(id, ctx)?);
                let isa = if let Some(isa) = isa {
                    Some(Box::from(self.modify(isa, ctx)?))
                } else {
                    None
                };
                Ok(AST { node: Node::Generic { id, isa }, ..ast.clone() })
            }
            Node::Parent { id, generics, args } => {
                let id = Box::from(self.modify(id, ctx)?);
                let generics = generics
                    .iter()
                    .map(|generic| self.modify(generic, ctx))
                    .collect::<Result<_, _>>()?;
                let args =
                    args.iter().map(|arg| self.modify(arg, ctx)).collect::<Result<_, _>>()?;
                Ok(AST { node: Node::Parent { id, generics, args }, ..ast.clone() })
            }
            Node::Script { statements } => {
                let statements = statements
                    .iter()
                    .map(|stmt| self.modify(stmt, ctx))
                    .collect::<Result<_, _>>()?;
                Ok(AST { node: Node::Script { statements }, ..ast.clone() })
            }
            Node::Init => Ok(ast.clone()),
            Node::Reassign { left, right } => {
                let left = Box::from(self.modify(left, ctx)?);
                let right = Box::from(self.modify(right, ctx)?);
                Ok(AST { node: Node::Reassign { left, right }, ..ast.clone() })
            }
            Node::VariableDef { private, id_maybe_type, expression, forward } => {
                let id_maybe_type = Box::from(self.modify(id_maybe_type, ctx)?);
                let expression = if let Some(expression) = expression {
                    Some(Box::from(self.modify(expression, ctx)?))
                } else {
                    None
                };
                let forward = forward
                    .iter()
                    .map(|forward| self.modify(forward, ctx))
                    .collect::<Result<_, _>>()?;
                Ok(AST {
                    node: Node::VariableDef {
                        private: private.clone(),
                        id_maybe_type,
                        expression,
                        forward
                    },
                    ..ast.clone()
                })
            }
            Node::FunDef { pure, private, id, fun_args, ret_ty, raises, body } => {
                let id = Box::from(self.modify(id, ctx)?);
                let fun_args =
                    fun_args.iter().map(|arg| self.modify(arg, ctx)).collect::<Result<_, _>>()?;
                let ret_ty = if let Some(ret_ty) = ret_ty {
                    Some(Box::from(self.modify(ret_ty, ctx)?))
                } else {
                    None
                };
                let raises =
                    raises.iter().map(|raise| self.modify(raise, ctx)).collect::<Result<_, _>>()?;
                let body = if let Some(body) = body {
                    Some(Box::from(self.modify(body, ctx)?))
                } else {
                    None
                };
                Ok(AST {
                    node: Node::FunDef {
                        pure: pure.clone(),
                        private: private.clone(),
                        id,
                        fun_args,
                        ret_ty,
                        raises,
                        body
                    },
                    ..ast.clone()
                })
            }
            Node::AnonFun { args, body } => {
                let args =
                    args.iter().map(|arg| self.modify(arg, ctx)).collect::<Result<_, _>>()?;
                let body = Box::from(self.modify(body, ctx)?);
                Ok(AST { node: Node::AnonFun { args, body }, ..ast.clone() })
            }
            Node::Raises { expr_or_stmt, errors } => {
                let expr_or_stmt = Box::from(self.modify(expr_or_stmt, ctx)?);
                let errors =
                    errors.iter().map(|error| self.modify(error, ctx)).collect::<Result<_, _>>()?;
                Ok(AST { node: Node::Raises { expr_or_stmt, errors }, ..ast.clone() })
            }
            Node::Raise { error } => {
                let error = Box::from(self.modify(error, ctx)?);
                Ok(AST { node: Node::Raise { error }, ..ast.clone() })
            }
            Node::Handle { expr_or_stmt, cases } => {
                let expr_or_stmt = Box::from(self.modify(expr_or_stmt, ctx)?);
                let cases =
                    cases.iter().map(|case| self.modify(case, ctx)).collect::<Result<_, _>>()?;
                Ok(AST { node: Node::Handle { expr_or_stmt, cases }, ..ast.clone() })
            }
            Node::Retry => Ok(ast.clone()),
            Node::With { resource, _as, expr } => {
                let resource = Box::from(self.modify(resource, ctx)?);
                let _as = if let Some(_as) = _as {
                    Some(Box::from(self.modify(_as, ctx)?))
                } else {
                    None
                };
                let expr = Box::from(self.modify(expr, ctx)?);
                Ok(AST { node: Node::With { resource, _as, expr }, ..ast.clone() })
            }
            Node::ConstructorCall { name, args } => {
                let name = Box::from(self.modify(name, ctx)?);
                let args =
                    args.iter().map(|arg| self.modify(arg, ctx)).collect::<Result<_, _>>()?;
                Ok(AST { node: Node::ConstructorCall { name, args }, ..ast.clone() })
            }
            Node::FunctionCall { name, args } => {
                let name = Box::from(self.modify(name, ctx)?);
                let args =
                    args.iter().map(|arg| self.modify(arg, ctx)).collect::<Result<_, _>>()?;
                Ok(AST { node: Node::FunctionCall { name, args }, ..ast.clone() })
            }
            Node::PropertyCall { instance, property } => {
                let instance = Box::from(self.modify(instance, ctx)?);
                let property = Box::from(self.modify(property, ctx)?);
                Ok(AST { node: Node::PropertyCall { instance, property }, ..ast.clone() })
            }
            Node::Id { .. } => Ok(ast.clone()),
            Node::IdType { id, mutable, _type } => {
                let id = Box::from(self.modify(id, ctx)?);
                let _type = if let Some(_type) = _type {
                    Some(Box::from(self.modify(_type, ctx)?))
                } else {
                    None
                };
                Ok(AST {
                    node: Node::IdType { mutable: mutable.clone(), id, _type },
                    ..ast.clone()
                })
            }
            Node::TypeDef { _type, isa, body } => {
                let _type = Box::from(self.modify(_type, ctx)?);
                let isa = if let Some(isa) = isa {
                    Some(Box::from(self.modify(isa, ctx)?))
                } else {
                    None
                };
                let body = if let Some(body) = body {
                    Some(Box::from(self.modify(body, ctx)?))
                } else {
                    None
                };
                Ok(AST { node: Node::TypeDef { _type, isa, body }, ..ast.clone() })
            }
            Node::TypeAlias { _type, isa, conditions } => {
                let _type = Box::from(self.modify(_type, ctx)?);
                let isa = Box::from(self.modify(isa, ctx)?);
                let conditions = conditions
                    .iter()
                    .map(|cond| self.modify(cond, ctx))
                    .collect::<Result<_, _>>()?;
                Ok(AST { node: Node::TypeAlias { _type, isa, conditions }, ..ast.clone() })
            }
            Node::TypeTup { types } => {
                let types =
                    types.iter().map(|ty| self.modify(ty, ctx)).collect::<Result<_, _>>()?;
                Ok(AST { node: Node::TypeTup { types }, ..ast.clone() })
            }
            Node::TypeUnion { types } => {
                let types =
                    types.iter().map(|ty| self.modify(ty, ctx)).collect::<Result<_, _>>()?;
                Ok(AST { node: Node::TypeUnion { types }, ..ast.clone() })
            }
            Node::Type { id, generics } => {
                let id = Box::from(self.modify(id, ctx)?);
                let generics = generics
                    .iter()
                    .map(|generic| self.modify(generic, ctx))
                    .collect::<Result<_, _>>()?;
                Ok(AST { node: Node::Type { id, generics }, ..ast.clone() })
            }
            Node::TypeFun { args, ret_ty } => {
                let args =
                    args.iter().map(|arg| self.modify(arg, ctx)).collect::<Result<_, _>>()?;
                let ret_ty = Box::from(self.modify(ret_ty, ctx)?);
                Ok(AST { node: Node::TypeFun { args, ret_ty }, ..ast.clone() })
            }
            Node::Condition { cond, _else } => {
                let cond = Box::from(self.modify(cond, ctx)?);
                let _else = if let Some(_else) = _else {
                    Some(Box::from(self.modify(_else, ctx)?))
                } else {
                    None
                };
                Ok(AST { node: Node::Condition { cond, _else }, ..ast.clone() })
            }
            Node::FunArg { vararg, id_maybe_type, default } => {
                let id_maybe_type = Box::from(self.modify(id_maybe_type, ctx)?);
                let default = if let Some(default) = default {
                    Some(Box::from(self.modify(default, ctx)?))
                } else {
                    None
                };
                Ok(AST {
                    node: Node::FunArg { vararg: vararg.clone(), id_maybe_type, default },
                    ..ast.clone()
                })
            }
            Node::_Self => Ok(ast.clone()),
            Node::AddOp => Ok(ast.clone()),
            Node::SubOp => Ok(ast.clone()),
            Node::SqrtOp => Ok(ast.clone()),
            Node::MulOp => Ok(ast.clone()),
            Node::FDivOp => Ok(ast.clone()),
            Node::DivOp => Ok(ast.clone()),
            Node::PowOp => Ok(ast.clone()),
            Node::ModOp => Ok(ast.clone()),
            Node::EqOp => Ok(ast.clone()),
            Node::LeOp => Ok(ast.clone()),
            Node::GeOp => Ok(ast.clone()),
            Node::SetBuilder { item, conditions } => {
                let item = Box::from(self.modify(item, ctx)?);
                let conditions = conditions
                    .iter()
                    .map(|cond| self.modify(cond, ctx))
                    .collect::<Result<_, _>>()?;
                Ok(AST { node: Node::SetBuilder { item, conditions }, ..ast.clone() })
            }
            Node::ListBuilder { item, conditions } => {
                let item = Box::from(self.modify(item, ctx)?);
                let conditions = conditions
                    .iter()
                    .map(|cond| self.modify(cond, ctx))
                    .collect::<Result<_, _>>()?;
                Ok(AST { node: Node::SetBuilder { item, conditions }, ..ast.clone() })
            }
            Node::Set { elements } => {
                let elements =
                    elements.iter().map(|el| self.modify(el, ctx)).collect::<Result<_, _>>()?;
                Ok(AST { node: Node::Set { elements }, ..ast.clone() })
            }
            Node::List { elements } => {
                let elements =
                    elements.iter().map(|el| self.modify(el, ctx)).collect::<Result<_, _>>()?;
                Ok(AST { node: Node::List { elements }, ..ast.clone() })
            }
            Node::Tuple { elements } => {
                let elements =
                    elements.iter().map(|el| self.modify(el, ctx)).collect::<Result<_, _>>()?;
                Ok(AST { node: Node::Tuple { elements }, ..ast.clone() })
            }
            Node::Range { from, to, inclusive, step } => {
                let from = Box::from(self.modify(from, ctx)?);
                let to = Box::from(self.modify(to, ctx)?);
                let step = if let Some(step) = step {
                    Some(Box::from(self.modify(step, ctx)?))
                } else {
                    None
                };
                Ok(AST {
                    node: Node::Range { from, to, inclusive: inclusive.clone(), step },
                    ..ast.clone()
                })
            }
            Node::Block { statements } => {
                let statements = statements
                    .iter()
                    .map(|stmt| self.modify(stmt, ctx))
                    .collect::<Result<_, _>>()?;
                Ok(AST { node: Node::Block { statements }, ..ast.clone() })
            }
            Node::Real { .. } => Ok(ast.clone()),
            Node::Int { .. } => Ok(ast.clone()),
            Node::ENum { .. } => Ok(ast.clone()),
            Node::Str { lit, expressions } => {
                let expressions = expressions
                    .iter()
                    .map(|expr| self.modify(expr, ctx))
                    .collect::<Result<_, _>>()?;
                Ok(AST { node: Node::Str { lit: lit.clone(), expressions }, ..ast.clone() })
            }
            Node::Bool { .. } => Ok(ast.clone()),
            Node::Add { left, right } => {
                let left = Box::from(self.modify(left, ctx)?);
                let right = Box::from(self.modify(right, ctx)?);
                Ok(AST { node: Node::Add { left, right }, ..ast.clone() })
            }
            Node::Sub { left, right } => {
                let left = Box::from(self.modify(left, ctx)?);
                let right = Box::from(self.modify(right, ctx)?);
                Ok(AST { node: Node::Sub { left, right }, ..ast.clone() })
            }
            Node::Mul { left, right } => {
                let left = Box::from(self.modify(left, ctx)?);
                let right = Box::from(self.modify(right, ctx)?);
                Ok(AST { node: Node::Mul { left, right }, ..ast.clone() })
            }
            Node::Div { left, right } => {
                let left = Box::from(self.modify(left, ctx)?);
                let right = Box::from(self.modify(right, ctx)?);
                Ok(AST { node: Node::Div { left, right }, ..ast.clone() })
            }
            Node::FDiv { left, right } => {
                let left = Box::from(self.modify(left, ctx)?);
                let right = Box::from(self.modify(right, ctx)?);
                Ok(AST { node: Node::FDiv { left, right }, ..ast.clone() })
            }
            Node::Mod { left, right } => {
                let left = Box::from(self.modify(left, ctx)?);
                let right = Box::from(self.modify(right, ctx)?);
                Ok(AST { node: Node::Mod { left, right }, ..ast.clone() })
            }
            Node::Pow { left, right } => {
                let left = Box::from(self.modify(left, ctx)?);
                let right = Box::from(self.modify(right, ctx)?);
                Ok(AST { node: Node::Pow { left, right }, ..ast.clone() })
            }
            Node::BAnd { left, right } => {
                let left = Box::from(self.modify(left, ctx)?);
                let right = Box::from(self.modify(right, ctx)?);
                Ok(AST { node: Node::BAnd { left, right }, ..ast.clone() })
            }
            Node::BOr { left, right } => {
                let left = Box::from(self.modify(left, ctx)?);
                let right = Box::from(self.modify(right, ctx)?);
                Ok(AST { node: Node::BOr { left, right }, ..ast.clone() })
            }
            Node::BXOr { left, right } => {
                let left = Box::from(self.modify(left, ctx)?);
                let right = Box::from(self.modify(right, ctx)?);
                Ok(AST { node: Node::BXOr { left, right }, ..ast.clone() })
            }
            Node::BLShift { left, right } => {
                let left = Box::from(self.modify(left, ctx)?);
                let right = Box::from(self.modify(right, ctx)?);
                Ok(AST { node: Node::BLShift { left, right }, ..ast.clone() })
            }
            Node::BRShift { left, right } => {
                let left = Box::from(self.modify(left, ctx)?);
                let right = Box::from(self.modify(right, ctx)?);
                Ok(AST { node: Node::BRShift { left, right }, ..ast.clone() })
            }
            Node::Le { left, right } => {
                let left = Box::from(self.modify(left, ctx)?);
                let right = Box::from(self.modify(right, ctx)?);
                Ok(AST { node: Node::Le { left, right }, ..ast.clone() })
            }
            Node::Ge { left, right } => {
                let left = Box::from(self.modify(left, ctx)?);
                let right = Box::from(self.modify(right, ctx)?);
                Ok(AST { node: Node::Ge { left, right }, ..ast.clone() })
            }
            Node::Leq { left, right } => {
                let left = Box::from(self.modify(left, ctx)?);
                let right = Box::from(self.modify(right, ctx)?);
                Ok(AST { node: Node::Leq { left, right }, ..ast.clone() })
            }
            Node::Geq { left, right } => {
                let left = Box::from(self.modify(left, ctx)?);
                let right = Box::from(self.modify(right, ctx)?);
                Ok(AST { node: Node::Geq { left, right }, ..ast.clone() })
            }
            Node::Is { left, right } => {
                let left = Box::from(self.modify(left, ctx)?);
                let right = Box::from(self.modify(right, ctx)?);
                Ok(AST { node: Node::Is { left, right }, ..ast.clone() })
            }
            Node::IsN { left, right } => {
                let left = Box::from(self.modify(left, ctx)?);
                let right = Box::from(self.modify(right, ctx)?);
                Ok(AST { node: Node::IsN { left, right }, ..ast.clone() })
            }
            Node::Eq { left, right } => {
                let left = Box::from(self.modify(left, ctx)?);
                let right = Box::from(self.modify(right, ctx)?);
                Ok(AST { node: Node::Eq { left, right }, ..ast.clone() })
            }
            Node::Neq { left, right } => {
                let left = Box::from(self.modify(left, ctx)?);
                let right = Box::from(self.modify(right, ctx)?);
                Ok(AST { node: Node::Neq { left, right }, ..ast.clone() })
            }
            Node::IsA { left, right } => {
                let left = Box::from(self.modify(left, ctx)?);
                let right = Box::from(self.modify(right, ctx)?);
                Ok(AST { node: Node::IsA { left, right }, ..ast.clone() })
            }
            Node::IsNA { left, right } => {
                let left = Box::from(self.modify(left, ctx)?);
                let right = Box::from(self.modify(right, ctx)?);
                Ok(AST { node: Node::IsNA { left, right }, ..ast.clone() })
            }
            Node::And { left, right } => {
                let left = Box::from(self.modify(left, ctx)?);
                let right = Box::from(self.modify(right, ctx)?);
                Ok(AST { node: Node::And { left, right }, ..ast.clone() })
            }
            Node::Or { left, right } => {
                let left = Box::from(self.modify(left, ctx)?);
                let right = Box::from(self.modify(right, ctx)?);
                Ok(AST { node: Node::Or { left, right }, ..ast.clone() })
            }
            Node::BOneCmpl { expr } => {
                let expr = Box::from(self.modify(expr, ctx)?);
                Ok(AST { node: Node::BOneCmpl { expr }, ..ast.clone() })
            }
            Node::Sqrt { expr } => {
                let expr = Box::from(self.modify(expr, ctx)?);
                Ok(AST { node: Node::Sqrt { expr }, ..ast.clone() })
            }
            Node::SubU { expr } => {
                let expr = Box::from(self.modify(expr, ctx)?);
                Ok(AST { node: Node::SubU { expr }, ..ast.clone() })
            }
            Node::AddU { expr } => {
                let expr = Box::from(self.modify(expr, ctx)?);
                Ok(AST { node: Node::AddU { expr }, ..ast.clone() })
            }
            Node::Not { expr } => {
                let expr = Box::from(self.modify(expr, ctx)?);
                Ok(AST { node: Node::Not { expr }, ..ast.clone() })
            }
            Node::IfElse { cond, then, _else } => {
                let cond = Box::from(self.modify(cond, ctx)?);
                let then = Box::from(self.modify(then, ctx)?);
                let _else = if let Some(_else) = _else {
                    Some(Box::from(self.modify(_else, ctx)?))
                } else {
                    None
                };
                Ok(AST { node: Node::IfElse { cond, then, _else }, ..ast.clone() })
            }
            Node::Match { cond, cases } => {
                let cond = Box::from(self.modify(cond, ctx)?);
                let cases =
                    cases.iter().map(|case| self.modify(case, ctx)).collect::<Result<_, _>>()?;
                Ok(AST { node: Node::Match { cond, cases }, ..ast.clone() })
            }
            Node::Case { cond, body } => {
                let cond = Box::from(self.modify(cond, ctx)?);
                let body = Box::from(self.modify(body, ctx)?);
                Ok(AST { node: Node::Case { cond, body }, ..ast.clone() })
            }
            Node::For { expr, col, body } => {
                let expr = Box::from(self.modify(expr, ctx)?);
                let col = Box::from(self.modify(col, ctx)?);
                let body = Box::from(self.modify(body, ctx)?);
                Ok(AST { node: Node::For { expr, col, body }, ..ast.clone() })
            }
            Node::In { left, right } => {
                let left = Box::from(self.modify(left, ctx)?);
                let right = Box::from(self.modify(right, ctx)?);
                Ok(AST { node: Node::In { left, right }, ..ast.clone() })
            }
            Node::Step { amount } => {
                let amount = Box::from(self.modify(amount, ctx)?);
                Ok(AST { node: Node::Step { amount }, ..ast.clone() })
            }
            Node::While { cond, body } => {
                let cond = Box::from(self.modify(cond, ctx)?);
                let body = Box::from(self.modify(body, ctx)?);
                Ok(AST { node: Node::While { cond, body }, ..ast.clone() })
            }
            Node::Break => Ok(ast.clone()),
            Node::Continue => Ok(ast.clone()),
            Node::Return { expr } => {
                let expr = Box::from(self.modify(expr, ctx)?);
                Ok(AST { node: Node::Return { expr }, ..ast.clone() })
            }
            Node::ReturnEmpty => Ok(ast.clone()),
            Node::Underscore => Ok(ast.clone()),
            Node::Undefined => Ok(ast.clone()),
            Node::Pass => Ok(ast.clone()),
            Node::Question { left, right } => {
                let left = Box::from(self.modify(left, ctx)?);
                let right = Box::from(self.modify(right, ctx)?);
                Ok(AST { node: Node::Question { left, right }, ..ast.clone() })
            }
            Node::QuestionOp { expr } => {
                let expr = Box::from(self.modify(expr, ctx)?);
                Ok(AST { node: Node::QuestionOp { expr }, ..ast.clone() })
            }
            Node::Print { expr } => {
                let expr = Box::from(self.modify(expr, ctx)?);
                Ok(AST { node: Node::Print { expr }, ..ast.clone() })
            }
            Node::Comment { .. } => Ok(ast.clone())
        }
    }
}

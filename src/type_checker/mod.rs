use crate::parser::ast::{ASTNode, ASTNodePos};
use crate::type_checker::node_type::Type;
use crate::type_checker::type_result::TypeResult;
use std::clone::Clone;

pub mod node_type;
pub mod type_result;

/// Checks whether a given [ASTNodePos](crate::parser::ast::ASTNodePos) is well
/// typed according to the specification of the language.
///
/// Should never panic.
///
/// # Examples
///
/// // examples here
///
/// # Failures
///
/// Any ill-typed [ASTNodePos](crate::parser::ast::ASTNodePos) results in a
/// failure.
///
/// // failure examples here
pub fn check(input: &[ASTNodePos]) -> Result<Vec<ASTNodePos>, Vec<String>> {
    let (_, errors): (Vec<_>, Vec<_>) =
        input.iter().map(|node_pos| get_type(node_pos.clone())).partition(Result::is_ok);

    if errors.is_empty() {
        Ok(input.to_vec())
    } else {
        Err(errors.into_iter().map(Result::unwrap_err).collect())
    }
}

fn get_type_expect(node_pos: &ASTNodePos, expected: &Type) -> Result<Type, String> {
    let node_type = get_type(node_pos.clone())?;
    if node_type == *expected {
        Ok(node_type)
    } else {
        Err(format!("Expected {:?}, but was {:?} ({:?})", expected, node_type, node_pos))
    }
}

fn get_type(node_pos: ASTNodePos) -> TypeResult<Type> {
    match node_pos.node {
        ASTNode::File { modules, type_defs, .. } => {
            for module in modules {
                get_type(module)?;
            }
            for type_def in type_defs {
                get_type(type_def)?;
            }
            Ok(Type::NA)
        }
        ASTNode::Import { .. } => Ok(Type::NA),
        ASTNode::FromImport { .. } => Ok(Type::NA),
        ASTNode::Class { body, .. } => {
            get_type(*body)?;
            Ok(Type::NA)
        }
        ASTNode::Generic { .. } => Ok(Type::NA),
        ASTNode::Parent { .. } => Ok(Type::NA),
        ASTNode::Script { statements } => {
            let mut last_type = Type::Empty;
            for statement in statements {
                last_type = get_type(statement)?;
            }
            Ok(last_type)
        }
        ASTNode::Init => Ok(Type::NA),

        ASTNode::Reassign { left, right } => {
            let left_type = get_type(*left)?;
            get_type_expect(&*right, &left_type)?;
            Ok(Type::NA)
        }
        ASTNode::Def { definition, .. } => get_type(*definition),
        ASTNode::VariableDef { id_maybe_type, expression, .. } => {
            let id_type = match id_maybe_type.node {
                ASTNode::IdType { _type: Some(_type), .. } => get_type(*_type)?,
                ASTNode::IdType { .. } => Type::Any,
                _ => return Err(String::from("expected id type"))
            };

            if expression.is_some() {
                let expression = expression.unwrap_or_else(|| unreachable!());
                get_type_expect(&*expression, &id_type)
            } else {
                Ok(id_type)
            }
        }

        ASTNode::FunDef { fun_args, ret_ty, body, .. } => {
            // TODO do something with raises
            for fun_arg in fun_args {
                match fun_arg.node {
                    ASTNode::FunArg { id_maybe_type, default, .. } => {
                        // TODO do something with vararg
                        let id_type = match id_maybe_type.node {
                            ASTNode::IdType { _type: Some(_type), .. } => get_type(*_type)?,
                            ASTNode::IdType { .. } => Type::Any,
                            _ => return Err(String::from("Expected id type"))
                        };

                        if default.is_some() {
                            get_type_expect(&*default.unwrap_or_else(|| unreachable!()), &id_type)?;
                        }
                    }
                    _ => return Err(String::from("Expected fun arg"))
                }
            }

            let body = if body.is_some() {
                body.unwrap()
            } else {
                return Ok(Type::NA);
            };

            // TODO add fun args to context when checking body
            let body_type = get_type(*body)?;
            if ret_ty.is_some() {
                let ret_ty = ret_ty.unwrap_or_else(|| unreachable!());
                let function_return_type = get_type(*ret_ty)?;
                if body_type == function_return_type {
                    Ok(function_return_type)
                } else {
                    Err(String::from("function return type did not match body type"))
                }
            } else {
                Ok(body_type)
            }
        }

        ASTNode::AnonFun { args, body } => {
            let arg_types: TypeResult<Vec<Type>> =
                args.iter().map(|arg| get_type(arg.clone())).collect();
            let body_type = get_type(*body)?;
            Ok(Type::AnonFun { args: arg_types?, out: Box::new(body_type) })
        }

        ASTNode::Raises { .. } => Ok(Type::NA),
        ASTNode::Raise { .. } => Ok(Type::Any),
        ASTNode::Handle { .. } => Ok(Type::NA),
        ASTNode::Retry => Ok(Type::NA),
        ASTNode::With { .. } => Ok(Type::NA),

        ASTNode::FunctionCall { .. } => Ok(Type::Any),
        ASTNode::PropertyCall { .. } => Ok(Type::Any),
        ASTNode::Id { .. } => Ok(Type::Any),

        // TODO implement
        ASTNode::IdType { .. } => Ok(Type::Any),
        ASTNode::Condition { .. } => Ok(Type::NA),
        ASTNode::FunArg { .. } => Err(String::from("fun arg cannot be top level")),
        ASTNode::TypeDef { .. } => Type::try_from_node(node_pos.node),
        ASTNode::TypeAlias { .. } => Type::try_from_node(node_pos.node),
        ASTNode::TypeTup { .. } => Type::try_from_node(node_pos.node),
        ASTNode::Type { .. } => Type::try_from_node(node_pos.node),
        ASTNode::TypeFun { .. } => Type::try_from_node(node_pos.node),

        ASTNode::_Self => Ok(Type::NA),
        ASTNode::AddOp => Ok(Type::NA),
        ASTNode::SubOp => Ok(Type::NA),
        ASTNode::SqrtOp => Ok(Type::NA),
        ASTNode::MulOp => Ok(Type::NA),
        ASTNode::FDivOp => Ok(Type::NA),
        ASTNode::DivOp => Ok(Type::NA),
        ASTNode::PowOp => Ok(Type::NA),
        ASTNode::ModOp => Ok(Type::NA),
        ASTNode::EqOp => Ok(Type::NA),
        ASTNode::LeOp => Ok(Type::NA),
        ASTNode::GeOp => Ok(Type::NA),

        ASTNode::Set { elements } => {
            let mut ty = Type::Any;
            for element in elements {
                ty = get_type_expect(&element, &ty)?;
            }
            Ok(Type::Set { ty: Box::from(ty) })
        }
        ASTNode::SetBuilder { .. } => unimplemented!(),
        ASTNode::List { elements } => {
            let mut ty = Type::Any;
            for element in elements {
                ty = get_type_expect(&element, &ty)?;
            }
            Ok(Type::List { ty: Box::from(ty) })
        }
        ASTNode::ListBuilder { .. } => unimplemented!(),
        ASTNode::Tuple { elements } => {
            let types: TypeResult<Vec<Type>> =
                elements.iter().map(|node_pos| get_type(node_pos.clone())).collect();
            Ok(Type::Tuple { tys: types? })
        }

        ASTNode::Range { from, to, .. } => {
            // TODO do something with step
            let from_type = get_type(*from)?;
            get_type_expect(&*to, &from_type)?;
            Ok(Type::Range { ty: Box::from(from_type) })
        }

        ASTNode::Block { statements } => {
            let mut last_type = Type::Empty;
            for statement in statements {
                last_type = get_type(statement)?
            }
            Ok(last_type)
        }

        ASTNode::Real { .. } => Ok(Type::Float),
        ASTNode::Int { .. } => Ok(Type::Int),
        ASTNode::ENum { .. } => unimplemented!(),
        ASTNode::Str { .. } => Ok(Type::String),
        ASTNode::Bool { .. } => Ok(Type::Bool),

        ASTNode::Add { left, right } => {
            // TODO check if types overwrite add function
            get_type(*left)?;
            get_type(*right)
        }
        ASTNode::AddU { expr } => get_type(*expr),
        ASTNode::Sub { left, right } => {
            // TODO check if types overwrite sub function
            get_type(*left)?;
            get_type(*right)
        }
        ASTNode::SubU { expr } => get_type(*expr),
        ASTNode::Mul { left, right } => {
            // TODO check if types overwrite mul function
            get_type(*left)?;
            get_type(*right)
        }
        ASTNode::Div { left, right } => {
            // TODO check if types overwrite div function
            get_type(*left)?;
            get_type(*right)
        }
        ASTNode::FDiv { left, right } => {
            // TODO check if types overwrite fdiv function
            get_type(*left)?;
            get_type(*right)?;
            Ok(Type::Int)
        }
        ASTNode::Mod { left, right } => {
            // TODO check if types overwrite mod function
            get_type(*left)?;
            get_type(*right)?;
            Ok(Type::Int)
        }
        ASTNode::Pow { left, right } => {
            // TODO check if types overwrite pow function
            get_type(*left)?;
            get_type(*right)
        }
        ASTNode::Sqrt { expr } => get_type(*expr),

        ASTNode::BAnd { .. } => Ok(Type::Int),
        ASTNode::BOr { .. } => Ok(Type::Int),
        ASTNode::BXOr { .. } => Ok(Type::Int),
        ASTNode::BOneCmpl { .. } => Ok(Type::Int),
        ASTNode::BLShift { .. } => Ok(Type::Int),
        ASTNode::BRShift { .. } => Ok(Type::Int),

        ASTNode::Le { left, right } => {
            // TODO check if types overwrite le function
            get_type(*left)?;
            get_type(*right)?;
            Ok(Type::Bool)
        }
        ASTNode::Ge { left, right } => {
            // TODO check if types overwrite ge function
            get_type(*left)?;
            get_type(*right)?;
            Ok(Type::Bool)
        }
        ASTNode::Leq { left, right } => {
            // TODO check if types overwrite leq function
            get_type(*left)?;
            get_type(*right)?;
            Ok(Type::Bool)
        }
        ASTNode::Geq { left, right } => {
            // TODO check if types overwrite geq function
            get_type(*left)?;
            get_type(*right)?;
            Ok(Type::Bool)
        }
        ASTNode::Is { left, right } => {
            get_type(*left)?;
            get_type(*right)?;
            Ok(Type::Bool)
        }
        ASTNode::IsN { left, right } => {
            get_type(*left)?;
            get_type(*right)?;
            Ok(Type::Bool)
        }
        ASTNode::Eq { left, right } => {
            // TODO check if types overwrite eq function
            get_type(*left)?;
            get_type(*right)?;
            Ok(Type::Bool)
        }
        ASTNode::Neq { left, right } => {
            // TODO check if types overwrite eq function
            get_type(*left)?;
            get_type(*right)?;
            Ok(Type::Bool)
        }
        ASTNode::IsA { left, right } => {
            get_type(*left)?;
            get_type(*right)?;
            Ok(Type::Bool)
        }
        ASTNode::IsNA { left, right } => {
            get_type(*left)?;
            get_type(*right)?;
            Ok(Type::Bool)
        }

        ASTNode::Not { expr } => get_type_expect(&*expr, &Type::Bool),
        ASTNode::And { left, right } => {
            get_type_expect(&*left, &Type::Bool)?;
            get_type_expect(&*right, &Type::Bool)
        }

        ASTNode::Or { left, right } => {
            get_type_expect(&*left, &Type::Bool)?;
            get_type_expect(&*right, &Type::Bool)
        }

        ASTNode::IfElse { cond, then, _else } => {
            get_type_expect(&*cond, &Type::Bool)?;
            match _else {
                Some(_else) => get_type_expect(&*_else, &get_type(*then)?),
                None => get_type(*then)
            }
        }
        ASTNode::Match { cond, cases } => {
            // TODO check type of cond and cross reference this with types of conditions
            let cond_type = get_type(*cond)?;
            let mut body_type = None;
            for case in cases.iter().map(|node_pos| node_pos.node.clone()) {
                match case {
                    ASTNode::Case { cond, body } => {
                        get_type_expect(&*cond, &cond_type)?;
                        if body_type.is_none() {
                            body_type = Some(get_type(*body)?)
                        } else {
                            get_type_expect(
                                &*body,
                                &body_type.clone().unwrap_or_else(|| unreachable!())
                            )?;
                        }
                    }
                    _ => return Err(String::from("expected case"))
                }
            }
            match body_type.clone() {
                Some(body_type) => Ok(body_type),
                None => Err(String::from("must have at least one arm"))
            }
        }
        ASTNode::Case { .. } => Err(String::from("case cannot be top level")),
        ASTNode::For { expr, body } => match expr.node {
            ASTNode::In { left, right } => {
                match get_type(*right)? {
                    Type::Range { ty } | Type::Set { ty } | Type::List { ty } =>
                        get_type_expect(&*left, ty.as_ref()),
                    _ => get_type(*left)
                }?;
                get_type(*body)?;
                Ok(Type::NA)
            }
            _ => Err(String::from("for must have in statement"))
        },
        ASTNode::In { left, right } => {
            get_type(*left)?;
            get_type(*right)?;
            Ok(Type::Bool)
        }
        ASTNode::Step { amount } => get_type_expect(&*amount, &Type::Int),
        ASTNode::While { cond, body } => {
            get_type_expect(&*cond, &Type::Bool)?;
            get_type(*body)
        }
        ASTNode::Break => Ok(Type::NA),
        ASTNode::Continue => Ok(Type::NA),

        ASTNode::Return { expr } => get_type(*expr),
        ASTNode::ReturnEmpty => Ok(Type::Empty),
        ASTNode::Underscore => Ok(Type::NA),
        ASTNode::Pass => Ok(Type::NA),

        ASTNode::QuestOr { left, right } => {
            let type_left = get_type(*left)?;
            let maybe = get_type_expect(&*right, &type_left)?;
            Ok(Type::Maybe { ty: Box::from(maybe) })
        }
        ASTNode::Print { expr } => get_type(*expr),
        ASTNode::Comment { .. } => Ok(Type::NA)
    }
}

use crate::parser::ast::{ASTNode, ASTNodePos};
use crate::type_checker::context::Context;
use crate::type_checker::type_node::Type;
use crate::type_checker::type_result::TypeResult;

pub fn type_check_expect(
    context: &Context,
    node_pos: &ASTNodePos,
    expected: &Type
) -> Result<Type, String> {
    let node_type = type_check(context, node_pos.clone())?;
    if node_type == *expected {
        Ok(node_type)
    } else {
        Err(format!("Expected {:?}, but was {:?} ({:?})", expected, node_type, node_pos))
    }
}

pub fn type_check(context: &Context, node_pos: ASTNodePos) -> TypeResult<Type> {
    match node_pos.node {
        ASTNode::File { modules, type_defs, .. } => {
            for module in modules {
                type_check(context, module)?;
            }
            for type_def in type_defs {
                type_check(context, type_def)?;
            }
            Ok(Type::NA)
        }
        ASTNode::Import { .. } => Ok(Type::NA),
        ASTNode::FromImport { .. } => Ok(Type::NA),
        ASTNode::Class { body, .. } => {
            type_check(context, *body)?;
            Ok(Type::NA)
        }
        ASTNode::Generic { .. } => Ok(Type::NA),
        ASTNode::Parent { .. } => Ok(Type::NA),
        ASTNode::Script { statements } => {
            let mut last_type = Type::Empty;
            for statement in statements {
                last_type = type_check(context, statement)?;
            }
            Ok(last_type)
        }
        ASTNode::Init => Ok(Type::NA),

        ASTNode::Reassign { left, right } => {
            let left_type = type_check(context, *left)?;
            type_check_expect(context, &*right, &left_type)?;
            Ok(Type::NA)
        }
        ASTNode::Def { definition, .. } => type_check(context, *definition),
        ASTNode::VariableDef { id_maybe_type, expression, .. } => {
            let id_type = match id_maybe_type.node {
                ASTNode::IdType { _type: Some(_type), .. } => type_check(context, *_type)?,
                ASTNode::IdType { .. } => Type::Any,
                _ => return Err(String::from("expected id type"))
            };

            if expression.is_some() {
                let expression = expression.unwrap_or_else(|| unreachable!());
                type_check_expect(context, &*expression, &id_type)
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
                            ASTNode::IdType { _type: Some(_type), .. } =>
                                type_check(context, *_type)?,
                            ASTNode::IdType { .. } => Type::Any,
                            _ => return Err(String::from("Expected id type"))
                        };

                        if default.is_some() {
                            type_check_expect(
                                context,
                                &*default.unwrap_or_else(|| unreachable!()),
                                &id_type
                            )?;
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
            let body_type = type_check(context, *body)?;
            if ret_ty.is_some() {
                let ret_ty = ret_ty.unwrap_or_else(|| unreachable!());
                let function_return_type = type_check(context, *ret_ty)?;
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
                args.iter().map(|arg| type_check(context, arg.clone())).collect();
            let body_type = type_check(context, *body)?;
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
                ty = type_check_expect(context, &element, &ty)?;
            }
            Ok(Type::Set { ty: Box::from(ty) })
        }
        ASTNode::SetBuilder { .. } => unimplemented!(),
        ASTNode::List { elements } => {
            let mut ty = Type::Any;
            for element in elements {
                ty = type_check_expect(context, &element, &ty)?;
            }
            Ok(Type::List { ty: Box::from(ty) })
        }
        ASTNode::ListBuilder { .. } => unimplemented!(),
        ASTNode::Tuple { elements } => {
            let types: TypeResult<Vec<Type>> =
                elements.iter().map(|node_pos| type_check(context, node_pos.clone())).collect();
            Ok(Type::Tuple { tys: types? })
        }

        ASTNode::Range { from, to, .. } => {
            // TODO do something with step
            let from_type = type_check(context, *from)?;
            type_check_expect(context, &*to, &from_type)?;
            Ok(Type::Range { ty: Box::from(from_type) })
        }

        ASTNode::Block { statements } => {
            let mut last_type = Type::Empty;
            for statement in statements {
                last_type = type_check(context, statement)?
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
            type_check(context, *left)?;
            type_check(context, *right)
        }
        ASTNode::AddU { expr } => type_check(context, *expr),
        ASTNode::Sub { left, right } => {
            // TODO check if types overwrite sub function
            type_check(context, *left)?;
            type_check(context, *right)
        }
        ASTNode::SubU { expr } => type_check(context, *expr),
        ASTNode::Mul { left, right } => {
            // TODO check if types overwrite mul function
            type_check(context, *left)?;
            type_check(context, *right)
        }
        ASTNode::Div { left, right } => {
            // TODO check if types overwrite div function
            type_check(context, *left)?;
            type_check(context, *right)
        }
        ASTNode::FDiv { left, right } => {
            // TODO check if types overwrite fdiv function
            type_check(context, *left)?;
            type_check(context, *right)?;
            Ok(Type::Int)
        }
        ASTNode::Mod { left, right } => {
            // TODO check if types overwrite mod function
            type_check(context, *left)?;
            type_check(context, *right)?;
            Ok(Type::Int)
        }
        ASTNode::Pow { left, right } => {
            // TODO check if types overwrite pow function
            type_check(context, *left)?;
            type_check(context, *right)
        }
        ASTNode::Sqrt { expr } => type_check(context, *expr),

        ASTNode::BAnd { .. } => Ok(Type::Int),
        ASTNode::BOr { .. } => Ok(Type::Int),
        ASTNode::BXOr { .. } => Ok(Type::Int),
        ASTNode::BOneCmpl { .. } => Ok(Type::Int),
        ASTNode::BLShift { .. } => Ok(Type::Int),
        ASTNode::BRShift { .. } => Ok(Type::Int),

        ASTNode::Le { left, right } => {
            // TODO check if types overwrite le function
            type_check(context, *left)?;
            type_check(context, *right)?;
            Ok(Type::Bool)
        }
        ASTNode::Ge { left, right } => {
            // TODO check if types overwrite ge function
            type_check(context, *left)?;
            type_check(context, *right)?;
            Ok(Type::Bool)
        }
        ASTNode::Leq { left, right } => {
            // TODO check if types overwrite leq function
            type_check(context, *left)?;
            type_check(context, *right)?;
            Ok(Type::Bool)
        }
        ASTNode::Geq { left, right } => {
            // TODO check if types overwrite geq function
            type_check(context, *left)?;
            type_check(context, *right)?;
            Ok(Type::Bool)
        }
        ASTNode::Is { left, right } => {
            type_check(context, *left)?;
            type_check(context, *right)?;
            Ok(Type::Bool)
        }
        ASTNode::IsN { left, right } => {
            type_check(context, *left)?;
            type_check(context, *right)?;
            Ok(Type::Bool)
        }
        ASTNode::Eq { left, right } => {
            // TODO check if types overwrite eq function
            type_check(context, *left)?;
            type_check(context, *right)?;
            Ok(Type::Bool)
        }
        ASTNode::Neq { left, right } => {
            // TODO check if types overwrite eq function
            type_check(context, *left)?;
            type_check(context, *right)?;
            Ok(Type::Bool)
        }
        ASTNode::IsA { left, right } => {
            type_check(context, *left)?;
            type_check(context, *right)?;
            Ok(Type::Bool)
        }
        ASTNode::IsNA { left, right } => {
            type_check(context, *left)?;
            type_check(context, *right)?;
            Ok(Type::Bool)
        }

        ASTNode::Not { expr } => type_check_expect(context, &*expr, &Type::Bool),
        ASTNode::And { left, right } => {
            type_check_expect(context, &*left, &Type::Bool)?;
            type_check_expect(context, &*right, &Type::Bool)
        }

        ASTNode::Or { left, right } => {
            type_check_expect(context, &*left, &Type::Bool)?;
            type_check_expect(context, &*right, &Type::Bool)
        }

        ASTNode::IfElse { cond, then, _else } => {
            type_check_expect(context, &*cond, &Type::Bool)?;
            match _else {
                Some(_else) => type_check_expect(context, &*_else, &type_check(context, *then)?),
                None => type_check(context, *then)
            }
        }
        ASTNode::Match { cond, cases } => {
            // TODO check type of cond and cross reference this with types of conditions
            let cond_type = type_check(context, *cond)?;
            let mut body_type = None;
            for case in cases.iter().map(|node_pos| node_pos.node.clone()) {
                match case {
                    ASTNode::Case { cond, body } => {
                        type_check_expect(context, &*cond, &cond_type)?;
                        if body_type.is_none() {
                            body_type = Some(type_check(context, *body)?)
                        } else {
                            type_check_expect(
                                context,
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
                match type_check(context, *right)? {
                    Type::Range { ty } | Type::Set { ty } | Type::List { ty } =>
                        type_check_expect(context, &*left, ty.as_ref()),
                    _ => type_check(context, *left)
                }?;
                type_check(context, *body)?;
                Ok(Type::NA)
            }
            _ => Err(String::from("for must have in statement"))
        },
        ASTNode::In { left, right } => {
            type_check(context, *left)?;
            type_check(context, *right)?;
            Ok(Type::Bool)
        }
        ASTNode::Step { amount } => type_check_expect(context, &*amount, &Type::Int),
        ASTNode::While { cond, body } => {
            type_check_expect(context, &*cond, &Type::Bool)?;
            type_check(context, *body)
        }
        ASTNode::Break => Ok(Type::NA),
        ASTNode::Continue => Ok(Type::NA),

        ASTNode::Return { expr } => type_check(context, *expr),
        ASTNode::ReturnEmpty => Ok(Type::Empty),
        ASTNode::Underscore => Ok(Type::Any),
        ASTNode::Pass => Ok(Type::NA),

        ASTNode::QuestOr { left, right } => {
            let type_left = type_check(context, *left)?;
            let maybe = type_check_expect(context, &*right, &type_left)?;
            Ok(Type::Maybe { ty: Box::from(maybe) })
        }
        ASTNode::Print { expr } => type_check(context, *expr),
        ASTNode::Comment { .. } => Ok(Type::NA)
    }
}

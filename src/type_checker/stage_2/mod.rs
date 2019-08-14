use crate::parser::ast::{ASTNode, ASTNodePos};
use crate::type_checker::stage_1::Context;
use crate::type_checker::type_node::Ty;
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
            Ok(Type::new(&Ty::NA))
        }
        ASTNode::Import { .. } => Ok(Type::new(&Ty::NA)),
        ASTNode::FromImport { .. } => Ok(Type::new(&Ty::NA)),
        ASTNode::Class { body, .. } => {
            type_check(context, *body)?;
            Ok(Type::new(&Ty::NA))
        }
        ASTNode::Generic { .. } => Ok(Type::new(&Ty::NA)),
        ASTNode::Parent { .. } => Ok(Type::new(&Ty::NA)),
        ASTNode::Script { statements } => {
            let mut last_type = Type::new(&Ty::Empty);
            for statement in statements {
                last_type = type_check(context, statement)?;
            }
            Ok(last_type)
        }
        ASTNode::Init => Ok(Type::new(&Ty::NA)),

        ASTNode::Reassign { left, right } => {
            let left_type = type_check(context, *left)?;
            type_check_expect(context, &*right, &left_type)?;
            Ok(Type::new(&Ty::NA))
        }
        ASTNode::VarDef { id_maybe_type, expression, .. } => {
            let id_type = match id_maybe_type.node {
                ASTNode::IdType { _type: Some(_type), .. } => type_check(context, *_type)?,
                ASTNode::IdType { .. } => Type::new(&Ty::Any),
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
                            ASTNode::IdType { .. } => Type::new(&Ty::Any),
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
                return Ok(Type::new(&Ty::NA));
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
            Ok(Type::new(&Ty::AnonFun { args: arg_types?, out: Box::new(body_type) }))
        }

        ASTNode::Raises { .. } => Ok(Type::new(&Ty::NA)),
        ASTNode::Raise { .. } => Ok(Type::new(&Ty::Any)),
        ASTNode::Handle { .. } => Ok(Type::new(&Ty::NA)),
        ASTNode::Retry => Ok(Type::new(&Ty::NA)),
        ASTNode::With { .. } => Ok(Type::new(&Ty::NA)),

        ASTNode::FunctionCall { .. } => Ok(Type::new(&Ty::Any)),
        ASTNode::PropertyCall { .. } => Ok(Type::new(&Ty::Any)),
        ASTNode::Id { .. } => Ok(Type::new(&Ty::Any)),

        // TODO implement
        ASTNode::IdType { .. } => Ok(Type::new(&Ty::Any)),
        ASTNode::Condition { .. } => Ok(Type::new(&Ty::NA)),
        ASTNode::FunArg { .. } => Err(String::from("fun arg cannot be top level")),
        ASTNode::TypeDef { .. } => Type::try_from_type(node_pos.node),
        ASTNode::TypeAlias { .. } => Type::try_from_type(node_pos.node),
        ASTNode::TypeTup { .. } => Type::try_from_type(node_pos.node),
        ASTNode::Type { .. } => Type::try_from_type(node_pos.node),
        ASTNode::TypeFun { .. } => Type::try_from_type(node_pos.node),

        ASTNode::_Self => Ok(Type::new(&Ty::NA)),
        ASTNode::AddOp => Ok(Type::new(&Ty::NA)),
        ASTNode::SubOp => Ok(Type::new(&Ty::NA)),
        ASTNode::SqrtOp => Ok(Type::new(&Ty::NA)),
        ASTNode::MulOp => Ok(Type::new(&Ty::NA)),
        ASTNode::FDivOp => Ok(Type::new(&Ty::NA)),
        ASTNode::DivOp => Ok(Type::new(&Ty::NA)),
        ASTNode::PowOp => Ok(Type::new(&Ty::NA)),
        ASTNode::ModOp => Ok(Type::new(&Ty::NA)),
        ASTNode::EqOp => Ok(Type::new(&Ty::NA)),
        ASTNode::LeOp => Ok(Type::new(&Ty::NA)),
        ASTNode::GeOp => Ok(Type::new(&Ty::NA)),

        ASTNode::Set { elements } => {
            let mut ty = Type::new(&Ty::Any);
            for element in elements {
                ty = type_check_expect(context, &element, &ty)?;
            }
            Ok(Type::new(&Ty::Set { ty: Box::from(ty) }))
        }
        ASTNode::SetBuilder { .. } => unimplemented!(),
        ASTNode::List { elements } => {
            let mut ty = Type::new(&Ty::Any);
            for element in elements {
                ty = type_check_expect(context, &element, &ty)?;
            }
            Ok(Type::new(&Ty::List { ty: Box::from(ty) }))
        }
        ASTNode::ListBuilder { .. } => unimplemented!(),
        ASTNode::Tuple { elements } => {
            let types: TypeResult<Vec<Type>> =
                elements.iter().map(|node_pos| type_check(context, node_pos.clone())).collect();
            Ok(Type::new(&Ty::Tuple { tys: types? }))
        }

        ASTNode::Range { from, to, .. } => {
            // TODO do something with step
            let from_type = type_check(context, *from)?;
            type_check_expect(context, &*to, &from_type)?;
            Ok(Type::new(&Ty::Range { ty: Box::from(from_type) }))
        }

        ASTNode::Block { statements } => {
            let mut last_type = Type::new(&Ty::Empty);
            for statement in statements {
                last_type = type_check(context, statement)?
            }
            Ok(last_type)
        }

        ASTNode::Real { .. } => Ok(Type::new(&Ty::Float)),
        ASTNode::Int { .. } => Ok(Type::new(&Ty::Int)),
        ASTNode::ENum { .. } => unimplemented!(),
        ASTNode::Str { .. } => Ok(Type::new(&Ty::String)),
        ASTNode::Bool { .. } => Ok(Type::new(&Ty::Bool)),

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
            Ok(Type::new(&Ty::Int))
        }
        ASTNode::Mod { left, right } => {
            // TODO check if types overwrite mod function
            type_check(context, *left)?;
            type_check(context, *right)?;
            Ok(Type::new(&Ty::Int))
        }
        ASTNode::Pow { left, right } => {
            // TODO check if types overwrite pow function
            type_check(context, *left)?;
            type_check(context, *right)
        }
        ASTNode::Sqrt { expr } => type_check(context, *expr),

        ASTNode::BAnd { .. } => Ok(Type::new(&Ty::Int)),
        ASTNode::BOr { .. } => Ok(Type::new(&Ty::Int)),
        ASTNode::BXOr { .. } => Ok(Type::new(&Ty::Int)),
        ASTNode::BOneCmpl { .. } => Ok(Type::new(&Ty::Int)),
        ASTNode::BLShift { .. } => Ok(Type::new(&Ty::Int)),
        ASTNode::BRShift { .. } => Ok(Type::new(&Ty::Int)),

        ASTNode::Le { left, right } => {
            // TODO check if types overwrite le function
            type_check(context, *left)?;
            type_check(context, *right)?;
            Ok(Type::new(&Ty::Bool))
        }
        ASTNode::Ge { left, right } => {
            // TODO check if types overwrite ge function
            type_check(context, *left)?;
            type_check(context, *right)?;
            Ok(Type::new(&Ty::Bool))
        }
        ASTNode::Leq { left, right } => {
            // TODO check if types overwrite leq function
            type_check(context, *left)?;
            type_check(context, *right)?;
            Ok(Type::new(&Ty::Bool))
        }
        ASTNode::Geq { left, right } => {
            // TODO check if types overwrite geq function
            type_check(context, *left)?;
            type_check(context, *right)?;
            Ok(Type::new(&Ty::Bool))
        }
        ASTNode::Is { left, right } => {
            type_check(context, *left)?;
            type_check(context, *right)?;
            Ok(Type::new(&Ty::Bool))
        }
        ASTNode::IsN { left, right } => {
            type_check(context, *left)?;
            type_check(context, *right)?;
            Ok(Type::new(&Ty::Bool))
        }
        ASTNode::Eq { left, right } => {
            // TODO check if types overwrite eq function
            type_check(context, *left)?;
            type_check(context, *right)?;
            Ok(Type::new(&Ty::Bool))
        }
        ASTNode::Neq { left, right } => {
            // TODO check if types overwrite eq function
            type_check(context, *left)?;
            type_check(context, *right)?;
            Ok(Type::new(&Ty::Bool))
        }
        ASTNode::IsA { left, right } => {
            type_check(context, *left)?;
            type_check(context, *right)?;
            Ok(Type::new(&Ty::Bool))
        }
        ASTNode::IsNA { left, right } => {
            type_check(context, *left)?;
            type_check(context, *right)?;
            Ok(Type::new(&Ty::Bool))
        }

        ASTNode::Not { expr } => type_check_expect(context, &*expr, &Type::new(&Ty::Bool)),
        ASTNode::And { left, right } => {
            type_check_expect(context, &*left, &Type::new(&&Ty::Bool))?;
            type_check_expect(context, &*right, &Type::new(&&Ty::Bool))
        }

        ASTNode::Or { left, right } => {
            type_check_expect(context, &*left, &Type::new(&&Ty::Bool))?;
            type_check_expect(context, &*right, &Type::new(&&Ty::Bool))
        }

        ASTNode::IfElse { cond, then, _else } => {
            type_check_expect(context, &*cond, &Type::new(&&Ty::Bool))?;
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
                match type_check(context, *right)?.ty {
                    Ty::Range { ty } | Ty::Set { ty } | Ty::List { ty } =>
                        type_check_expect(context, &*left, ty.as_ref()),
                    _ => type_check(context, *left)
                }?;
                type_check(context, *body)?;
                Ok(Type::new(&Ty::NA))
            }
            _ => Err(String::from("for must have in statement"))
        },
        ASTNode::In { left, right } => {
            type_check(context, *left)?;
            type_check(context, *right)?;
            Ok(Type::new(&Ty::Bool))
        }
        ASTNode::Step { amount } => type_check_expect(context, &*amount, &Type::new(&&Ty::Int)),
        ASTNode::While { cond, body } => {
            type_check_expect(context, &*cond, &Type::new(&&Ty::Bool))?;
            type_check(context, *body)
        }
        ASTNode::Break => Ok(Type::new(&Ty::NA)),
        ASTNode::Continue => Ok(Type::new(&Ty::NA)),

        ASTNode::Return { expr } => type_check(context, *expr),
        ASTNode::ReturnEmpty => Ok(Type::new(&Ty::Empty)),
        ASTNode::Underscore => Ok(Type::new(&Ty::Any)),
        ASTNode::Pass => Ok(Type::new(&Ty::NA)),

        ASTNode::QuestOr { left, right } => {
            let type_left = type_check(context, *left)?;
            Ok(type_check_expect(context, &*right, &type_left)?.into_optional())
        }
        ASTNode::Print { expr } => type_check(context, *expr),
        ASTNode::Comment { .. } => Ok(Type::new(&Ty::NA))
    }
}

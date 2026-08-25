use crate::backend::python::ast::node::PythonCore;
use crate::backend::python::convert::convert_node;
use crate::backend::python::convert::state::{Imports, State};
use crate::backend::python::name::ToPy;
use crate::backend::python::result::{GenResult, UnimplementedErr};
use crate::check::ast::NodeTy;
use crate::{ASTTy, Context};

pub fn convert_handle(ast: &ASTTy, imp: &mut Imports, state: &State, ctx: &Context) -> GenResult {
    Ok(match &ast.node {
        NodeTy::Raise { error } => PythonCore::Raise {
            error: Box::from(convert_node(error, imp, state, ctx)?),
        },

        NodeTy::Handle {
            expr_or_stmt,
            cases,
        } => {
            let (var, ty) = if let NodeTy::VariableDef { var, ty, .. } = &expr_or_stmt.node {
                let ty = ty.as_ref().map(|ty| ty.to_py(imp)).map(Box::from);
                (Some(Box::from(convert_node(var, imp, state, ctx)?)), ty)
            } else {
                (None, None)
            };

            let assign_state = state.must_assign_to(var.as_deref(), expr_or_stmt.ty.clone());
            PythonCore::TryExcept {
                setup: var.map(|var| {
                    Box::from(PythonCore::VarDef {
                        var,
                        ty,
                        expr: None,
                    })
                }),
                attempt: Box::from(convert_node(&expr_or_stmt.clone(), imp, state, ctx)?),
                except: {
                    let mut except = Vec::new();
                    for case in cases {
                        let (cond, body) = match &case.node {
                            NodeTy::Case { cond, body } => (cond, body),
                            other => {
                                let msg = format!("Expected case, was {other:?}");
                                return Err(Box::from(UnimplementedErr::new(case, &msg)));
                            }
                        };

                        match &cond.node {
                            NodeTy::ExpressionType { expr, ty, .. } => {
                                let expr = Box::from(convert_node(expr, imp, state, ctx)?);
                                let class = Box::from(ty.as_ref().map_or_else(
                                    || panic!("handle case must have class"),
                                    |ty| ty.to_py(imp),
                                ));
                                let body = Box::from(convert_node(body, imp, &assign_state, ctx)?);

                                except.push(if *expr == PythonCore::UnderScore {
                                    PythonCore::Except { class, body }
                                } else {
                                    PythonCore::ExceptId {
                                        id: expr,
                                        class,
                                        body,
                                    }
                                });
                            }
                            other => {
                                let msg = format!("Expected id type, was {other:?}");
                                return Err(Box::from(UnimplementedErr::new(case, &msg)));
                            }
                        };
                    }
                    except
                },
            }
        }
        other => {
            let msg = format!("Expected handle, was {other:?}");
            return Err(Box::from(UnimplementedErr::new(ast, &msg)));
        }
    })
}

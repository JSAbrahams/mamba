use crate::{ASTTy, Context};
use crate::check::ast::NodeTy;
use crate::generate::ast::node::Core;
use crate::generate::convert::convert_node;
use crate::generate::convert::state::{Imports, State};
use crate::generate::result::{GenResult, UnimplementedErr};

pub fn convert_handle(ast: &ASTTy, imp: &mut Imports, state: &State, ctx: &Context) -> GenResult {
    Ok(match &ast.node {
        NodeTy::Raise { error } => Core::Raise { error: Box::from(convert_node(error, imp, state, ctx)?) },

        NodeTy::Handle { expr_or_stmt, cases } => {
            let (var, ty) = if let NodeTy::VariableDef { var, ty, .. } = &expr_or_stmt.node {
                (
                    Some(Box::from(convert_node(var, imp, state, ctx)?)),
                    if let Some(ty) = ty {
                        Some(Box::from(convert_node(ty, imp, state, ctx)?))
                    } else {
                        None
                    },
                )
            } else {
                (None, None)
            };
            let assign_state = state.must_assign_to(var.as_deref(), expr_or_stmt.ty.clone());

            Core::TryExcept {
                setup: var.map(|var| Box::from(Core::VarDef { var, ty, expr: None })),
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
                                let Some(ty) = ty else {
                                    let msg = format!("Must have condition, was {cond:?}");
                                    return Err(Box::from(UnimplementedErr::new(cond, &msg)));
                                };
                                let class = Box::from(convert_node(ty, imp, state, ctx)?);
                                let body = Box::from(convert_node(body, imp, &assign_state, ctx)?);

                                except.push(if *expr == Core::UnderScore {
                                    Core::Except { class, body }
                                } else {
                                    Core::ExceptId { id: expr, class, body }
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

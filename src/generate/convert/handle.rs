use crate::generate::ast::node::Core;
use crate::generate::convert::convert_node;
use crate::generate::convert::state::{Imports, State};
use crate::generate::result::GenResult;
use crate::parse::ast::{AST, Node};

pub fn convert_handle(ast: &AST, imp: &mut Imports, state: &State) -> GenResult {
    Ok(match &ast.node {
        Node::Raises { expr_or_stmt, .. } => convert_node(expr_or_stmt, imp, state)?,
        Node::Raise { error } => Core::Raise { error: Box::from(convert_node(error, imp, state)?) },

        Node::Handle { expr_or_stmt, cases } => {
            let (var, ty) = if let Node::VariableDef { var, ty, .. } = &expr_or_stmt.node {
                (
                    Some(Box::from(convert_node(var, imp, state)?)),
                    if let Some(ty) = ty {
                        Some(Box::from(convert_node(ty, imp, state)?))
                    } else {
                        None
                    },
                )
            } else {
                (None, None)
            };
            let assign_state = state.assign_to(var.as_deref());

            Core::TryExcept {
                setup: var.map(|var| Box::from(Core::VarDef { var, ty, expr: None })),
                attempt: Box::from(convert_node(&expr_or_stmt.clone(), imp, state)?),
                except: {
                    let mut except = Vec::new();
                    for case in cases {
                        let (cond, body) = match &case.node {
                            Node::Case { cond, body } => (cond, body),
                            other => panic!("Expected case but was {:?}", other),
                        };

                        match &cond.node {
                            Node::ExpressionType { expr, ty, .. } => except.push(Core::Except {
                                id: Box::from(convert_node(expr, imp, state)?),
                                class: if let Some(ty) = ty {
                                    Some(Box::from(convert_node(ty, imp, state)?))
                                } else {
                                    None
                                },
                                body: Box::from(convert_node(body, imp, &assign_state)?),
                            }),
                            other => panic!("Expected id type but was {:?}", other),
                        };
                    }
                    except
                },
            }
        }
        other => panic!("Expected handle {:?}", other)
    })
}
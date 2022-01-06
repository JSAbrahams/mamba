use std::ops::Deref;

use crate::check::context::{arg, function};
use crate::core::construct::Core;
use crate::desugar::common::desugar_vec;
use crate::desugar::node::desugar_node;
use crate::desugar::result::DesugarResult;
use crate::desugar::state::Imports;
use crate::desugar::state::State;
use crate::desugar::ty::desugar_type;
use crate::parse::ast::AST;
use crate::parse::ast::Node;

/// Desugar a class.
///
/// If a class has inline arguments (arguments next to class), then we create a
/// constructor and assume that there is no constructor in the body of a class.
/// This property should be ensured by the type checker.
///
/// We add arguments and calls to super for parents.
pub fn desugar_class(ast: &AST, imp: &mut Imports, state: &State) -> DesugarResult {
    match &ast.node {
        Node::TypeAlias { ty, isa, .. } => {
            let parents = vec![isa.deref().clone()];
            let body = None;
            extract_class(ty, &body, &[], &parents, imp, &state.in_interface(true))
        }
        Node::TypeDef { ty, body, isa } => {
            let parents = if let Some(isa) = isa { vec![isa.deref().clone()] } else { vec![] };
            extract_class(ty, body, &[], &parents, imp, &state.in_interface(true))
        }
        Node::Class { ty, body, args, parents } =>
            extract_class(ty, body, args, parents, imp, &state.in_interface(false)),

        other => panic!("Expected class or type definition but was {:?}", other)
    }
}

fn extract_class(
    ty: &AST,
    body: &Option<Box<AST>>,
    args: &[AST],
    parents: &[AST],
    imp: &mut Imports,
    state: &State,
) -> DesugarResult {
    let statements = if let Some(body) = body {
        match &body.deref().node {
            Node::Block { statements } => statements.clone(),
            _ => vec![]
        }
    } else {
        vec![]
    };

    match &ty.node {
        Node::Type { id, .. } => {
            let (parent_names, parent_args, super_calls) = extract_parents(parents, imp, state)?;
            let core_definitions: Vec<Core> = desugar_vec(&statements, imp, state)?;
            let inline_args = desugar_vec(args, imp, state)?;

            let final_definitions = if parent_names.is_empty() && inline_args.is_empty() {
                desugar_vec(&statements, imp, state)?
            } else {
                let (found_constructor, augmented_definitions) =
                    add_parent_to_constructor(&core_definitions, &super_calls)?;

                if found_constructor && !args.is_empty() {
                    panic!("Cannot have explicit constructor and inline arguments.")
                } else if found_constructor {
                    augmented_definitions
                } else {
                    constructor_from_inline(
                        &inline_args,
                        &parent_args,
                        &super_calls,
                        &augmented_definitions,
                    )?
                }
            };

            let mut final_definitions = if final_definitions.is_empty() {
                vec![Core::FunDef {
                    id: Box::new(Core::Id { lit: String::from(function::python::INIT) }),
                    arg: vec![Core::FunArg {
                        vararg: false,
                        var: Box::new(Core::Id { lit: String::from(arg::python::SELF) }),
                        ty: None,
                        default: None,
                    }],
                    ty: None,
                    body: Box::new(Core::Pass),
                }]
            } else {
                final_definitions
            };

            let (mut stmts, mut non_variables): (Vec<_>, Vec<_>) =
                final_definitions.into_iter().partition(|stmt| matches!(stmt, Core::VarDef { .. }));
            stmts.append(&mut non_variables);
            final_definitions = stmts;

            Ok(Core::ClassDef {
                name: Box::from(desugar_node(id, imp, state)?),
                parents: parent_names,
                definitions: final_definitions,
            })
        }
        other => panic!("Didn't recognize while making class: {:?}.", other)
    }
}

// TODO simplify application logic
fn constructor_from_inline(
    inline_args: &[Core],
    parent_args: &[Core],
    super_calls: &[Core],
    definitions: &[Core],
) -> DesugarResult<Vec<Core>> {
    let mut final_definitions = vec![];
    let mut arg = vec![Core::Id { lit: String::from("self") }];
    let mut statements = Vec::from(super_calls);

    for inline_arg in inline_args {
        match inline_arg {
            Core::FunArg { var, .. } => {
                arg.push(inline_arg.clone());
                if !parent_args.contains(&var) {
                    final_definitions
                        .push(Core::Assign { left: var.clone(), right: Box::from(Core::None) })
                }
            }

            Core::VarDef { var, ty, expr, .. } => {
                arg.push(Core::FunArg {
                    vararg: false,
                    var: var.clone(),
                    ty: ty.clone(),
                    default: match &expr.deref() {
                        Some(expr) => Some(expr.clone()),
                        _ => None
                    },
                });

                if !parent_args.contains(&var) {
                    final_definitions.push(inline_arg.clone());
                    statements.push(Core::Assign {
                        left: Box::from(Core::PropertyCall {
                            object: Box::new(Core::Id { lit: String::from("self") }),
                            property: var.clone(),
                        }),
                        right: var.clone(),
                    });
                }
            }
            _ => panic!("Inline arg was not function argument: {:?}", inline_arg)
        }
    }

    let id = Box::from(Core::Id { lit: String::from("init") });
    let body = Box::from(Core::Block { statements });
    let core_init = Core::FunDef { id, arg, ty: None, body };

    final_definitions.push(core_init);
    final_definitions.append(&mut Vec::from(definitions));
    Ok(final_definitions)
}

fn add_parent_to_constructor(
    core_definitions: &[Core],
    super_calls: &[Core],
) -> DesugarResult<(bool, Vec<Core>)> {
    let mut final_definitions = vec![];
    let mut found_constructor = false;

    for definition in core_definitions {
        final_definitions.push(
            if let Core::FunDef { id, arg, body: old_body, .. } = definition {
                if let Core::Id { lit, .. } = id.clone().deref() {
                    if lit == "init" {
                        if found_constructor {
                            panic!("Cannot have more than one constructor.")
                        }
                        found_constructor = true;
                        let body = match (super_calls.is_empty(), *old_body.clone()) {
                            (true, _) => old_body.clone(),
                            (false, Core::Block { statements: old_statements }) => {
                                let mut statements = Vec::from(super_calls);
                                statements.append(&mut old_statements.clone());
                                Box::from(Core::Block { statements })
                            }
                            (false, core) => {
                                let mut statements = Vec::from(super_calls);
                                statements.push(core);
                                Box::from(Core::Block { statements })
                            }
                        };

                        Core::FunDef { id: id.clone(), arg: arg.clone(), ty: None, body }
                    } else {
                        definition.clone()
                    }
                } else {
                    definition.clone()
                }
            } else {
                definition.clone()
            }
        );
    }

    Ok((found_constructor, final_definitions))
}

fn extract_parents(
    parents: &[AST],
    ctx: &mut Imports,
    state: &State,
) -> DesugarResult<(Vec<Core>, Vec<Core>, Vec<Core>)> {
    let mut parent_names: Vec<Core> = vec![];
    let mut parent_args: Vec<Core> = vec![];
    let mut super_calls: Vec<Core> = vec![];

    for parent in parents {
        match &parent.node {
            Node::Parent { ty, args: old_args } => {
                let parent_name = desugar_type(ty, ctx, state)?;
                parent_names.push(parent_name.clone());

                let mut args = vec![];
                args.append(&mut desugar_vec(old_args, ctx, state)?);
                parent_args.append(&mut desugar_vec(old_args, ctx, state)?);

                super_calls.push(Core::PropertyCall {
                    object: Box::from(Core::FunctionCall {
                        function: Box::from(Core::Id { lit: String::from("super") }),
                        args: vec![parent_name, Core::Id {
                            lit: String::from(arg::python::SELF)
                        }],
                    }),
                    property: Box::from(Core::FunctionCall {
                        function: Box::from(Core::Id { lit: String::from("__init__") }),
                        args,
                    }),
                });
            }
            other => panic!("Expected parent, was {:?}", other)
        }
    }

    Ok((parent_names, parent_args, super_calls))
}

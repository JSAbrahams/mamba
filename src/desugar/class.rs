use crate::core::construct::Core;
use crate::desugar::common::desugar_vec;
use crate::desugar::context::Context;
use crate::desugar::context::State;
use crate::desugar::node::desugar_node;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use std::ops::Deref;

/// Desugar a class.
///
/// If a class has inline arguments (arguments next to class), then we create a
/// constructor and assume that there is no constructor in the body of a class.
/// This property should be ensured by the type checker.
///
/// We add arguments and calls to super for parents.
pub fn desugar_class(node: &ASTNode, ctx: &Context, state: &State) -> Core {
    match node {
        ASTNode::TypeDef { _type, body: Some(body) } => match (&_type.node, &body.node) {
            (ASTNode::Type { id, .. }, ASTNode::Block { statements }) => Core::ClassDef {
                name:        Box::from(desugar_node(id, ctx, state)),
                parents:     Vec::new(),
                definitions: desugar_vec(statements, ctx, &State {
                    tup:         state.tup,
                    expect_expr: state.expect_expr,
                    interface:   true
                })
            },
            other => panic!("desugar didn't recognize while making type definition: {:?}.", other)
        },
        ASTNode::TypeDef { _type, body: None } => match &_type.node {
            ASTNode::Type { id, .. } => Core::ClassDef {
                name:        Box::from(desugar_node(id, ctx, state)),
                parents:     Vec::new(),
                definitions: Vec::new()
            },
            other => panic!("desugar didn't recognize while making type definition: {:?}.", other)
        },

        ASTNode::Class { _type, body, args, parents } => match (&_type.node, &body.node) {
            (ASTNode::Type { id, .. }, ASTNode::Block { statements }) => {
                let (parent_names, parent_args, super_calls) = extract_parents(parents, ctx, state);
                let core_definitions: Vec<Core> = desugar_vec(statements, ctx, state);
                let inline_args = desugar_vec(args, ctx, state);

                let final_definitions = if parent_names.is_empty() && inline_args.is_empty() {
                    desugar_vec(statements, ctx, state)
                } else {
                    let (found_constructor, augmented_definitions) =
                        add_parent_to_constructor(&core_definitions, &parent_args, &super_calls);

                    if found_constructor && !args.is_empty() {
                        panic!("Cannot have explicit constructor and inline arguments.")
                    } else if found_constructor {
                        augmented_definitions
                    } else {
                        constructor_from_inline(
                            &inline_args,
                            &parent_args,
                            &super_calls,
                            &augmented_definitions
                        )
                    }
                };

                Core::ClassDef {
                    name:        Box::from(desugar_node(id, ctx, state)),
                    parents:     parent_names,
                    definitions: final_definitions
                }
            }
            other => panic!("desugarer didn't recognize while making class: {:?}.", other)
        },
        other => panic!("Expected class or type definition but was {:?}", other)
    }
}

fn constructor_from_inline(
    inline_args: &[Core],
    parent_args: &[Core],
    super_calls: &[Core],
    definitions: &[Core]
) -> Vec<Core> {
    let mut final_definitions = vec![];
    let mut args = vec![Core::Id { lit: String::from("self") }];

    for inline_arg in inline_args {
        if let Core::FunArg { id, .. } = inline_arg {
            args.push(inline_arg.clone());
            if !parent_args.contains(&id) {
                final_definitions
                    .push(Core::Assign { left: id.clone(), right: Box::from(Core::None) })
            }
        } else {
            panic!("Inline arg was not function argument.")
        }
    }

    let id = Box::from(Core::Id { lit: String::from("init") });
    let body = Box::from(Core::Block { statements: Vec::from(super_calls) });
    let core_init = Core::FunDef { private: false, id, args, body };
    final_definitions.push(core_init);
    final_definitions.append(&mut Vec::from(definitions));

    final_definitions
}

fn add_parent_to_constructor(
    core_definitions: &[Core],
    parent_args: &[Core],
    super_calls: &[Core]
) -> (bool, Vec<Core>) {
    let mut final_definitions = vec![];
    let mut found_constructor = false;

    for definition in core_definitions {
        final_definitions.push(
            if let Core::FunDef { private, id, args: old_args, body: old_body } = definition {
                if let Core::Id { lit } = id.clone().deref() {
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

                        let mut args = old_args.clone();
                        args.append(&mut Vec::from(parent_args));

                        Core::FunDef { private: *private, id: id.clone(), args, body }
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

    (found_constructor, final_definitions)
}

fn extract_parents(
    parents: &[ASTNodePos],
    ctx: &Context,
    state: &State
) -> (Vec<Core>, Vec<Core>, Vec<Core>) {
    let mut parent_names: Vec<Core> = vec![];
    let mut parent_args: Vec<Core> = vec![];
    let mut super_calls: Vec<Core> = vec![];

    for parent in parents {
        match &parent.node {
            ASTNode::Parent { ref id, args: old_args, .. } => {
                parent_names.push(desugar_node(id, ctx, state));

                let mut args = vec![Core::Id { lit: String::from("self") }];
                args.append(&mut desugar_vec(old_args, ctx, state));
                parent_args.append(&mut desugar_vec(old_args, ctx, state));

                super_calls.push(Core::PropertyCall {
                    object:   Box::from(Core::FunctionCall {
                        function: Box::from(Core::Id { lit: String::from("super") }),
                        args:     vec![]
                    }),
                    property: Box::from(Core::Id { lit: String::from("__init__") })
                });
            }
            other => panic!("Expected parent but got {:?}", other)
        }
    }

    (parent_names, parent_args, super_calls)
}

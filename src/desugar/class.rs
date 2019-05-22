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
        ASTNode::Class { _type, body, args, parents } => match (&_type.node, &body.node) {
            (ASTNode::Type { id, .. }, ASTNode::Block { statements }) => {
                let (parent_names, parent_args, super_calls) = extract_parents(parents, ctx, state);
                let core_definitions: Vec<Core> = desugar_vec(statements, ctx, state);

                let inline_args = desugar_vec(args, ctx, state);

                let (found_constructor, augmented_definitions) =
                    augment_constructor(&core_definitions, &super_calls);
                let final_definitions =
                    if found_constructor || (inline_args.is_empty() && parent_args.is_empty()) {
                        augmented_definitions
                    } else {
                        // We have to create a constructor because there was none present, and we
                        // need to convert inline args to init call and/or make calls to super
                        let mut final_definitions = augmented_definitions.clone();
                        let mut args = vec![Core::Id { lit: String::from("self") }];

                        for inline_arg in inline_args {
                            if let Core::FunArg { id, .. } = inline_arg.clone() {
                                args.push(inline_arg);
                                if !parent_args.contains(&id) {
                                    final_definitions.push(Core::Assign {
                                        left:  id.clone(),
                                        right: Box::from(Core::None)
                                    })
                                }
                            } else {
                                panic!("Inline arg was not function argument")
                            }
                        }

                        let id = Box::from(Core::Id { lit: String::from("__init__") });
                        let body = Box::from(Core::Block { statements: super_calls });
                        let core_init = Core::FunDef { private: false, id, args, body };
                        final_definitions.push(core_init);

                        final_definitions
                    };

                Core::ClassDef {
                    name:        Box::from(desugar_node(id, ctx, state)),
                    parents:     parent_names,
                    definitions: final_definitions
                }
            }
            other => panic!("desugarer didn't recognize while making class: {:?}.", other)
        },
        other => panic!("Expected class but was {:?}", other)
    }
}

fn augment_constructor(core_definitions: &[Core], super_calls: &[Core]) -> (bool, Vec<Core>) {
    let mut final_definitions = vec![];
    let mut found_constructor = false;

    for definition in core_definitions {
        final_definitions.push(
            if let Core::FunDef { private, id, args: old_args, body: old_body } = definition {
                if let Core::Id { lit } = id.clone().deref() {
                    if lit == "__init__" {
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

                        Core::FunDef {
                            private: *private,
                            id: id.clone(),
                            args: old_args.clone(),
                            body
                        }
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
            ASTNode::Parent { ref id, args, .. } => {
                parent_names.push(desugar_node(id, ctx, state));

                let args = desugar_vec(args, ctx, state);
                parent_args.append(&mut args.clone());

                super_calls.push(Core::MethodCall {
                    object: Box::from(Core::MethodCall {
                        object: Box::from(Core::Empty),
                        method: String::from("super"),
                        args:   vec![]
                    }),
                    method: String::from("__init__"),
                    args
                });
            }
            other => panic!("Expected parent but got {:?}", other)
        }
    }

    (parent_names, parent_args, super_calls)
}

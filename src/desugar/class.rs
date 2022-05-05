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

            let name = Box::from(desugar_node(id, imp, state)?);
            Ok(if final_definitions.len() > 1 {
                Core::ClassDef { name, parents: parent_names, definitions: final_definitions }
            } else if final_definitions.len() == 1 {
                let expr = Option::from(Box::from(parent));
                Core::VarDef { var: name, ty: None, expr }
            } else {
                let expr = Option::from(Box::from(Core::Tuple { elements: parent_names }));
                Core::VarDef { var: name, ty: None, expr }
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
                if !parent_args.contains(var) {
                    final_definitions
                        .push(Core::Assign { left: var.clone(), right: Box::from(Core::None) })
                }
            }

            Core::VarDef { var, ty, expr, .. } => {
                arg.push(Core::FunArg {
                    vararg: false,
                    var: var.clone(),
                    ty: ty.clone(),
                    default: expr.deref().as_ref().cloned(),
                });

                if !parent_args.contains(var) {
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


#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use crate::common::position::Position;
    use crate::core::construct::Core;
    use crate::desugar::desugar;
    use crate::parse::ast::AST;
    use crate::parse::ast::Node;

    macro_rules! to_pos_unboxed {
        ($node:expr) => {{
            AST { pos: Position::default(), node: $node }
        }};
    }

    macro_rules! to_pos {
        ($node:expr) => {{
            Box::from(to_pos_unboxed!($node))
        }};
    }

    #[test]
    fn import_verify() {
        let import = vec![
            to_pos_unboxed!(Node::ENum { num: String::from("a"), exp: String::from("100") }),
            to_pos_unboxed!(Node::Real { lit: String::from("3000.5") }),
        ];
        let aliases = vec![];
        let import = to_pos!(Node::Import { import, aliases });

        let core_import = match desugar(&import) {
            Ok(Core::Import { imports }) => imports,
            other => panic!("Expected tuple but got {:?}", other)
        };

        assert_eq!(core_import.len(), 2);
        assert_eq!(core_import[0], Core::ENum { num: String::from("a"), exp: String::from("100") });
        assert_eq!(core_import[1], Core::Float { float: String::from("3000.5") });
    }

    #[test]
    fn import_as_verify() {
        let import = vec![
            to_pos_unboxed!(Node::ENum { num: String::from("a"), exp: String::from("100") }),
            to_pos_unboxed!(Node::Real { lit: String::from("3000.5") }),
        ];
        let aliases = vec![to_pos_unboxed!(Node::Real { lit: String::from("0.5") })];
        let import = to_pos!(Node::Import { import, aliases });

        let (core_import, core_as) = match desugar(&import) {
            Ok(Core::ImportAs { imports, alias }) => (imports, alias),
            other => panic!("Expected import but got {:?}", other)
        };

        assert_eq!(core_import.len(), 2);
        assert_eq!(core_import[0], Core::ENum { num: String::from("a"), exp: String::from("100") });
        assert_eq!(core_import[1], Core::Float { float: String::from("3000.5") });
        assert_eq!(core_as.len(), 1);
        assert_eq!(core_as[0], Core::Float { float: String::from("0.5") });
    }

    #[test]
    fn from_import_verify() {
        let id = to_pos!(Node::Id { lit: String::from("afs") });
        let import = vec![
            to_pos_unboxed!(Node::ENum { num: String::from("a"), exp: String::from("100") }),
            to_pos_unboxed!(Node::Real { lit: String::from("3000.5") }),
        ];
        let import =
            to_pos!(Node::FromImport { id, import: to_pos!(Node::Import { import, aliases: vec![] }) });

        let (from, import) = match desugar(&import) {
            Ok(Core::FromImport { from, import }) => match &import.deref() {
                Core::Import { imports } => (from.clone(), imports.clone()),
                other => panic!("Expected import but got {:?}", other)
            },
            other => panic!("Expected from import but got {:?}", other)
        };

        assert_eq!(*from, Core::Id { lit: String::from("afs") });
        assert_eq!(import.len(), 2);
        assert_eq!(import[0], Core::ENum { num: String::from("a"), exp: String::from("100") });
        assert_eq!(import[1], Core::Float { float: String::from("3000.5") });
    }

    #[test]
    fn condition_verify() {
        let cond = to_pos!(Node::Bool { lit: true });
        let condition = to_pos!(Node::Condition { cond, el: None });

        let result = desugar(&condition);
        assert!(result.is_err());
    }

    #[test]
    fn type_alias() {
        let alias = to_pos!(Node::Class {
            ty: to_pos!(Node::Type { id: to_pos!(Node::Id { lit: String::from("MyErr1") }), generics: vec![] }),
            args: vec![],
            parents: vec![
                to_pos_unboxed!(Node::Parent {
                    ty: to_pos!(Node::Type { id: to_pos!(Node::Id { lit: String::from("Exception") }), generics: vec![] }),
                    args: vec![to_pos_unboxed!(Node::Str { lit: String::from("Something went wrong"), expressions: vec![] })]})],
            body: None });

        let (var, ty, expr) = match desugar(&alias) {
            Ok(Core::VarDef { var, ty, expr }) => (*var.clone(), ty.clone(), expr.clone()),
            other => panic!("Expected type alias but got {:?}", other)
        };

        assert_eq!(var, Core::Id { lit: String::from("MyErr1") });
        assert_eq!(ty, None);
        assert!(expr.is_some());
        match expr.clone().unwrap().deref() {
            Core::FunctionCall { function, args } => {
                assert_eq!(*function.deref(), Core::Id { lit: String::from("Exception") });
                assert_eq!(args.len(), 1);
                assert_eq!(*args.first().unwrap(), Core::Str { string: String::from("Something went wrong") })
            }
            _ => panic!("Expected function call, was {:?}", expr.clone())
        }
    }


    #[test]
    fn type_alias_with_arguments() {
        let alias = to_pos!(Node::Class {
            ty: to_pos!(Node::Type { id: to_pos!(Node::Id { lit: String::from("MyErr1") }), generics: vec![] }),
            args: vec![to_pos_unboxed!(Node::FunArg {
                vararg: false,
                mutable: false,
                var: to_pos!(Node::Id {lit: String::from("a1")}),
                ty: None,
                default: None
            })],
            parents: vec![
                to_pos_unboxed!(Node::Parent {
                    ty: to_pos!(Node::Type { id: to_pos!(Node::Id { lit: String::from("Exception") }), generics: vec![] }),
                    args: vec![to_pos_unboxed!(Node::Id { lit: String::from("a1") })]})],
            body: None });

        let (name, parents, definitions) = match desugar(&alias) {
            Ok(Core::ClassDef { name, parents, definitions }) => (name, parents, definitions),
            other => panic!("Expected class def but got {:?}", other)
        };
    }
}

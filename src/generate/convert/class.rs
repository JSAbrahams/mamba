use std::ops::Deref;

use crate::check::context::{arg, function};
use crate::generate::ast::node::Core;
use crate::generate::convert::common::convert_vec;
use crate::generate::convert::convert_node;
use crate::generate::convert::state::{Imports, State};
use crate::generate::result::{GenResult, UnimplementedErr};
use crate::parse::ast::{AST, Node};

/// Desugar a class.
///
/// If a class has inline arguments (arguments next to class), then we create a
/// constructor and assume that there is no constructor in the body of a class.
/// This property should be ensured by the type checker.
///
/// We add arguments and calls to super for parents.
pub fn convert_class(ast: &AST, imp: &mut Imports, state: &State) -> GenResult {
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
        Node::Class { ty, body, args, parents } => {
            extract_class(ty, body, args, parents, imp, &state.in_interface(false))
        }

        Node::Parent { ty, args } => {
            if args.is_empty() {
                convert_node(ty, imp, state)
            } else {
                Ok(Core::FunctionCall {
                    function: Box::from(match convert_node(ty, imp, state)? {
                        Core::Type { lit, .. } => Core::Id { lit }, // ignore generics
                        other => other,
                    }),
                    args: convert_vec(args, imp, state)?,
                })
            }
        }

        other => panic!("Expected class or type definition but was {:?}", other),
    }
}

/// Extract class.
///
/// Construct custom constructor to call parents if:
/// - There are class arguments
/// - There are multiple parents
/// - The class has a body and one or more parents has class arguments
fn extract_class(
    ty: &AST,
    body: &Option<Box<AST>>,
    args: &[AST],
    parents: &[AST],
    imp: &mut Imports,
    state: &State,
) -> GenResult {
    let id = match &ty.node {
        Node::Type { id, .. } => convert_node(id, imp, state)?,
        _ => return Err(UnimplementedErr::new(ty, "Other than type as class identifier")),
    };

    let mut args = convert_vec(args, imp, &state.def_as_fun_arg(true))?;
    let parents = convert_vec(parents, imp, state)?;

    let body = match body {
        Some(body) => Some(convert_node(body, imp, state)?),
        _ => None,
    };

    let body = if !args.is_empty()
        || parents.len() > 1
        || (body.is_some() && parents.iter().any(|p| matches!(p, Core::FunctionCall { .. })))
    {
        let old_stmts = match body {
            Some(Core::Block { statements }) => statements,
            None => vec![],
            Some(..) => return Err(UnimplementedErr::new(ty, "Body not block")),
        };

        let parents: Vec<(String, Vec<Core>)> = parents
            .iter()
            .map(|parent| match parent.clone() {
                Core::FunctionCall { function, args } => match *function {
                    Core::Id { lit } => Ok((lit, args)),
                    _ => Err(UnimplementedErr::new(ty, "Parent and custom constructor")),
                },
                Core::Type { lit, .. } => Ok((lit, vec![])),
                _ => Err(UnimplementedErr::new(ty, "Parent and custom constructor")),
            })
            .collect::<GenResult<_>>()?;

        // super(<parent>, self).__init__(args)
        let mut parent_calls: Vec<Core> = parents
            .iter()
            .map(|(parent, args)| Core::PropertyCall {
                object: Box::from(Core::FunctionCall {
                    function: Box::from(Core::Id { lit: String::from(function::python::SUPER) }),
                    args: vec![
                        Core::Id { lit: parent.clone() },
                        Core::Id { lit: String::from(arg::python::SELF) },
                    ],
                }),
                property: Box::from(Core::FunctionCall {
                    function: Box::new(Core::Id { lit: String::from(function::python::INIT) }),
                    args: args.clone(),
                }),
            })
            .collect();

        // Class arguments not in any parent must be assigned here
        for arg in &args {
            let arg = match arg {
                Core::FunArg { var, .. } => var.clone(),
                _ => return Err(UnimplementedErr::new(ty, "Class argument not in parent")),
            };

            if !parents.iter().any(|(_, args)| args.iter().any(|a| Box::from(a.clone()) == arg)) {
                parent_calls.push(Core::Assign {
                    left: Box::from(Core::PropertyCall {
                        object: Box::from(Core::Id { lit: String::from(arg::python::SELF) }),
                        property: arg.clone(),
                    }),
                    right: arg,
                    op: String::from("="),
                })
            }
        }

        let mut new_args = vec![Core::FunArg {
            vararg: false,
            var: Box::from(Core::Id { lit: String::from(arg::python::SELF) }),
            ty: None,
            default: None,
        }];
        new_args.append(&mut args);

        let mut statements = old_stmts;
        statements.append(&mut vec![Core::FunDef {
            id: Box::from(Core::Id { lit: String::from(function::python::INIT) }),
            arg: new_args,
            ty: None,
            body: Box::from(Core::Block { statements: parent_calls }),
        }]);

        Some(Core::Block { statements })
    } else {
        body
    };

    match body {
        Some(body) => {
            let parent_names = parents
                .iter()
                .map(|parent| match parent.clone() {
                    Core::FunctionCall { function, .. } => match *function {
                        Core::Id { .. } => Ok(*function),
                        _ => Err(UnimplementedErr::new(ty, "Parent")),
                    },
                    Core::Type { lit, .. } => Ok(Core::Id { lit }),
                    _ => Err(UnimplementedErr::new(ty, "Parent")),
                })
                .collect::<GenResult<Vec<Core>>>()?;

            Ok(Core::ClassDef { name: Box::from(id), parent_names, body: Box::from(body) })
        }
        None => {
            if parents.is_empty() {
                Ok(Core::ClassDef {
                    name: Box::from(id),
                    parent_names: vec![],
                    body: Box::from(Core::Pass),
                })
            } else if parents.len() == 1 {
                Ok(Core::VarDef {
                    var: Box::from(id),
                    ty: None,
                    expr: parents.first().cloned().map(Box::from),
                })
            } else {
                Err(UnimplementedErr::new(ty, "More than one parent"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use crate::check::context::function;
    use crate::common::position::Position;
    use crate::generate::ast::node::Core;
    use crate::generate::gen;
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

        let core_import = match gen(&import) {
            Ok(Core::Import { imports }) => imports,
            other => panic!("Expected tuple but got {:?}", other),
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

        let (core_import, core_as) = match gen(&import) {
            Ok(Core::ImportAs { imports, aliases }) => (imports, aliases),
            other => panic!("Expected import but got {:?}", other),
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
        let import = to_pos!(Node::FromImport {
            id,
            import: to_pos!(Node::Import { import, aliases: vec![] })
        });

        let (from, import) = match gen(&import) {
            Ok(Core::FromImport { from, import }) => match &import.deref() {
                Core::Import { imports } => (from.clone(), imports.clone()),
                other => panic!("Expected import but got {:?}", other),
            },
            other => panic!("Expected from import but got {:?}", other),
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

        let result = gen(&condition);
        assert!(result.is_err());
    }

    #[test]
    fn type_alias() {
        let alias = to_pos!(Node::Class {
            ty: to_pos!(Node::Type {
                id: to_pos!(Node::Id { lit: String::from("MyErr1") }),
                generics: vec![]
            }),
            args: vec![],
            parents: vec![to_pos_unboxed!(Node::Parent {
                ty: to_pos!(Node::Type {
                    id: to_pos!(Node::Id { lit: String::from("Exception") }),
                    generics: vec![]
                }),
                args: vec![to_pos_unboxed!(Node::Str {
                    lit: String::from("Something went wrong"),
                    expressions: vec![]
                })]
            })],
            body: None
        });

        let (var, ty, expr) = match gen(&alias) {
            Ok(Core::VarDef { var, ty, expr }) => (*var.clone(), ty.clone(), expr.clone()),
            other => panic!("Expected type alias but got {:?}", other),
        };

        assert_eq!(var, Core::Id { lit: String::from("MyErr1") });
        assert_eq!(ty, None);
        assert!(expr.is_some());
        match expr.clone().unwrap().deref() {
            Core::FunctionCall { function, args } => {
                assert_eq!(*function.deref(), Core::Id { lit: String::from("Exception") });
                assert_eq!(args.len(), 1);
                assert_eq!(
                    *args.first().unwrap(),
                    Core::Str { string: String::from("Something went wrong") }
                )
            }
            _ => panic!("Expected function call, was {:?}", expr.clone()),
        }
    }

    #[test]
    fn type_alias_with_arguments() {
        let alias = to_pos!(Node::Class {
            ty: to_pos!(Node::Type {
                id: to_pos!(Node::Id { lit: String::from("MyErr1") }),
                generics: vec![]
            }),
            args: vec![to_pos_unboxed!(Node::FunArg {
                vararg: false,
                mutable: false,
                var: to_pos!(Node::Id { lit: String::from("a1") }),
                ty: None,
                default: None
            })],
            parents: vec![to_pos_unboxed!(Node::Parent {
                ty: to_pos!(Node::Type {
                    id: to_pos!(Node::Id { lit: String::from("Exception") }),
                    generics: vec![]
                }),
                args: vec![to_pos_unboxed!(Node::Id { lit: String::from("a1") })]
            })],
            body: None
        });

        let (name, parent_names, body) = match gen(&alias) {
            Ok(Core::ClassDef { name, parent_names, body }) => (*name, parent_names, *body),
            other => panic!("Expected class def but got {:?}", other),
        };

        assert_eq!(name, Core::Id { lit: String::from("MyErr1") });
        assert_eq!(parent_names.len(), 1);
        assert_eq!(*parent_names.first().unwrap(), Core::Id { lit: String::from("Exception") });

        if let Core::Block { statements } = body {
            assert_eq!(statements.len(), 1);
            let statement = statements.first().unwrap();

            assert_eq!(
                *statement,
                Core::FunDef {
                    id: Box::new(Core::Id { lit: String::from("__init__") }),
                    arg: vec![
                        Core::FunArg {
                            vararg: false,
                            var: Box::from(Core::Id { lit: String::from("self") }),
                            ty: None,
                            default: None,
                        },
                        Core::FunArg {
                            vararg: false,
                            var: Box::from(Core::Id { lit: String::from("a1") }),
                            ty: None,
                            default: None,
                        },
                    ],
                    ty: None,
                    body: Box::from(Core::Block {
                        statements: vec![Core::PropertyCall {
                            object: Box::from(Core::FunctionCall {
                                function: Box::from(Core::Id { lit: String::from("super") }),
                                args: vec![
                                    Core::Id { lit: String::from("Exception") },
                                    Core::Id { lit: String::from("self") },
                                ],
                            }),
                            property: Box::from(Core::FunctionCall {
                                function: Box::from(Core::Id { lit: String::from(function::python::INIT) }),
                                args: vec![Core::Id { lit: String::from("a1") }],
                            }),
                        }]
                    }),
                }
            )
        } else {
            assert_eq!(body, Core::Block { statements: vec![] });
        }
    }
}

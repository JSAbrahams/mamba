use std::ops::Deref;

use crate::check::context::{arg, function};
use crate::check::context::function::python;
use crate::core::construct::Core;
use crate::core::construct::Core::FunctionCall;
use crate::desugar::common::desugar_vec;
use crate::desugar::desugar;
use crate::desugar::node::desugar_node;
use crate::desugar::result::{DesugarResult, UnimplementedErr};
use crate::desugar::state::Imports;
use crate::desugar::state::State;
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

        Node::Parent { ty, args } => Ok(Core::FunctionCall {
            function: Box::from(match desugar_node(ty, imp, state)? {
                Core::Type { lit, .. } => Core::Id { lit }, // ignore generics
                other => other
            }),
            args: desugar_vec(args, imp, state)?,
        }),

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
    let id = match &ty.node {
        Node::Type { id, .. } => desugar_node(id, imp, state)?,
        _ => return Err(UnimplementedErr::new(ty, "Other than type as class identifier"))
    };

    let args = desugar_vec(args, imp, state)?;
    let parents = desugar_vec(parents, imp, state)?;

    let body = match body {
        Some(body) => Some(desugar_node(body, imp, state)?),
        _ => None
    };

    // inline arguments are special
    let body = if args.len() > 0 {
        let mut old_stmts = match body {
            Some(Core::Block { statements }) => statements.clone(),
            None => vec![],
            Some(..) => return Err(UnimplementedErr::new(ty, "Body not block")),
        };

        for stmt in old_stmts.clone() {
            match stmt {
                Core::FunDef { id, .. } => match *id {
                    Core::Id { lit } if lit == python::INIT => {
                        return Err(UnimplementedErr::new(ty, "Inline and in-body constructor"));
                    }
                    _ => {}
                }
                _ => {}
            }
        }

        let parent_args: Vec<(String, Vec<Core>)> = parents.iter().map(|parent| match parent.clone() {
            Core::FunctionCall { function, args } => match *function.clone() {
                Core::Id { lit } => Ok((lit, args)),
                _ => Err(UnimplementedErr::new(ty, "Parent"))
            }
            _ => Err(UnimplementedErr::new(ty, "Parent"))
        }).collect::<DesugarResult<_>>()?;

        // super(<parent>, self).__init__(args)
        let mut statements: Vec<Core> = parent_args.iter().map(|(parent, args)| Core::PropertyCall {
            object: Box::from(Core::FunctionCall {
                function: Box::from(Core::Id { lit: String::from(function::python::SUPER) }),
                args: vec![
                    Core::Id { lit: parent.clone() },
                    Core::Id { lit: String::from(arg::python::SELF) },
                ],
            }),
            property: Box::from(FunctionCall {
                function: Box::new(Core::Id { lit: String::from(python::INIT) }),
                args: args.clone(),
            }),
        }).collect();
        statements.append(&mut old_stmts);

        Some(Core::Block { statements })
    } else {
        body
    };

    match body {
        Some(body) => Ok(Core::ClassDef { name: Box::from(id), parents, body: Box::from(body) }),
        None => if parents.len() == 0 {
            Ok(Core::ClassDef { name: Box::from(id), parents, body: Box::from(Core::Pass) })
        } else if parents.len() == 1 {
            Ok(Core::VarDef { var: Box::from(id), ty: None, expr: parents.first().cloned().map(Box::from) })
        } else {
            let expr = Core::Tuple { elements: parents };
            Ok(Core::VarDef { var: Box::from(id), ty: None, expr: Some(Box::from(expr)) })
        }
    }
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
            Ok(Core::ClassDef { name, parents, body: definitions }) => (name, parents, definitions),
            other => panic!("Expected class def but got {:?}", other)
        };
    }
}

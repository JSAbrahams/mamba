use std::collections::HashMap;
use std::ops::Deref;

use itertools::Itertools;

use crate::{ASTTy, Context};
use crate::check::ast::NodeTy;
use crate::check::context::{arg, function, LookupClass};
use crate::check::context::clss::Class;
use crate::check::name::string_name::StringName;
use crate::common::position::Position;
use crate::generate::ast::node::{Core, CoreOp};
use crate::generate::convert::common::convert_vec;
use crate::generate::convert::convert_node;
use crate::generate::convert::state::{Imports, State};
use crate::generate::name::ToPy;
use crate::generate::result::{GenResult, UnimplementedErr};

/// Desugar a class.
///
/// If a class has inline arguments (arguments next to class), then we create a
/// constructor and assume that there is no constructor in the body of a class.
/// This property should be ensured by the type checker.
///
/// We add arguments and calls to super for parents.
pub fn convert_class(ast: &ASTTy, imp: &mut Imports, state: &State, ctx: &Context) -> GenResult {
    match &ast.node {
        NodeTy::TypeAlias { ty, isa, .. } => {
            imp.add_from_import("typing", "NewType");
            let lit = ty.name.clone();

            Ok(Core::Assign {
                left: Box::new(Core::Id { lit: lit.clone() }),
                right: Box::new(Core::FunctionCall {
                    function: Box::new(Core::Id { lit: String::from("NewType") }),
                    args: vec![Core::Str { string: lit }, isa.to_py(imp)],
                }),
                op: CoreOp::Assign,
            })
        }
        NodeTy::TypeDef { ty, body, isa } => {
            let parents = isa.as_ref().map_or_else(Vec::new, |isa| vec![isa.to_py(imp)]);
            extract_class(ty, body, &[], &parents, imp, &state.in_interface(true), ctx)
        }
        NodeTy::Class { ty, body, args, parents } => {
            let parents = convert_vec(parents, imp, state, ctx)?;
            extract_class(ty, body, args, &parents, imp, &state.in_interface(false), ctx)
        }

        NodeTy::Parent { ty, args } if args.is_empty() => Ok(ty.to_py(imp)),
        NodeTy::Parent { ty, args } => Ok(Core::FunctionCall {
            function: Box::from(ty.to_py(imp)),
            args: convert_vec(args, imp, state, ctx)?,
        }),

        other => {
            let msg = format!("Expected class or type definition, was {other:?}");
            Err(Box::from(UnimplementedErr::new(ast, &msg)))
        }
    }
}

/// Extract class.
///
/// Construct custom constructor to call parents if:
/// - There are class arguments
/// - There are multiple parents
/// - The class has a body and one or more parents has class arguments
///
/// If creating a new constructor, it is inserted after the last found variable.
fn extract_class(
    ty: &StringName,
    body: &Option<Box<ASTTy>>,
    args: &[ASTTy],
    parents: &[Core],
    imp: &mut Imports,
    state: &State,
    ctx: &Context,
) -> GenResult {
    let body = body.clone().map(|body| convert_node(body.deref(), imp, state, ctx));
    let body = if let Some(body) = body { Some(body?) } else { None };
    let mut body_name_stmts: HashMap<Core, (usize, Core)> =
        match body {
            Some(Core::Block { statements }) => statements,
            Some(other) => vec![other],
            None => vec![],
        }.iter().enumerate().map(|(i, stmt)| {
            // function two further to leave place for init
            let (pos, key) = match stmt {
                Core::FunDef { id, .. } => (i + 2, Core::Id { lit: id.clone() }),
                Core::FunDefOp { op, .. } => (i + 2, Core::Id { lit: format!("{op}") }),
                Core::VarDef { var, .. } => (i, var.deref().clone()),
                _ => (i, Core::Id { lit: String::from("@") }),
            };
            (key, (pos, stmt.clone()))
        }).collect();

    let args = convert_vec(args, imp, &state.def_as_fun_arg(true), ctx)?;

    let old_init = body_name_stmts
        .iter()
        .find(|(name, _)| matches!(name, Core::Id { lit } if *lit == function::python::INIT))
        .map(|(_, (_, function))| function);
    if let Some(new_init) = init(&old_init, &args, parents)? {
        let init = Core::Id { lit: String::from(function::python::INIT) };
        let pos = if let Some((pos, _)) = body_name_stmts.get(&init) {
            *pos // leave pos untouched
        } else {
            body_name_stmts
                .values()
                .filter(|(_, stmt)| matches!(stmt, Core::VarDef { .. }))
                .map(|(pos, _)| *pos + 1)
                .max()
                .unwrap_or(0) // otherwise always first
        };

        body_name_stmts.insert(init, (pos, new_init));
    }

    let parent_names = parents
        .iter()
        .map(|parent| match parent.clone() {
            Core::FunctionCall { function, .. } => match *function {
                Core::Type { lit, .. } => Ok(Core::Id { lit }),
                other => panic!("Expected type in parent, was {}", other)
            },
            Core::Type { .. } => Ok(parent.clone()),
            other => panic!("Expected type in parent, was {}", other)
        })
        .collect::<GenResult<Vec<Core>>>()?;

    let class = ctx.class(ty, Position::invisible()).ok();

    let parent_names = if state.interface && !has_abstract_parent(&class, ctx) {
        imp.add_from_import("abc", "ABC");
        parent_names.into_iter().chain(vec![Core::Id { lit: String::from("ABC") }]).collect()
    } else {
        parent_names
    };

    let body_stmts: Vec<Core> = body_name_stmts
        .values()
        .into_iter()
        .sorted_by_key(|(pos, _)| *pos)
        .map(|(_, stmt)| stmt.clone())
        .collect();

    let statements = if body_stmts.is_empty() { vec![Core::Pass] } else { body_stmts };
    let body = Core::Block { statements };

    if let Core::Type { lit, .. } = ty.to_py(imp) {
        let name = Box::from(Core::Id { lit });
        Ok(Core::ClassDef { name, parent_names, body: Box::from(body) })
    } else {
        panic!("class name should be type")
    }
}

fn has_abstract_parent(clss: &Option<Class>, ctx: &Context) -> bool {
    if let Some(clss) = clss {
        clss.parents.iter().any(|parent| {
            let clss = ctx.class(parent, Position::invisible()).ok();
            is_abstract(&clss, ctx)
        })
    } else {
        false
    }
}

fn is_abstract(clss: &Option<Class>, ctx: &Context) -> bool {
    if let Some(clss) = clss {
        !clss.concrete || clss.parents.iter().any(|parent| {
            let clss = ctx.class(parent, Position::invisible()).ok();
            has_abstract_parent(&clss, ctx)
        })
    } else {
        false
    }
}

fn init(
    old_init: &Option<&Core>,
    class_args: &[Core],
    parents: &[Core],
) -> GenResult<Option<Core>> {
    let (parent_inits, parent_args): (Vec<Core>, Vec<Vec<Core>>) = parents
        .iter()
        .map(|parent| {
            let (lit, mut arg) = match parent {
                Core::FunctionCall { function, args } => match function.deref() {
                    Core::Type { lit, .. } => (lit.clone(), args.clone()),
                    _ => (String::from(""), args.clone()),
                },
                Core::Type { lit, .. } => (lit.clone(), vec![]),
                _ => (String::from(""), vec![]),
            };

            let mut args = vec![Core::Id { lit: String::from(arg::python::SELF) }];
            args.append(&mut arg);

            (
                Core::PropertyCall {
                    object: Box::from(Core::Id { lit }),
                    property: Box::new(Core::FunctionCall {
                        function: Box::new(Core::Id { lit: String::from(function::python::INIT) }),
                        args: args.clone(),
                    }),
                },
                args,
            )
        })
        .unzip();

    // Parent calls from parents
    let (mut args, mut statements) = if let Some(old_init) = old_init {
        let (mut old_stmts, args) = match old_init {
            Core::FunDef { body, arg, .. } => match body.deref() {
                Core::Block { statements } => (statements.clone(), arg.clone()),
                other => (vec![other.clone()], arg.clone()),
            },
            _ => (vec![], vec![]),
        };

        let mut new_stmts = parent_inits;
        new_stmts.append(&mut old_stmts);
        (args, new_stmts)
    } else {
        (Vec::from(class_args), parent_inits)
    };

    // Assignments from class args not given to parent
    statements.append(
        &mut class_args
            .iter()
            .flat_map(|arg| match arg {
                Core::FunArg { var, .. } => Some(var.deref().clone()),
                _ => None,
            })
            .filter(|arg| !parent_args.iter().any(|p_args| p_args.iter().any(|p_arg| p_arg == arg)))
            .map(|var| Core::Assign {
                left: Box::from(Core::PropertyCall {
                    object: Box::from(Core::Id { lit: String::from(arg::python::SELF) }),
                    property: Box::from(var.clone()),
                }),
                right: Box::from(var),
                op: CoreOp::Assign,
            })
            .collect(),
    );

    let first_is_self = args
        .first()
        .map(|arg| match arg {
            Core::FunArg { var, .. } => {
                if let Core::Id { lit } = var.deref() {
                    lit == arg::python::SELF
                } else {
                    false
                }
            }
            _ => false,
        })
        .unwrap_or(false);
    let args = if first_is_self {
        args
    } else {
        let mut new_args = vec![Core::Id { lit: String::from(arg::python::SELF) }];
        new_args.append(&mut args);
        new_args
    };

    let id = String::from(function::python::INIT);
    Ok(if !statements.is_empty() {
        let dec = vec![];
        Some(Core::FunDef { dec, id, arg: args, ty: None, body: Box::new(Core::Block { statements }) })
    } else {
        None
    })
}

#[cfg(test)]
mod tests {
    use crate::ASTTy;
    use crate::common::position::Position;
    use crate::generate::ast::node::Core;
    use crate::generate::gen;
    use crate::parse::ast::{AST, Node};

    macro_rules! to_pos_unboxed {
        ($node:expr) => {{
            AST { pos: Position::invisible(), node: $node }
        }};
    }

    macro_rules! to_pos {
        ($node:expr) => {{
            Box::from(to_pos_unboxed!($node))
        }};
    }

    #[test]
    fn import_verify() {
        let from = Some(to_pos!(Node::Break));
        let import = vec![
            to_pos_unboxed!(Node::ENum { num: String::from("a"), exp: String::from("100") }),
            to_pos_unboxed!(Node::Real { lit: String::from("3000.5") }),
        ];
        let alias = vec![];
        let import = to_pos!(Node::Import { from, import, alias });

        let (from, import, alias) = match gen(&ASTTy::from(&*import)) {
            Ok(Core::Import { from, import, alias }) => {
                (from.clone(), import.clone(), alias.clone())
            }
            other => panic!("Expected import but got {:?}", other),
        };

        assert_eq!(*from.unwrap(), Core::Break);
        assert_eq!(import[0], Core::ENum { num: String::from("a"), exp: String::from("100") });
        assert_eq!(import[1], Core::Float { float: String::from("3000.5") });
        assert!(alias.is_empty());
    }

    #[test]
    fn condition_verify() {
        let cond = to_pos!(Node::Bool { lit: true });
        let condition = to_pos!(Node::Condition { cond, el: None });

        let result = gen(&ASTTy::from(&condition));
        assert!(result.is_err());
    }
}

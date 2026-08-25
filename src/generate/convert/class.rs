use std::collections::{HashMap, HashSet};
use std::ops::Deref;

use itertools::Itertools;

use crate::check::ast::NodeTy;
use crate::check::context::clss::Class;
use crate::check::context::{arg, function, LookupClass};
use crate::check::name::string_name::StringName;
use crate::common::position::Position;
use crate::generate::ast::node::{Core, CoreOp};
use crate::generate::convert::common::convert_vec;
use crate::generate::convert::convert_node;
use crate::generate::convert::state::{Imports, State};
use crate::generate::name::ToPy;
use crate::generate::result::{GenResult, UnimplementedErr};
use crate::{ASTTy, Context};

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
                    function: Box::new(Core::Id {
                        lit: String::from("NewType"),
                    }),
                    args: vec![Core::Str { string: lit }, isa.to_py(imp)],
                }),
                op: CoreOp::Assign,
            })
        }
        NodeTy::TypeDef { ty, body, isa } | NodeTy::Trait { ty, body, isa } => {
            let parents = isa
                .as_ref()
                .map_or_else(Vec::new, |isa| vec![isa.to_py(imp)]);
            extract_class(ty, body, &[], &parents, imp, &state.in_interface(true), ctx)
        }
        NodeTy::Class {
            ty,
            body,
            args,
            parents,
        } => {
            let parents = convert_vec(parents, imp, state, ctx)?;
            extract_class(
                ty,
                body,
                args,
                &parents,
                imp,
                &state.in_interface(false),
                ctx,
            )
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
    let body = body
        .clone()
        .map(|body| convert_node(body.deref(), imp, state, ctx));
    let body = if let Some(body) = body {
        Some(body?)
    } else {
        None
    };
    let mut body_name_stmts: HashMap<Core, (usize, Core)> = match body {
        Some(Core::Block { statements }) => statements,
        Some(other) => vec![other],
        None => vec![],
    }
    .iter()
    .enumerate()
    .map(|(i, stmt)| {
        // function two further to leave place for init
        let (pos, key) = match stmt {
            Core::FunDef { id, .. } => (i + 2, Core::Id { lit: id.clone() }),
            Core::FunDefOp { op, .. } => (
                i + 2,
                Core::Id {
                    lit: format!("{op}"),
                },
            ),
            Core::VarDef { var, .. } => (i, var.deref().clone()),
            _ => (
                i,
                Core::Id {
                    lit: String::from("@"),
                },
            ),
        };
        (key, (pos, stmt.clone()))
    })
    .collect();

    let args = convert_vec(args, imp, &state.def_as_fun_arg(true), ctx)?;
    let class = ctx.class(ty, Position::invisible()).ok();

    // `self` is only in scope inside a method, so any class-body statement referencing it
    // must move into `__init__`.
    let self_name: HashSet<String> = HashSet::from([String::from(arg::python::SELF)]);
    let hoisted = hoist_constructor_dependent_stmts(&mut body_name_stmts, &self_name);

    let old_init = body_name_stmts
        .iter()
        .find(|(name, _)| matches!(name, Core::Id { lit } if *lit == function::python::INIT))
        .map(|(_, (_, function))| function);
    if let Some(new_init) = init(&old_init, &args, parents, hoisted)? {
        let init = Core::Id {
            lit: String::from(function::python::INIT),
        };
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
                other => panic!("Expected type in parent, was {other}"),
            },
            Core::Type { .. } => Ok(parent.clone()),
            other => panic!("Expected type in parent, was {other}"),
        })
        .collect::<GenResult<Vec<Core>>>()?;

    let parent_names = if state.interface && !has_abstract_parent(&class, ctx) {
        imp.add_from_import("abc", "ABC");
        parent_names
            .into_iter()
            .chain(vec![Core::Id {
                lit: String::from("ABC"),
            }])
            .collect()
    } else {
        parent_names
    };

    let body_stmts: Vec<Core> = body_name_stmts
        .values()
        .sorted_by_key(|(pos, _)| *pos)
        .map(|(_, stmt)| stmt.clone())
        .collect();

    let statements = if body_stmts.is_empty() {
        vec![Core::Pass]
    } else {
        body_stmts
    };
    let body = Core::Block { statements };

    if let Core::Type { lit, .. } = ty.to_py(imp) {
        let name = Box::from(Core::Id { lit });
        Ok(Core::ClassDef {
            name,
            parent_names,
            body: Box::from(body),
        })
    } else {
        panic!("class name should be type")
    }
}

/// Move class-body statements referencing `self` into `__init__`, ordered by dependency
/// (see `order_by_self_field_deps`).
///
/// A field declaration keeps its class-level slot with `None` in place of the initializer;
/// any other statement (e.g. a bare `print(self.a)`) is moved wholesale.
fn hoist_constructor_dependent_stmts(
    body_name_stmts: &mut HashMap<Core, (usize, Core)>,
    self_name: &HashSet<String>,
) -> Vec<Core> {
    let mut hoisted: Vec<(usize, Core)> = vec![];
    let mut to_remove = vec![];

    for (key, (pos, stmt)) in body_name_stmts.iter_mut() {
        match stmt {
            Core::VarDef {
                var,
                expr: Some(expr),
                ..
            } if references_free_var(expr, self_name) => {
                hoisted.push((
                    *pos,
                    Core::Assign {
                        left: Box::from(Core::PropertyCall {
                            object: Box::from(Core::Id {
                                lit: String::from(arg::python::SELF),
                            }),
                            property: var.clone(),
                        }),
                        right: expr.clone(),
                        op: CoreOp::Assign,
                    },
                ));
                *expr = Box::from(Core::None);
            }
            // A docstring must stay a literal first statement in the class body, not move into
            // `__init__`.
            Core::FunDef { .. }
            | Core::FunDefOp { .. }
            | Core::VarDef { .. }
            | Core::DocStr { .. } => {}
            // Any other class-body statement (e.g. a bare `print(...)`) runs once per instance,
            // like the rest of the constructor — not once at class-definition time — so it
            // always moves into `__init__`, whether or not it happens to reference `self`.
            other => {
                hoisted.push((*pos, other.clone()));
                to_remove.push(key.clone());
            }
        }
    }

    for key in to_remove {
        body_name_stmts.remove(&key);
    }

    order_by_self_field_deps(hoisted)
}

/// Order hoisted statements so a field's assignment comes after any other hoisted field it reads via `self.<field>`.
/// Declaration order alone isn't enough: a field may read another hoisted field declared later in the class body.
/// This could still be `None` at that point.
///
/// A statement referencing its *own* field is exempt, since that reads the constructor-arg auto-assignment, not a hoisted default.
fn order_by_self_field_deps(mut hoisted: Vec<(usize, Core)>) -> Vec<Core> {
    hoisted.sort_by_key(|(pos, _)| *pos);

    let names: Vec<Option<String>> = hoisted
        .iter()
        .map(|(_, stmt)| assigned_self_field(stmt))
        .collect();
    let deps: Vec<Vec<usize>> = hoisted
        .iter()
        .enumerate()
        .map(|(i, (_, stmt))| {
            names
                .iter()
                .enumerate()
                .filter(|(j, name)| {
                    *j != i
                        && name
                            .as_deref()
                            .is_some_and(|field| references_self_field(stmt, field))
                })
                .map(|(j, _)| j)
                .collect()
        })
        .collect();

    let n = hoisted.len();
    let mut order = vec![];
    let mut emitted = vec![false; n];
    while order.len() < n {
        let before = order.len();
        for i in 0..n {
            if !emitted[i] && deps[i].iter().all(|&d| emitted[d]) {
                order.push(i);
                emitted[i] = true;
            }
        }
        if order.len() == before {
            // cycle: emit whatever's left in declaration order rather than looping forever
            for (i, emitted) in emitted.iter_mut().enumerate() {
                if !*emitted {
                    order.push(i);
                    *emitted = true;
                }
            }
        }
    }

    order.into_iter().map(|i| hoisted[i].1.clone()).collect()
}

/// The field name of a hoisted `self.<field> = ...` assignment, if `stmt` is one.
fn assigned_self_field(stmt: &Core) -> Option<String> {
    match stmt {
        Core::Assign { left, .. } => match left.as_ref() {
            Core::PropertyCall { object, property } => match (object.as_ref(), property.as_ref()) {
                (Core::Id { lit: obj }, Core::Id { lit: prop }) if obj == arg::python::SELF => {
                    Some(prop.clone())
                }
                _ => None,
            },
            _ => None,
        },
        _ => None,
    }
}

/// Applies `test` to `core` and every sub-expression, depth-first. Not exhaustive over every
/// `Core` variant; a missed variant just means a match goes undetected.
fn any_node(core: &Core, test: &impl Fn(&Core) -> bool) -> bool {
    if test(core) {
        return true;
    }
    match core {
        Core::PropertyCall { object, .. } => any_node(object, test),
        Core::FunctionCall { function, args } => {
            any_node(function, test) || args.iter().any(|a| any_node(a, test))
        }
        Core::Index { item, range } => any_node(item, test) || any_node(range, test),
        Core::KeyValue { key, value } => any_node(key, test) || any_node(value, test),
        Core::Ge { left, right }
        | Core::Geq { left, right }
        | Core::Le { left, right }
        | Core::Leq { left, right }
        | Core::Is { left, right }
        | Core::IsN { left, right }
        | Core::Eq { left, right }
        | Core::Neq { left, right }
        | Core::IsA { left, right }
        | Core::And { left, right }
        | Core::Or { left, right }
        | Core::Add { left, right }
        | Core::Sub { left, right }
        | Core::Mul { left, right }
        | Core::Mod { left, right }
        | Core::Pow { left, right }
        | Core::Div { left, right }
        | Core::FDiv { left, right }
        | Core::In { left, right } => any_node(left, test) || any_node(right, test),
        Core::Not { expr }
        | Core::AddU { expr }
        | Core::SubU { expr }
        | Core::Sqrt { expr }
        | Core::Return { expr }
        | Core::Raise { error: expr } => any_node(expr, test),
        Core::If { cond, then } => any_node(cond, test) || any_node(then, test),
        Core::IfElse { cond, then, el } | Core::Ternary { cond, then, el } => {
            any_node(cond, test) || any_node(then, test) || any_node(el, test)
        }
        Core::Tuple { elements }
        | Core::TupleLiteral { elements }
        | Core::Set { elements }
        | Core::List { elements } => elements.iter().any(|e| any_node(e, test)),
        Core::Dictionary { elements } => elements
            .iter()
            .any(|(k, v)| any_node(k, test) || any_node(v, test)),
        Core::Assign { right, .. } => any_node(right, test),
        Core::VarDef {
            expr: Some(expr), ..
        } => any_node(expr, test),
        Core::Block { statements } => statements.iter().any(|s| any_node(s, test)),
        _ => false,
    }
}

/// Whether `core` contains a free reference to any name in `names`.
fn references_free_var(core: &Core, names: &HashSet<String>) -> bool {
    any_node(
        core,
        &|c| matches!(c, Core::Id { lit } if names.contains(lit)),
    )
}

/// Whether `core` reads `self.<field>` anywhere.
fn references_self_field(core: &Core, field: &str) -> bool {
    any_node(core, &|c| match c {
        Core::PropertyCall { object, property } => {
            matches!(object.as_ref(), Core::Id { lit } if lit == arg::python::SELF)
                && matches!(property.as_ref(), Core::Id { lit } if lit == field)
        }
        _ => false,
    })
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
        !clss.concrete
            || clss.parents.iter().any(|parent| {
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
    mut extra_statements: Vec<Core>,
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

            let mut args = vec![Core::Id {
                lit: String::from(arg::python::SELF),
            }];
            args.append(&mut arg);

            (
                Core::PropertyCall {
                    object: Box::from(Core::Id { lit }),
                    property: Box::new(Core::FunctionCall {
                        function: Box::new(Core::Id {
                            lit: String::from(function::python::INIT),
                        }),
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
            .filter(|arg| {
                !parent_args
                    .iter()
                    .any(|p_args| p_args.iter().any(|p_arg| p_arg == arg))
            })
            .map(|var| Core::Assign {
                left: Box::from(Core::PropertyCall {
                    object: Box::from(Core::Id {
                        lit: String::from(arg::python::SELF),
                    }),
                    property: Box::from(var.clone()),
                }),
                right: Box::from(var),
                op: CoreOp::Assign,
            })
            .collect(),
    );
    statements.append(&mut extra_statements);

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
        let mut new_args = vec![Core::Id {
            lit: String::from(arg::python::SELF),
        }];
        new_args.append(&mut args);
        new_args
    };

    let id = String::from(function::python::INIT);
    Ok(if !statements.is_empty() {
        let dec = vec![];
        Some(Core::FunDef {
            dec,
            id,
            arg: args,
            ty: None,
            body: Box::new(Core::Block { statements }),
        })
    } else {
        None
    })
}

#[cfg(test)]
mod tests {
    use crate::common::position::Position;
    use crate::generate::ast::node::Core;
    use crate::generate::gen;
    use crate::parse::ast::{Node, AST};
    use crate::ASTTy;

    macro_rules! to_pos_unboxed {
        ($node:expr) => {{
            AST {
                pos: Position::invisible(),
                node: $node,
            }
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
            to_pos_unboxed!(Node::ENum {
                num: String::from("a"),
                exp: String::from("100")
            }),
            to_pos_unboxed!(Node::Real {
                lit: String::from("3000.5")
            }),
        ];
        let alias = vec![];
        let import = to_pos!(Node::Import {
            from,
            import,
            alias
        });

        let (from, import, alias) = match gen(&ASTTy::from(&*import)) {
            Ok(Core::Import {
                from,
                import,
                alias,
            }) => (from.clone(), import.clone(), alias.clone()),
            other => panic!("Expected import but got {other:?}"),
        };

        assert_eq!(*from.unwrap(), Core::Break);
        assert_eq!(
            import[0],
            Core::ENum {
                num: String::from("a"),
                exp: String::from("100")
            }
        );
        assert_eq!(
            import[1],
            Core::Float {
                float: String::from("3000.5")
            }
        );
        assert!(alias.is_empty());
    }

    #[test]
    fn condition_verify() {
        let cond = to_pos!(Node::Id {
            lit: "True".to_string()
        });
        let condition = to_pos!(Node::Condition { cond, el: None });

        let result = gen(&ASTTy::from(&condition));
        assert!(result.is_err());
    }
}

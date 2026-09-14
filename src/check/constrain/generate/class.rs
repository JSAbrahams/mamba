use std::convert::TryFrom;

use log::warn;

use crate::check::constrain::constraint::builder::ConstrBuilder;
use crate::check::constrain::generate::definition::id_from_var;
use crate::check::constrain::generate::env::Environment;
use crate::check::constrain::generate::{gen_vec, generate, Constrained};
use crate::check::context::arg::python::SELF;
use crate::check::context::function::python::INIT;
use crate::check::context::Context;
use crate::check::name::string_name::StringName;
use crate::check::name::Name;
use crate::check::result::TypeErr;
use crate::check::{is_new_marker, is_pure_new, NEW};
use crate::parse::ast::Node::Id;
use crate::parse::ast::{Node, AST};

pub fn gen_class(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder,
) -> Constrained {
    match &ast.node {
        Node::Class {
            body: Some(body),
            ty,
            args,
            ..
        } => match &body.node {
            Node::Block { statements } => {
                // Bind self so field initializers can reach class arguments via `self.<name>`.
                let name = Some(Name::from(&StringName::try_from(ty)?));
                let var = AST::new(
                    ty.pos,
                    Id {
                        lit: String::from(SELF),
                    },
                );
                let env = id_from_var(&var, &name, &None, true, ctx, constr, env)?;

                constrain_class_body(statements, ty, args, true, &env, ctx, constr)
            }
            _ => Err(vec![TypeErr::new(body.pos, "Expected code block")]),
        },
        Node::Trait {
            body: Some(body),
            ty,
            ..
        } => match &body.node {
            Node::Block { statements } => {
                constrain_class_body(statements, ty, &[], false, env, ctx, constr)
            }
            _ => Err(vec![TypeErr::new(body.pos, "Expected code block")]),
        },
        Node::Class { .. } | Node::Trait { .. } => Ok(env.clone()),

        Node::Condition { cond, el: Some(el) } => {
            generate(cond, env, ctx, constr)?;
            generate(el, env, ctx, constr)
        }
        Node::Condition { cond, .. } => generate(cond, env, ctx, constr),

        _ => Err(vec![TypeErr::new(
            ast.pos,
            "Expected class or type definition",
        )]),
    }
}

/// A class body may only declare fields and methods.
///
/// `is_class` also requires each field to be assigned. A trait's field is a *requirement* on
/// its implementors rather than a derivation, so it has nothing to assign.
///
/// A bare statement would run once per instance, like a constructor, which leaves it
/// ambiguous whether constructing the class has side effects. Side effects belong in an
/// explicit constructor instead, where they are visible in its signature. A leading
/// docstring is the one exception, since it is documentation rather than a statement.
fn check_only_declarations(
    statements: &[AST],
    class_args: &[AST],
    is_class: bool,
) -> Constrained<()> {
    let mut errors: Vec<TypeErr> = vec![];

    for (i, stmt) in statements.iter().enumerate() {
        match &stmt.node {
            // Mamba has no constructor to override. Construction goes through the class
            // arguments, and `new` is where anything more than that belongs.
            Node::FunDef { id, .. } if matches!(&id.node, Node::Id { lit } if lit == INIT) => {
                let msg = format!(
                    "Mamba has no '{INIT}'. Take the values as class arguments, and declare '{NEW}' if construction needs to do more than that"
                );
                errors.push(TypeErr::new(stmt.pos, &msg));
            }
            // The marker's argument list stands for the class arguments rather than declaring
            // any, so it has to say how many there are.
            Node::FunDef { args, .. } if is_new_marker(stmt) => {
                let (matches_class, list, stands_for) = match args.first().map(|arg| &arg.node) {
                    None => (class_args.is_empty(), "()", "no class arguments"),
                    Some(Node::Underscore) => {
                        (class_args.len() == 1, "(_)", "exactly one class argument")
                    }
                    _ => (
                        !class_args.is_empty(),
                        "(..)",
                        "one or more class arguments",
                    ),
                };

                if !matches_class {
                    let write = match class_args.len() {
                        0 => format!("'{NEW}()'"),
                        1 => format!("'{NEW}(_)' or '{NEW}(..)'"),
                        _ => format!("'{NEW}(..)'"),
                    };
                    let msg = format!(
                        "'{NEW}{list}' stands for {stands_for}, but the class has {}. Write {write}",
                        class_args.len()
                    );
                    errors.push(TypeErr::new(stmt.pos, &msg));
                } else if !is_pure_new(stmt) {
                    warn!(
                        "{}:{} redundant '{NEW}{list}', as one taking the class arguments is already generated. Mark it 'pure' to assert construction is pure, or give it arguments and a body to replace it",
                        stmt.pos.start.line, stmt.pos.start.pos
                    );
                }
            }
            Node::FunDef { .. } => {}
            Node::DocStr { .. } if i == 0 => {}
            Node::VariableDef { ty, expr, .. } => {
                // A field declared in the body is derived: it is computed rather than passed,
                // so it needs something to compute. Without one it would silently hold `None`,
                // whatever its type says. A nullable field is the exception, since `None` is a
                // value it can actually take.
                if is_class && expr.is_none() && !ty.as_ref().is_some_and(|ty| is_nullable(ty)) {
                    let msg = "A field declared in a class body must be assigned a value, as it is derived rather than passed. Make it a class argument to pass it instead";
                    errors.push(TypeErr::new(stmt.pos, msg));
                }
            }
            _ => {
                let msg = format!(
                    "A class body may only declare fields and methods, was {}",
                    stmt.node
                );
                errors.push(TypeErr::new(stmt.pos, &msg));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Whether a type annotation admits `None`, so a field of it needs no value of its own.
fn is_nullable(ty: &AST) -> bool {
    match &ty.node {
        Node::QuestionOp { .. } => true,
        Node::TypeTup { types } => types.iter().all(is_nullable),
        _ => false,
    }
}

pub fn constrain_class_body(
    statements: &[AST],
    ty: &AST,
    class_args: &[AST],
    is_class: bool,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder,
) -> Constrained {
    let name = StringName::try_from(ty)?;
    check_only_declarations(statements, class_args, is_class)?;
    let class_env = env.in_class(&name);

    let pure_new = statements.iter().any(is_pure_new);

    // The marker declares nothing, so there is nothing in it to constrain. Dropping it here
    // also keeps its `..` away from the generator, which rejects one wherever it lands.
    let statements: Vec<AST> = statements
        .iter()
        .filter(|stmt| !is_new_marker(stmt))
        .cloned()
        .collect();
    let statements = statements.as_slice();

    // `def pure new(..)` asserts the generated constructor is pure, and the derived field
    // initializers are the only thing it runs. Checking them under the purity rules is what
    // turns the assertion into a guarantee.
    if pure_new {
        let (fields, rest): (Vec<AST>, Vec<AST>) = statements
            .iter()
            .cloned()
            .partition(|stmt| matches!(stmt.node, Node::VariableDef { .. }));

        gen_vec(&fields, &class_env.in_pure(&class_env), true, ctx, constr)?;
        gen_vec(&rest, &class_env, true, ctx, constr)?;
    } else {
        gen_vec(statements, &class_env, true, ctx, constr)?;
    }

    // preserve mapping of self outside class to prevent contamination
    if let Some(self_map) = constr.var_mapping.get(SELF) {
        Ok(env.override_mapping(SELF, *self_map))
    } else {
        Ok(env.clone())
    }
}

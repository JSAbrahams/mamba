//! Mutability through fields and `mut self`, checked once every receiver has a type.
//!
//! Generating constraints already checks the binding a change starts from.
//! It cannot check the fields a change goes through, or whether a method takes `mut self`, since both need the class of a receiver.

use std::collections::HashSet;
use std::convert::TryFrom;

use crate::check::constrain::constraint::expected::Expect::Type;
use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::unify::finished::Finished;
use crate::check::context::clss::{GetField, GetFun};
use crate::check::context::{Context, LookupClass};
use crate::check::name::string_name::StringName;
use crate::check::name::true_name::TrueName;
use crate::check::name::{ContainsTemp, Empty, Name, Union};
use crate::check::result::TypeErr;
use crate::common::position::Position;
use crate::parse::ast::{Node, AST};

/// A change to a value reached through a chain such as `a.b.c`, from its root `a` onward.
///
/// `root` holds the bindings the root names, as the environment had them, which give it its type.
#[derive(Clone, Debug)]
pub enum Mutation {
    /// An assignment to the last link of `place`, as in `a.b.c := x`.
    Assign {
        place: Vec<AST>,
        root: Vec<Expected>,
        pos: Position,
    },
    /// A call of `method` on `receiver`, which changes it if the method takes `mut self`.
    Call {
        receiver: Vec<AST>,
        root: Vec<Expected>,
        method: StringName,
        pos: Position,
        root_mutable: bool,
    },
}

/// A link of a chain after its root, as far as mutability is concerned.
enum Link<'a> {
    /// A field of `class`, which may be changed only if it is `mut`.
    Field {
        name: &'a str,
        class: TrueName,
        mutable: bool,
    },
    /// The result of a call, which has no binding to declare `mut`.
    Call(&'a AST),
}

/// Report every change that goes through a field that is not `mut`, or through the result of a call.
pub fn check_mutations(
    mutations: &[Mutation],
    finished: &Finished,
    ctx: &Context,
) -> Result<(), Vec<TypeErr>> {
    let mut seen = HashSet::new();
    let errs: Vec<TypeErr> = mutations
        .iter()
        .filter(|mutation| seen.insert(position(mutation)))
        .filter_map(|mutation| check_mutation(mutation, finished, ctx))
        .collect();

    if errs.is_empty() {
        Ok(())
    } else {
        Err(errs)
    }
}

fn position(mutation: &Mutation) -> Position {
    match mutation {
        Mutation::Assign { pos, .. } | Mutation::Call { pos, .. } => *pos,
    }
}

fn check_mutation(mutation: &Mutation, finished: &Finished, ctx: &Context) -> Option<TypeErr> {
    match mutation {
        Mutation::Assign { place, root, pos } => {
            let (links, _) = walk(place, root, finished, ctx)?;
            let action = format!("assign to '{}'", chain_text(place));
            immutable_link(&links, &action, *pos)
        }
        Mutation::Call {
            receiver,
            root,
            method,
            pos,
            root_mutable,
        } => {
            let (links, receiver_ty) = walk(receiver, root, finished, ctx)?;
            if !takes_mut_self(&receiver_ty, method, ctx, *pos) {
                return None;
            }

            let action = format!(
                "call '{method}', which takes 'mut self', on '{}'",
                chain_text(receiver)
            );
            immutable_root(receiver, *root_mutable, &action, *pos)
                .or_else(|| immutable_link(&links, &action, *pos))
        }
    }
}

/// The links of `chain` after its root, and the type it ends in.
///
/// [None] if a class or member on the way is unknown, which the rest of the checker reports.
fn walk<'a>(
    chain: &'a [AST],
    bindings: &[Expected],
    finished: &Finished,
    ctx: &Context,
) -> Option<(Vec<Link<'a>>, Name)> {
    let (root, rest) = chain.split_first()?;
    let mut ty = root_type(root, bindings, finished)?;

    let mut links = vec![];
    for ast in rest {
        let mut next = Name::empty();
        for class in &ty.names {
            let (link, member_ty) = member(class, ast, ctx)?;
            links.push(link);
            next = next.union(&member_ty);
        }
        ty = next;
    }
    Some((links, ty))
}

/// The type of the value a chain starts from.
///
/// A name has the type of the bindings it names.
/// Anything else, such as the result of a call, has the type unification gave its position.
fn root_type(root: &AST, bindings: &[Expected], finished: &Finished) -> Option<Name> {
    if bindings.is_empty() {
        return finished.pos_to_name.get(&root.pos).cloned();
    }
    let types: Vec<Name> = bindings
        .iter()
        .map(|binding| binding_type(binding, finished))
        .collect::<Option<_>>()?;
    Some(types.iter().fold(Name::empty(), |acc, ty| acc.union(ty)))
}

/// The type of a binding: the type it was declared with, or else what unification made of it where it was defined.
fn binding_type(binding: &Expected, finished: &Finished) -> Option<Name> {
    match &binding.expect {
        Type { name } if !name.contains_temp() => Some(name.clone()),
        _ => finished.pos_to_name.get(&binding.pos).cloned(),
    }
}

/// The link `ast` makes on a value of type `class`, and the type of what it reads.
fn member<'a>(class: &TrueName, ast: &'a AST, ctx: &Context) -> Option<(Link<'a>, Name)> {
    let class_def = ctx.class(class, ast.pos).ok()?;
    match &ast.node {
        Node::Id { lit } => {
            let field = class_def.field(lit, ast.pos).ok()?;
            let link = Link::Field {
                name: lit,
                class: class.clone(),
                mutable: field.mutable,
            };
            Some((link, field.ty))
        }
        Node::FunctionCall { name, .. } => {
            let name = StringName::try_from(name).ok()?;
            let fun = class_def.fun(&name, ast.pos).ok()?;
            Some((Link::Call(ast), fun.ret_ty))
        }
        _ => None,
    }
}

/// Whether `method` of any class in `receiver_ty` takes `mut self`.
///
/// A Python stub cannot say whether a method changes its receiver, so only a Mamba method counts.
fn takes_mut_self(receiver_ty: &Name, method: &StringName, ctx: &Context, pos: Position) -> bool {
    receiver_ty.names.iter().any(|class| {
        let fun = ctx
            .class(class, pos)
            .and_then(|class| class.fun(method, pos));
        fun.is_ok_and(|fun| !fun.is_py_type && fun.self_mutable == Some(true))
    })
}

/// An error if the binding `chain` starts from may not be changed.
fn immutable_root(
    chain: &[AST],
    root_mutable: bool,
    action: &str,
    pos: Position,
) -> Option<TypeErr> {
    if root_mutable {
        return None;
    }
    let root = chain.first()?;
    let msg = match &root.node {
        Node::Id { lit } => format!("Cannot {action}, as '{lit}' was not declared 'mut'"),
        _ => format!(
            "Cannot {action}, as '{}' has no binding to declare 'mut'",
            root.node
        ),
    };
    Some(TypeErr::new(pos, &msg))
}

/// An error for the first link in `links` that may not be changed.
fn immutable_link(links: &[Link], action: &str, pos: Position) -> Option<TypeErr> {
    let msg = links.iter().find_map(|link| match link {
        Link::Field { mutable: true, .. } => None,
        Link::Field { name, class, .. } => Some(format!(
            "Cannot {action}, as field '{name}' of '{class}' was not declared 'mut'"
        )),
        Link::Call(call) => Some(format!(
            "Cannot {action}, as '{}' gives a value that has no binding to declare 'mut'",
            call.node
        )),
    })?;
    Some(TypeErr::new(pos, &msg))
}

/// The chain as it is written, such as `a.b.c`.
fn chain_text(chain: &[AST]) -> String {
    let links: Vec<String> = chain.iter().map(|ast| ast.node.to_string()).collect();
    links.join(".")
}

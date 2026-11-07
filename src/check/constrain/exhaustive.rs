//! Match exhaustiveness and reachability, checked once every subject has a type.
//!
//! This is the usefulness algorithm of Maranget, *Warnings for pattern matching*, JFP 2007, for the patterns Mamba has.
//! A pattern is useful against earlier rows if some value matches it but none of them.
//! A match is exhaustive if a wildcard is not useful against its arms, and an arm is unreachable if it is not useful against the arms above it.

use std::collections::HashSet;
use std::slice::from_ref;

use crate::check::constrain::unify::finished::Finished;
use crate::check::context::clss::{BOOL, NONE, TUPLE};
use crate::check::name::true_name::TrueName;
use crate::check::name::Name;
use crate::check::result::TypeErr;
use crate::common::position::Position;
use crate::parse::ast::{Node, AST};

/// A pattern reduced to what decides which values it matches.
#[derive(Clone, Debug)]
enum Pat {
    /// `_` or a name, which match every value.
    Wild,
    /// Any other pattern, such as `0`, `"a"`, `True` or `None`, which matches only a value equal to it.
    Lit(Node),
    Tuple(Vec<Pat>),
}

/// What a pattern can take apart, as opposed to a wildcard.
#[derive(Clone, Debug)]
enum Ctor {
    Lit(Node),
    Tuple(usize),
}

/// The values a column can hold, as far as patterns can tell them apart.
#[derive(Clone, Debug)]
struct Domain {
    kind: Kind,
    nullable: bool,
}

#[derive(Clone, Debug)]
enum Kind {
    Bool,
    Tuple(Vec<Domain>),
    /// Too many values to list, such as an `Int`, so only a wildcard covers them all.
    Open,
}

/// Report every match that misses a value, and every arm that can never be reached.
pub fn check_matches(matches: &[AST], finished: &Finished) -> Result<(), Vec<TypeErr>> {
    let mut seen = HashSet::new();
    let errs: Vec<TypeErr> = matches
        .iter()
        .filter(|ast| seen.insert(ast.pos))
        .flat_map(|ast| check_match(ast, finished))
        .collect();

    if errs.is_empty() {
        Ok(())
    } else {
        Err(errs)
    }
}

fn check_match(ast: &AST, finished: &Finished) -> Vec<TypeErr> {
    let Node::Match { cond, cases } = &ast.node else {
        return vec![];
    };

    let domain = subject_domain(cond, finished);
    let arms: Vec<(Pat, bool, &AST)> = cases.iter().filter_map(arm).collect();

    let mut errs = unreachable_arms(&arms, &domain);
    errs.extend(missing_value(cond, &arms, &domain));
    errs
}

/// An arm's pattern, whether it has a guard, and the arm itself.
fn arm(case: &AST) -> Option<(Pat, bool, &AST)> {
    let Node::Case { cond, guard, .. } = &case.node else {
        return None;
    };
    Some((pattern(cond), guard.is_some(), case))
}

/// An arm is unreachable if the unguarded arms above it already match every value it does.
///
/// A guarded arm above can always fail, so it never hides one below it.
fn unreachable_arms(arms: &[(Pat, bool, &AST)], domain: &Domain) -> Vec<TypeErr> {
    let mut above: Vec<Vec<Pat>> = vec![];
    let mut errs = vec![];
    for (pat, guarded, case) in arms {
        if useful(&above, from_ref(pat), from_ref(domain)).is_none() {
            let msg = "This arm can never match, the arms above it cover every value it does";
            errs.push(TypeErr::new(case.pos, msg));
        }
        if !guarded {
            above.push(vec![pat.clone()]);
        }
    }
    errs
}

/// A match must cover every value of its subject, and a guarded arm covers none for certain.
fn missing_value(subject: &AST, arms: &[(Pat, bool, &AST)], domain: &Domain) -> Option<TypeErr> {
    let rows: Vec<Vec<Pat>> = arms
        .iter()
        .filter(|(_, guarded, _)| !guarded)
        .map(|(pat, _, _)| vec![pat.clone()])
        .collect();

    let witness = useful(&rows, &[Pat::Wild], from_ref(domain))?;
    let msg = format!(
        "This match does not cover every value, such as {}",
        to_node(&witness[0])
    );
    Some(TypeErr::new(subject.pos, &msg))
}

/// A value the row `q` matches but no row in `rows` does, one pattern per column, if there is one.
fn useful(rows: &[Vec<Pat>], q: &[Pat], domains: &[Domain]) -> Option<Vec<Pat>> {
    let Some((head, _)) = q.split_first() else {
        return rows.is_empty().then(Vec::new);
    };

    let domain = refine(&domains[0], rows, head);
    match ctor_of(head) {
        Some(ctor) => useful_ctor(rows, q, &ctor, &domain, &domains[1..]),
        None => useful_wild(rows, &q[1..], &domain, &domains[1..]),
    }
}

/// Usefulness of a row whose first pattern takes values apart, which only rows agreeing with it can compete with.
fn useful_ctor(
    rows: &[Vec<Pat>],
    q: &[Pat],
    ctor: &Ctor,
    domain: &Domain,
    rest: &[Domain],
) -> Option<Vec<Pat>> {
    let rows = specialize(rows, ctor);
    let q = specialize_row(q, ctor)?;
    let domains = sub_domains(ctor, domain, rest);
    useful(&rows, &q, &domains).map(|witness| rebuild(ctor, witness))
}

/// Usefulness of a row starting with a wildcard.
///
/// If the rows name every value of a finite column, each is tried in turn.
/// Otherwise some value goes unnamed, and only the rows with a wildcard there can still match it.
fn useful_wild(
    rows: &[Vec<Pat>],
    tail: &[Pat],
    domain: &Domain,
    rest: &[Domain],
) -> Option<Vec<Pat>> {
    let seen: Vec<Ctor> = rows.iter().filter_map(|row| ctor_of(&row[0])).collect();
    let all = domain.ctors();
    let missing = all.as_ref().and_then(|all| {
        all.iter()
            .find(|c| !seen.iter().any(|s| same_ctor(s, c)))
            .cloned()
    });

    if let (Some(all), None) = (&all, &missing) {
        return all.iter().find_map(|ctor| {
            let q = [vec![Pat::Wild], tail.to_vec()].concat();
            useful_ctor(rows, &q, ctor, domain, rest)
        });
    }

    let defaults: Vec<Vec<Pat>> = rows
        .iter()
        .filter(|row| matches!(row[0], Pat::Wild))
        .map(|row| row[1..].to_vec())
        .collect();
    let witness = useful(&defaults, tail, rest)?;
    let head = missing.map_or(Pat::Wild, |ctor| example(&ctor));
    Some([vec![head], witness].concat())
}

/// The rows that match a value built with `ctor`, with that first pattern replaced by what it holds.
fn specialize(rows: &[Vec<Pat>], ctor: &Ctor) -> Vec<Vec<Pat>> {
    rows.iter()
        .filter_map(|row| specialize_row(row, ctor))
        .collect()
}

fn specialize_row(row: &[Pat], ctor: &Ctor) -> Option<Vec<Pat>> {
    let inner = match (&row[0], ctor) {
        (Pat::Wild, _) => wilds(ctor),
        (Pat::Lit(lit), Ctor::Lit(other)) if lit.same_value(other) => vec![],
        (Pat::Tuple(elements), Ctor::Tuple(arity)) if elements.len() == *arity => elements.clone(),
        _ => return None,
    };
    Some([inner, row[1..].to_vec()].concat())
}

/// Put the patterns a witness found for the parts of `ctor` back together.
fn rebuild(ctor: &Ctor, witness: Vec<Pat>) -> Vec<Pat> {
    match ctor {
        Ctor::Lit(lit) => [vec![Pat::Lit(lit.clone())], witness].concat(),
        Ctor::Tuple(arity) => {
            let (elements, rest) = witness.split_at(*arity);
            [vec![Pat::Tuple(elements.to_vec())], rest.to_vec()].concat()
        }
    }
}

/// Whether two constructors build the same values, comparing literals by value rather than position.
fn same_ctor(left: &Ctor, right: &Ctor) -> bool {
    match (left, right) {
        (Ctor::Lit(left), Ctor::Lit(right)) => left.same_value(right),
        (Ctor::Tuple(left), Ctor::Tuple(right)) => left == right,
        _ => false,
    }
}

fn ctor_of(pat: &Pat) -> Option<Ctor> {
    match pat {
        Pat::Wild => None,
        Pat::Lit(lit) => Some(Ctor::Lit(lit.clone())),
        Pat::Tuple(elements) => Some(Ctor::Tuple(elements.len())),
    }
}

fn wilds(ctor: &Ctor) -> Vec<Pat> {
    match ctor {
        Ctor::Lit(_) => vec![],
        Ctor::Tuple(arity) => vec![Pat::Wild; *arity],
    }
}

/// A value built with `ctor`, to show in an error.
fn example(ctor: &Ctor) -> Pat {
    match ctor {
        Ctor::Lit(lit) => Pat::Lit(lit.clone()),
        Ctor::Tuple(arity) => Pat::Tuple(vec![Pat::Wild; *arity]),
    }
}

/// The domains of the columns that replace the first, once a value built with `ctor` is taken apart.
fn sub_domains(ctor: &Ctor, domain: &Domain, rest: &[Domain]) -> Vec<Domain> {
    let inner = match (ctor, &domain.kind) {
        (Ctor::Tuple(arity), Kind::Tuple(elements)) if elements.len() == *arity => elements.clone(),
        (Ctor::Tuple(arity), _) => vec![Domain::open(); *arity],
        (Ctor::Lit(_), _) => vec![],
    };
    [inner, rest.to_vec()].concat()
}

/// Treat a column as a tuple if its patterns are, when its type did not say so.
fn refine(domain: &Domain, rows: &[Vec<Pat>], head: &Pat) -> Domain {
    if !matches!(domain.kind, Kind::Open) {
        return domain.clone();
    }

    let tuple_arity = std::iter::once(head)
        .chain(rows.iter().map(|row| &row[0]))
        .find_map(|pat| match pat {
            Pat::Tuple(elements) => Some(elements.len()),
            _ => None,
        });
    match tuple_arity {
        Some(arity) => Domain {
            kind: Kind::Tuple(vec![Domain::open(); arity]),
            nullable: domain.nullable,
        },
        None => domain.clone(),
    }
}

impl Domain {
    fn open() -> Domain {
        Domain {
            kind: Kind::Open,
            nullable: false,
        }
    }

    /// Every way to build a value of this domain, or nothing if there are too many to list.
    fn ctors(&self) -> Option<Vec<Ctor>> {
        let mut ctors = match &self.kind {
            Kind::Bool => vec![Ctor::Lit(id("True")), Ctor::Lit(id("False"))],
            Kind::Tuple(elements) => vec![Ctor::Tuple(elements.len())],
            Kind::Open => return None,
        };
        if self.nullable {
            ctors.push(Ctor::Lit(id(NONE)));
        }
        Some(ctors)
    }
}

/// The domain of a subject, seen through a tuple literal so that each element keeps its own type.
///
/// The type of a tuple literal can lose an element's nullability, which its element still has.
fn subject_domain(subject: &AST, finished: &Finished) -> Domain {
    if let Node::Tuple { elements } = &subject.node {
        return Domain {
            kind: Kind::Tuple(
                elements
                    .iter()
                    .map(|e| subject_domain(e, finished))
                    .collect(),
            ),
            nullable: false,
        };
    }
    finished
        .pos_to_name
        .get(&subject.pos)
        .map_or_else(Domain::open, name_domain)
}

/// The domain of a type, where `None` in a union makes the rest nullable.
fn name_domain(name: &Name) -> Domain {
    let is_none = |true_name: &&TrueName| true_name.variant.name == NONE;
    let nullable = name.names.iter().any(|n| n.is_nullable || is_none(&n));
    let values: Vec<&TrueName> = name.names.iter().filter(|n| !is_none(n)).collect();

    let kind = match values.as_slice() {
        [only] if only.variant.name == BOOL => Kind::Bool,
        [only] if only.variant.name == TUPLE => {
            Kind::Tuple(only.variant.generics.iter().map(name_domain).collect())
        }
        _ => Kind::Open,
    };
    Domain { kind, nullable }
}

/// Reduce a match case's pattern.
fn pattern(ast: &AST) -> Pat {
    match &ast.node {
        Node::ExpressionType { expr, .. } => pattern(expr),
        Node::Underscore => Pat::Wild,
        Node::Id { lit } if !matches!(lit.as_str(), "True" | "False" | NONE) => Pat::Wild,
        Node::Tuple { elements } => Pat::Tuple(elements.iter().map(pattern).collect()),
        other => Pat::Lit(other.clone()),
    }
}

/// `True`, `False` and `None` as the parser gives them.
fn id(lit: &str) -> Node {
    Node::Id {
        lit: String::from(lit),
    }
}

/// A witness as a node, so it is shown the way the compiler shows any other pattern.
fn to_node(pat: &Pat) -> Node {
    match pat {
        Pat::Wild => Node::Underscore,
        Pat::Lit(node) => node.clone(),
        Pat::Tuple(elements) => Node::Tuple {
            elements: elements
                .iter()
                .map(|e| AST::new(Position::invisible(), to_node(e)))
                .collect(),
        },
    }
}

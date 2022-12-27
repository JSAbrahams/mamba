use std::convert::TryFrom;

use crate::check::constrain::constraint::builder::ConstrBuilder;
use crate::check::constrain::constraint::expected::{Expect, Expected};
use crate::check::constrain::constraint::expected::Expect::*;
use crate::check::constrain::generate::{Constrained, gen_vec};
use crate::check::constrain::generate::env::Environment;
use crate::check::context::Context;
use crate::check::ident::Identifier;
use crate::check::name::{Any, Name};
use crate::check::result::{TypeErr, TypeResult};
use crate::parse::ast::{AST, Node};

pub fn gen_coll(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder,
) -> Constrained {
    match &ast.node {
        Node::Set { elements } | Node::List { elements } | Node::Tuple { elements } => {
            constr_col(ast, env, constr, None)?;
            gen_vec(elements, env, false, ctx, constr)
        }

        Node::SetBuilder { .. } => {
            Err(vec![TypeErr::new(ast.pos, "Set builders currently not supported")])
        }
        Node::ListBuilder { .. } => {
            Err(vec![TypeErr::new(ast.pos, "List builders currently not supported")])
        }
        _ => Err(vec![TypeErr::new(ast.pos, "Expected collection")]),
    }
}

/// Generate constraint for collection by taking first element.
///
/// The assumption here being that every element in the set has the same type.
pub fn constr_col(
    collection: &AST,
    env: &Environment,
    constr: &mut ConstrBuilder,
    temp_type: Option<Name>,
) -> TypeResult<()> {
    let (msg, col) = match &collection.node {
        Node::Set { elements } | Node::List { elements } => {
            let ty = if let Some(first) = elements.first() {
                for element in elements {
                    let parent = Expected::try_from((first, &env.var_mappings))?;
                    let child = Expected::try_from((element, &env.var_mappings))?;
                    constr.add("collection item", &parent, &child)
                }
                Box::from(Expected::new(first.pos, &Expect::try_from((first, &env.var_mappings))?))
            } else {
                Box::from(Expected::new(collection.pos, &Expect::any()))
            };

            ("collection", Collection { ty })
        }
        Node::Tuple { elements } => {
            let map = |ast: &AST| Expected::try_from((ast, &env.var_mappings));
            let elements = elements.iter().map(map).collect::<Result<_, _>>()?;
            ("tuple", Tuple { elements })
        }

        _ => {
            let expect = if let Some(name) = temp_type { Type { name } } else { Expect::any() };
            let expected = Collection { ty: Box::from(Expected::new(collection.pos, &expect)) };
            ("collection", expected)
        }
    };

    let col_exp = Expected::new(collection.pos, &col);
    constr.add(msg, &col_exp, &Expected::try_from((collection, &env.var_mappings))?);
    Ok(())
}

/// Constrain lookup an collection.
///
/// Adds constraint of collection of type lookup, and the given collection.
pub fn gen_collection_lookup(
    lookup: &AST,
    col: &AST,
    env: &Environment,
    constr: &mut ConstrBuilder,
) -> Constrained {
    let mut env = env.clone();

    // Make col constraint before inserting environment, in case shadowed here
    let col_exp = Expected::try_from((col, &env.var_mappings))?;
    for (mutable, var) in Identifier::try_from(lookup)?.fields(lookup.pos)? {
        env = env.insert_var(mutable, &var, &Expected::new(lookup.pos, &Expect::any()));
    }

    let col_ty_exp = Expected::new(
        col.pos,
        &Collection { ty: Box::from(Expected::try_from((lookup, &env.var_mappings))?) },
    );

    constr.add("collection lookup", &col_ty_exp, &col_exp);
    Ok(env)
}

#[cfg(test)]
mod tests {
    use crate::check::ast::NodeTy;
    use crate::check::check_all;
    use crate::check::name::Name;
    use crate::parse::parse;

    #[test]
    fn for_col_variable_ty() {
        let src = "def a := 0 ..= 2\nfor i in a do\n    print(\"hello\")";
        let ast = parse(src).unwrap();
        let result = check_all(&[*ast]).unwrap();

        let statements = if let NodeTy::Block { statements } = &result[0].node {
            statements.clone()
        } else {
            panic!()
        };

        let (col, expr) = match &statements[1].node {
            NodeTy::For { col, expr, .. } => (col.clone(), expr.clone()),
            other => panic!("Expected for: {:?}", other)
        };

        assert_eq!(expr.ty, Some(Name::from("Int")));
        assert_eq!(col.ty, Some(Name::from("Range")));
    }
}

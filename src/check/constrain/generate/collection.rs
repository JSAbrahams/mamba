use std::convert::TryFrom;

use crate::check::constrain::constraint::builder::ConstrBuilder;
use crate::check::constrain::constraint::Constraint;
use crate::check::constrain::constraint::expected::Expect::*;
use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::generate::{Constrained, gen_vec, generate};
use crate::check::constrain::generate::env::Environment;
use crate::check::context::clss::{COLLECTION, LIST, SET};
use crate::check::context::Context;
use crate::check::ident::Identifier;
use crate::check::name::{Any, Empty, Name, Union};
use crate::check::name::string_name::StringName;
use crate::check::result::{TypeErr, TypeResult};
use crate::parse::ast::{AST, Node};

pub fn gen_coll(ast: &AST, env: &Environment, ctx: &Context, constr: &mut ConstrBuilder)
                -> Constrained {
    match &ast.node {
        Node::Set { elements } | Node::List { elements } => {
            gen_vec(elements, env, false, ctx, constr)?;
            constr_col(ast, env, constr)?;
            Ok(env.clone())
        }
        Node::Tuple { elements } => {
            let res = gen_vec(elements, env, env.is_def_mode, ctx, constr)?;
            constr_col(ast, env, constr)?;
            Ok(res)
        }

        Node::SetBuilder { item, conditions } | Node::ListBuilder { item, conditions } => {
            let conds_env = generate(item, &env.is_def_mode(true), ctx, constr)?;
            if let Some(cond) = conditions.first() {
                let Node::In { left, right } = &cond.node else {
                    let msg = format!("Expected in, was {}", cond.node);
                    return Err(vec![TypeErr::new(cond.pos, &msg)]);
                };

                let item = Expected::from(left);
                let temp_name = constr.temp_var();
                let temp_ty = Expected::new(left.pos, &Type { name: Name::from(temp_name.as_str()) });
                constr.add("temporary builder type", &item, &temp_ty, env);

                let col_exp = Expected::collection(right.pos, &Name::from(temp_name.as_str()));
                constr.add("comprehension collection type", &col_exp, &Expected::from(right), env);

                generate(cond, &conds_env.is_def_mode(false), ctx, constr)?;
                if let Some(conditions) = conditions.strip_prefix(&[cond.clone()]) {
                    for cond in conditions {
                        generate(cond, &conds_env.is_def_mode(false), ctx, constr)?;
                        let cond = Expected::from(cond);
                        constr.add_constr(&Constraint::truthy("comprehension condition", &cond), &conds_env);
                    }
                }

                // if define mode, propagate out conditions environment
                Ok(if env.is_def_mode { conds_env } else { env.clone() })
            } else {
                Err(vec![TypeErr::new(ast.pos, "Builder must have a least one element")])
            }
        }
        _ => Err(vec![TypeErr::new(ast.pos, "Expected collection")]),
    }
}

/// Generate constraint for collection by taking first element.
///
/// The assumption here being that every element in the set has the same type.
pub fn constr_col(collection: &AST, env: &Environment, constr: &mut ConstrBuilder)
                  -> TypeResult<()> {
    let (col_ty, col_items_ty) = match &collection.node {
        Node::Set { elements } => (SET, constraint_collection_items(elements, env, constr)?),
        Node::List { elements } => (LIST, constraint_collection_items(elements, env, constr)?),
        Node::Tuple { elements } => {
            let exp = Tuple { elements: elements.iter().map(Expected::from).collect() };
            let col_exp = Expected::new(collection.pos, &exp);
            constr.add("tuple", &col_exp, &Expected::from(collection), env);
            return Ok(());
        }

        _ => (COLLECTION, Name::any())
    };

    let col_exp = Type { name: Name::from(&StringName::new(col_ty, &[col_items_ty])) };
    let col_exp = Expected::new(collection.pos, &col_exp);
    constr.add("collection", &col_exp, &Expected::from(collection), env);
    Ok(())
}

/// For each item in a collection, create a temporary type and return a union of these types.
///
/// A constraint is also generated for each item and temporary name in the set.
fn constraint_collection_items(elements: &[AST], env: &Environment, constr: &mut ConstrBuilder)
                               -> Constrained<Name> {
    let mut name = Name::empty();
    for element in elements {
        let exp_element = Expected::from(element);
        let new_name = constr.temp_var();
        name = name.union(&Name::from(new_name.as_str()));

        let exp_ty = Expected::new(element.pos, &Type { name: Name::from(new_name.as_str()) });
        constr.add("collection element", &exp_ty, &exp_element, env);
    }

    Ok(if name.is_empty() { Name::any() } else { name })
}

/// Constrain lookup an collection.
///
/// Adds constraint of collection of type lookup, and the given collection.
pub fn gen_collection_lookup(lookup: &AST, _col: &AST, env: &Environment, constr: &mut ConstrBuilder)
                             -> Constrained {
    let mut env = env.clone();

    for (mutable, var) in Identifier::try_from(lookup)?.fields(lookup.pos)? {
        constr.insert_var(&var);
        env = env.insert_var(mutable, &var, &Expected::any(lookup.pos), &constr.var_mapping);
    }

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

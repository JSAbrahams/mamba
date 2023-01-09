use std::convert::TryFrom;

use crate::check::constrain::constraint::builder::ConstrBuilder;
use crate::check::constrain::constraint::Constraint;
use crate::check::constrain::constraint::expected::Expect::*;
use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::generate::{Constrained, gen_vec, generate};
use crate::check::constrain::generate::env::Environment;
use crate::check::context::clss::{COLLECTION, LIST, SET, TUPLE};
use crate::check::context::Context;
use crate::check::ident::Identifier;
use crate::check::name::{Any, Empty, Name, Union};
use crate::check::name::string_name::StringName;
use crate::check::result::TypeErr;
use crate::parse::ast::{AST, Node};

pub fn gen_coll(ast: &AST, env: &Environment, ctx: &Context, constr: &mut ConstrBuilder)
                -> Constrained {
    match &ast.node {
        Node::Set { elements } | Node::List { elements } => {
            gen_vec(elements, env, false, ctx, constr)?;
            gen_col(ast, env, constr)?;
            Ok(env.clone())
        }
        Node::Dict { elements } => {
            let elements: Vec<AST> = elements.iter().flat_map(|(from, to)| [from.clone(), to.clone()]).collect();
            gen_vec(&elements, env, false, ctx, constr)?;

            constr_col(ast, env, constr, None)?;
            Ok(env.clone())
        }
        Node::Tuple { elements } => {
            let res = gen_vec(elements, env, env.is_def_mode, ctx, constr)?;
            gen_col(ast, env, constr)?;
            Ok(res)
        }
        Node::DictBuilder { .. } => Ok(env.clone()),
        Node::SetBuilder { item, conditions } => {
            let builder_env = gen_builder(ast, item, conditions, env, ctx, constr)?;

            let item = retrieve_nested_builder_item(item);
            let set = AST::new(ast.pos, Node::Set { elements: vec![item] });
            gen_col(&set, &builder_env, constr)
        }
        Node::ListBuilder { item, conditions } => {
            let builder_env = gen_builder(ast, item, conditions, env, ctx, constr)?;

            let item = retrieve_nested_builder_item(item);
            let set = AST::new(ast.pos, Node::List { elements: vec![item] });
            gen_col(&set, &builder_env, constr)
        }
        _ => Err(vec![TypeErr::new(ast.pos, "Expected collection")]),
    }
}

fn retrieve_nested_builder_item(ast: &AST) -> AST {
    match &ast.node {
        Node::SetBuilder { item, .. } => *item.clone(),
        Node::ListBuilder { item, .. } => *item.clone(),
        _ => ast.clone()
    }
}

fn gen_builder(ast: &AST, item: &AST, conditions: &[AST], env: &Environment, ctx: &Context, constr: &mut ConstrBuilder) -> Constrained {
    if let Some(cond) = conditions.first() {
        let Node::In { left, right } = &cond.node else {
            let msg = format!("Expected in, was {}", cond.node);
            return Err(vec![TypeErr::new(cond.pos, &msg)]);
        };

        generate(right, env, ctx, constr)?;
        let conds_env = generate(left, &env.is_def_mode(true), ctx, constr)?.is_def_mode(false);
        let conds_env = constr_col_lookup(left, right, &conds_env, constr)?;

        generate(item, &conds_env, ctx, constr)?;

        if let Some(conditions) = conditions.strip_prefix(&[cond.clone()]) {
            for cond in conditions {
                generate(cond, &conds_env, ctx, constr)?;
                let cond = Expected::from(cond);
                constr.add_constr(&Constraint::truthy("comprehension condition", &cond), &conds_env);
            }
        }

        Ok(if env.is_def_mode { conds_env } else { env.clone() })
    } else {
        Err(vec![TypeErr::new(ast.pos, "Builder must have a least one element")])
    }
}

/// Generate constraint for collection by taking first element.
///
/// The assumption here being that every element in the set has the same type.
fn gen_col(collection: &AST, env: &Environment, constr: &mut ConstrBuilder) -> Constrained {
    let (col_ty, col_items_ty) = match &collection.node {
        Node::Set { elements } => (SET, gen_col_items(elements, env, constr)?),
        Node::List { elements } => (LIST, gen_col_items(elements, env, constr)?),
        Node::Dict { elements } => {
            unimplemented!();
        }
        Node::Tuple { elements } => {
            let mut names = vec![];
            for element in elements {
                let exp_element = Expected::from(element);
                let new_name = constr.temp_name();
                names.push(new_name.clone());

                let exp_ty = Expected::new(element.pos, &Type { name: new_name });
                constr.add("collection element", &exp_ty, &exp_element, env);
            }

            let col_exp = Type { name: Name::from(&StringName::new(TUPLE, &names)) };
            let col_exp = Expected::new(collection.pos, &col_exp);
            constr.add("collection", &col_exp, &Expected::from(collection), env);
            return Ok(env.clone());
        }
        _ => (COLLECTION, Name::any())
    };

    let col_exp = Type { name: Name::from(&StringName::new(col_ty, &[col_items_ty])) };
    let col_exp = Expected::new(collection.pos, &col_exp);
    constr.add("collection", &col_exp, &Expected::from(collection), env);
    Ok(env.clone())
}

/// For each item in a collection, create a temporary type and return a union of these types.
///
/// A constraint is also generated for each item and temporary name in the set.
fn gen_col_items(elements: &[AST], env: &Environment, constr: &mut ConstrBuilder) -> Constrained<Name> {
    let mut name = Name::empty();
    for element in elements {
        let exp_element = Expected::from(element);
        let new_name = constr.temp_name();
        name = name.union(&new_name);

        let exp_ty = Expected::new(element.pos, &Type { name: new_name });
        constr.add("collection element", &exp_ty, &exp_element, env);
    }

    Ok(if name.is_empty() { Name::any() } else { name })
}

/// Constrain lookup an collection.
///
/// Adds constraint of collection of type lookup, and the given collection.
pub fn constr_col_lookup(lookup: &AST, col: &AST, env: &Environment, constr: &mut ConstrBuilder)
                         -> Constrained {
    let mut env = env.clone();
    let (temp_name, helper_ty) = (constr.temp_name(), constr.temp_name());

    let (col_exp1, col_exp2) = Constraint::iterable("collection lookup", &Expected::from(col), &temp_name, &helper_ty);
    constr.add_constr(&col_exp1, &env);
    constr.add_constr(&col_exp2, &env);

    for (mutable, var) in Identifier::try_from(lookup)?.fields(lookup.pos)? {
        constr.insert_var(&var);
        env = env.insert_var(mutable, &var, &Expected::any(lookup.pos), &constr.var_mapping);
    }

    let exp_lookup_temp = Expected::new(lookup.pos, &Type { name: temp_name });
    constr.add("lookup type", &exp_lookup_temp, &Expected::from(lookup), &env);
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
        assert_eq!(col.ty, Some(Name::from("Int")));
    }
}

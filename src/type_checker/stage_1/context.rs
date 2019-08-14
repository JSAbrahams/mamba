use crate::parser::ast::{ASTNode, ASTNodePos};
use crate::type_checker::stage_1::class::{Class, Interface};
use crate::type_checker::stage_1::field::Field;
use crate::type_checker::stage_1::function::Function;
use crate::type_checker::stage_1::Context;

macro_rules! is {
    ($node_pos:expr, $ast:ident) => {{
        match $node_pos.node {
            ASTNode::$ast { .. } => true,
            _ => false
        }
    }};
}

impl Context {
    pub fn new(node_pos: &[ASTNodePos]) -> Result<Context, Vec<String>> {
        // TODO use file location for import analysis
        let (all_modules, all_type_defs) = node_pos.iter().try_fold(
            (vec![], vec![]),
            |(mut all_modules, mut all_type_defs), node_pos| match &node_pos.node {
                ASTNode::File { modules, type_defs, .. } => {
                    all_modules.extend(modules);
                    all_type_defs.extend(type_defs);
                    Ok((all_modules, all_type_defs))
                }
                other => Err(vec![format!("Expected file but got {:?}", other)])
            }
        )?;

        let (interfaces, interface_errs): (Vec<_>, Vec<_>) = all_type_defs
            .into_iter()
            .map(|node_pos| Interface::new(&node_pos))
            .partition(Result::is_ok);
        let (classes, class_errs): (Vec<_>, Vec<_>) = all_modules
            .iter()
            .filter(|node_pos| is!(node_pos, Class))
            .map(|node_pos| Class::new(&node_pos))
            .partition(Result::is_ok);
        let (fields, field_errs): (Vec<_>, Vec<_>) = all_modules
            .iter()
            .filter(|node_pos| is!(node_pos, VarDef))
            .map(|node_pos| Field::new(&node_pos))
            .partition(Result::is_ok);
        let (functions, function_errs): (Vec<_>, Vec<_>) = all_modules
            .iter()
            .filter(|node_pos| is!(node_pos, FunDef))
            .map(|node_pos| Function::new(None, &node_pos))
            .partition(Result::is_ok);

        let all_errs = [
            interface_errs.into_iter().map(Result::unwrap_err).collect::<Vec<_>>(),
            class_errs.into_iter().map(Result::unwrap_err).collect::<Vec<_>>(),
            field_errs.into_iter().map(Result::unwrap_err).collect::<Vec<_>>(),
            function_errs.into_iter().map(Result::unwrap_err).collect::<Vec<_>>()
        ]
        .concat();

        if all_errs.is_empty() {
            Ok(Context {
                interfaces: interfaces.into_iter().map(Result::unwrap).collect(),
                classes:    classes.into_iter().map(Result::unwrap).collect(),
                fields:     fields.into_iter().map(Result::unwrap).collect(),
                functions:  functions.into_iter().map(Result::unwrap).collect()
            })
        } else {
            Err(all_errs)
        }
    }
}

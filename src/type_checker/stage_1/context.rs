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
        let mut all_modules = vec![];
        let mut all_type_defs = vec![];
        for np in node_pos {
            match &np.node {
                ASTNode::File { modules, type_defs, .. } => {
                    all_modules.append(&mut modules.clone());
                    all_type_defs.append(&mut type_defs.clone());
                }
                other => return Err(vec![format!("Expected file but got {:?}", other)])
            }
        }

        let (interfaces, interface_errors): (Vec<_>, Vec<_>) = all_type_defs
            .into_iter()
            .map(|node_pos| Interface::new(&node_pos))
            .partition(Result::is_ok);
        let (classes, class_errors): (Vec<_>, Vec<_>) = all_modules
            .clone()
            .into_iter()
            .filter(|node_pos| is!(node_pos, Class))
            .map(|node_pos| Class::new(&node_pos))
            .partition(Result::is_ok);
        let (fields, field_errors): (Vec<_>, Vec<_>) = all_modules
            .clone()
            .into_iter()
            .filter(|node_pos| is!(node_pos, VariableDef))
            .map(|node_pos| Field::new(&node_pos))
            .partition(Result::is_ok);
        let (functions, function_errors): (Vec<_>, Vec<_>) = all_modules
            .clone()
            .into_iter()
            .filter(|node_pos| is!(node_pos, FunDef))
            .map(|node_pos| Function::new(&node_pos))
            .partition(Result::is_ok);

        if !interface_errors.is_empty()
            || !class_errors.is_empty()
            || !field_errors.is_empty()
            || !function_errors.is_empty()
        {
            let mut interface_errors: Vec<String> =
                interface_errors.into_iter().map(Result::unwrap_err).collect();
            let mut class_errors: Vec<String> =
                class_errors.into_iter().map(Result::unwrap_err).collect();
            let mut field_errors: Vec<String> =
                field_errors.into_iter().map(Result::unwrap_err).collect();
            let mut function_errors: Vec<String> =
                function_errors.into_iter().map(Result::unwrap_err).collect();

            interface_errors.append(&mut class_errors);
            interface_errors.append(&mut field_errors);
            interface_errors.append(&mut function_errors);
            return Err(interface_errors);
        }

        let interfaces = interfaces.into_iter().map(Result::unwrap).collect();
        let classes = classes.into_iter().map(Result::unwrap).collect();
        let fields = fields.into_iter().map(Result::unwrap).collect();
        let functions = functions.into_iter().map(Result::unwrap).collect();

        Ok(Context { interfaces, classes, fields, functions })
    }
}

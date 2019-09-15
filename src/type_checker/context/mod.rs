use std::collections::HashMap;
use std::convert::TryFrom;

use crate::common::position::Position;
use crate::parser::ast::Node;
use crate::type_checker::context::concrete::type_name::TypeName;
use crate::type_checker::context::concrete::Type;
use crate::type_checker::context::generic::field::GenericField;
use crate::type_checker::context::generic::function::GenericFunction;
use crate::type_checker::context::generic::GenericType;
use crate::type_checker::type_result::TypeErr;
use crate::type_checker::CheckInput;

mod generic;

pub mod concrete;

/// A context stores all information of all identified types of the current
/// application.
///
/// Functions and fields are also stored alongside identified classes such that
/// we can also check usage of top-level fields and functions.
#[derive(Debug)]
pub struct Context {
    types:     Vec<GenericType>,
    functions: Vec<GenericFunction>,
    fields:    Vec<GenericField>
}

impl Context {
    pub fn lookup(&self, type_name: &TypeName, pos: &Position) -> Result<Type, TypeErr> {
        unimplemented!()
    }
}

impl TryFrom<&[CheckInput]> for Context {
    type Error = Vec<TypeErr>;

    fn try_from(files: &[CheckInput]) -> Result<Self, Self::Error> {
        let mut results = vec![];
        let mut fun_res = vec![];
        let mut field_res = vec![];

        files.iter().for_each(|(file, source, path)| match &file.node {
            Node::File { pure, modules, .. } =>
                modules.iter().for_each(|module| match &module.node {
                    Node::Class { .. } | Node::TypeDef { .. } => results.push(
                        GenericType::try_from(module)
                            .and_then(|ty| ty.all_pure(*pure).map_err(|e| vec![e]))
                            .map_err(|errs| {
                                errs.into_iter().map(|e| e.into_with_source(source, path)).collect()
                            })
                    ),
                    Node::FunDef { .. } => fun_res.push(
                        GenericFunction::try_from(module)
                            .and_then(|f| f.in_class(None))
                            .map_err(|e| e.into_with_source(source, path))
                    ),
                    Node::VariableDef { .. } => field_res.push(
                        GenericField::try_from(module)
                            .map_err(|e| e.into_with_source(source, path))
                    ),
                    _ => {}
                }),
            _ => results.push(Err(vec![TypeErr::new(&file.pos, "Expected file")]))
        });

        let (types, type_errs): (Vec<_>, Vec<_>) = results.into_iter().partition(Result::is_ok);
        let (functions, fun_errs): (Vec<_>, Vec<_>) = fun_res.into_iter().partition(Result::is_ok);
        let (fields, field_errs): (Vec<_>, Vec<_>) = field_res.into_iter().partition(Result::is_ok);

        if !type_errs.is_empty() || !fun_errs.is_empty() || !field_errs.is_empty() {
            let mut errs = vec![];
            errs.append(&mut type_errs.into_iter().map(Result::unwrap_err).flatten().collect());
            errs.append(&mut fun_errs.into_iter().map(Result::unwrap_err).collect());
            errs.append(&mut field_errs.into_iter().map(Result::unwrap_err).collect());
            Err(errs)
        } else {
            Ok(Context {
                types:     types.into_iter().map(Result::unwrap).collect(),
                functions: functions.into_iter().map(Result::unwrap).collect(),
                fields:    fields.into_iter().map(Result::unwrap).collect()
            })
        }
    }
}

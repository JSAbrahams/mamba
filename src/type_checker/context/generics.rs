use crate::parser::ast::Node;
use crate::type_checker::context::field::generic::GenericField;
use crate::type_checker::context::function::generic::GenericFunction;
use crate::type_checker::context::ty::generic::GenericType;
use crate::type_checker::type_result::{TypeErr, TypeResult};
use crate::type_checker::CheckInput;
use std::convert::TryFrom;

pub fn generics(
    files: &[CheckInput]
) -> TypeResult<(Vec<GenericType>, Vec<GenericField>, Vec<GenericFunction>)> {
    let mut results = vec![];
    let mut fun_res = vec![];
    let mut field_res = vec![];

    files.iter().for_each(|(file, source, path)| match &file.node {
        Node::File { pure, modules, .. } => modules.iter().for_each(|module| match &module.node {
            Node::Class { .. } | Node::TypeDef { .. } => results.push(
                GenericType::try_from(module)
                    .and_then(|ty| ty.all_pure(*pure).map_err(|e| vec![e]))
                    .map_err(|errs| {
                        errs.into_iter().map(|e| e.into_with_source(source, path)).collect()
                    })
            ),
            Node::Script { statements } => statements.iter().for_each(|stmt| match &stmt.node {
                Node::FunDef { .. } => fun_res.push(
                    GenericFunction::try_from(stmt)
                        .and_then(|f| f.in_class(None))
                        .map_err(|e| e.iter().map(|e| e.into_with_source(source, path)))
                ),
                Node::VariableDef { .. } => field_res.push(
                    GenericField::try_from(stmt)
                        .map_err(|e| e.iter().map(|e| e.into_with_source(source, path)))
                ),
                _ => {}
            }),
            // TODO process imports
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
        let types: Vec<_> = types.into_iter().map(Result::unwrap).collect();
        let fields: Vec<_> = fields.into_iter().map(Result::unwrap).collect();
        let functions: Vec<_> = functions.into_iter().map(Result::unwrap).collect();
        Ok((types, fields, functions))
    }
}

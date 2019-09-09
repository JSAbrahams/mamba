use std::convert::TryFrom;

use crate::parser::ast::ASTNode;
use crate::type_checker::context::field::Field;
use crate::type_checker::context::function::Function;
use crate::type_checker::type_result::TypeErr;
use crate::type_checker::CheckInput;

pub struct Environment {
    pub functions: Vec<Function>,
    pub fields:    Vec<Field>
}

impl TryFrom<&[CheckInput]> for Environment {
    type Error = Vec<TypeErr>;

    fn try_from(files: &[CheckInput]) -> Result<Self, Self::Error> {
        let mut errs: Vec<TypeErr> = vec![];
        let mut fun_res: Vec<Result<Function, TypeErr>> = vec![];
        let mut field_res: Vec<Result<Field, TypeErr>> = vec![];

        files.iter().for_each(|(file, source, path)| match &file.node {
            ASTNode::File { pure, modules, .. } => modules.iter().for_each(|module| {
                if let ASTNode::Script { statements } = &module.node {
                    statements.iter().for_each(|statement| match &statement.node {
                        ASTNode::FunDef { .. } => fun_res.push(
                            Function::try_from(statement)
                                .map(|function| function.pure(*pure))
                                .and_then(|function| function.in_class(false))
                                .map_err(|err| err.into_with_source(source.clone(), path))
                        ),
                        ASTNode::VariableDef { .. } => field_res.push(
                            Field::try_from(statement)
                                .map_err(|err| err.into_with_source(source.clone(), path))
                        ),
                        _ => {}
                    })
                } else {
                    {}
                }
            }),
            _ => errs.push(TypeErr::new(&file.position, "Expected file"))
        });

        let (functions, fun_errs): (Vec<_>, Vec<_>) = fun_res.into_iter().partition(Result::is_ok);
        let (fields, field_errs): (Vec<_>, Vec<_>) = field_res.into_iter().partition(Result::is_ok);

        if !errs.is_empty() || !fun_errs.is_empty() || !field_errs.is_empty() {
            errs.append(&mut fun_errs.into_iter().map(Result::unwrap_err).collect());
            errs.append(&mut field_errs.into_iter().map(Result::unwrap_err).collect());
            Err(errs)
        } else {
            Ok(Environment {
                functions: functions.into_iter().map(Result::unwrap).collect(),
                fields:    fields.into_iter().map(Result::unwrap).collect()
            })
        }
    }
}

use std::convert::TryFrom;

use crate::common::position::Position;
use crate::parser::ast::Node;
use crate::type_checker::context::field::Field;
use crate::type_checker::context::function::Function;
use crate::type_checker::type_result::{TypeErr, TypeResult};
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
            Node::File { pure, modules, .. } => modules.iter().for_each(|module| {
                if let Node::Script { statements } = &module.node {
                    statements.iter().for_each(|statement| match &statement.node {
                        Node::FunDef { .. } => fun_res.push(
                            Function::try_from(statement)
                                .map(|function| function.pure(*pure))
                                .and_then(|function| function.in_class(false))
                                .map_err(|err| err.into_with_source(source, path))
                        ),
                        Node::VariableDef { .. } => field_res.push(
                            Field::try_from(statement)
                                .map_err(|err| err.into_with_source(source, path))
                        ),
                        _ => {}
                    })
                } else {
                    {}
                }
            }),
            _ => errs.push(TypeErr::new(&file.pos, "Expected file"))
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

impl Environment {
    // TODO implement
    pub fn union(&self, _: &Environment) -> Environment {
        Environment { functions: self.functions.clone(), fields: self.fields.clone() }
    }

    /// Add field to environment.
    ///
    /// Shadows current field.
    ///
    /// # Failure
    ///
    /// If shadowing field with a different type.
    pub fn add(&self, _: &Field, _: &Position) -> TypeResult<Environment> {
        // TODO implement
        unimplemented!()
    }

    /// Add function to environment.
    ///
    /// # Failure
    ///
    /// If attempting to shadow a function.
    pub fn add_function(&self, _: &Function, _: &Position) -> TypeResult<Environment> {
        // TODO implement
        unimplemented!()
    }

    pub fn lookup(&self, variable: &String) -> Option<Field> { unimplemented!() }
}

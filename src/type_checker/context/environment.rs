use crate::common::position::Position;
use crate::parser::ast::Node;
use crate::type_checker::context::field::Field;
use crate::type_checker::context::function::Function;
use crate::type_checker::context::ReturnType;
use crate::type_checker::type_result::{TypeErr, TypeResult};
use crate::type_checker::CheckInput;
use std::collections::HashMap;
use std::convert::TryFrom;

#[derive(Clone)]
pub struct Environment {
    functions: HashMap<String, Function>,
    fields:    HashMap<String, Field>
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
            Ok(Environment::new(
                functions.into_iter().map(Result::unwrap).collect(),
                fields.into_iter().map(Result::unwrap).collect()
            ))
        }
    }
}

impl Environment {
    pub fn new(functions: Vec<Function>, fields: Vec<Field>) -> Environment {
        Environment {
            functions: functions.iter().map(|f| (f.name.clone(), f)).collect(),
            fields:    fields.iter().map(|f| (f.name.clone(), f)).collect()
        }
    }

    // TODO implement
    pub fn union(&self, _: &Environment) -> Environment {
        Environment { functions: self.functions.clone(), fields: self.fields.clone() }
    }

    /// Add field to environment.
    ///
    /// # Failure
    ///
    /// If attempting to shadow field with a different type.
    pub fn add_field(&mut self, field: &Field, pos: &Position) -> TypeResult<()> {
        match self.fields.get(field.name.as_str()) {
            Some(self_field)
                if self_field.get_return_type_name() != field.get_return_type_name() =>
                return Err(TypeErr::new(pos, "Cannot shadow if type is different")),
        }

        self.fields.insert(field.name, field.clone())
    }

    /// Add function to environment.
    ///
    /// # Failure
    ///
    /// If attempting to shadow function that is already in scope.
    pub fn add_function(&mut self, function: &Function, pos: &Position) -> TypeResult<()> {
        if self.functions.contains_key(function.name.as_str()) {
            return Err(TypeErr::new(pos, "Function is already in scope"));
        }

        self.functions.insert(function.name, function.clone())
    }

    pub fn lookup_field(&self, field: &String) -> Option<Field> {
        self.fields.get(field.as_str()).cloned()
    }

    pub fn lookup_function(&self, function: String) -> Option<Function> {
        self.functions.get(function.as_str()).cloned()
    }
}

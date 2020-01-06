use crate::parser::ast::AST;
use crate::type_checker::context::type_name::TypeName;

#[derive(Debug, Clone)]
pub struct Constraint {
    ast:      AST,
    expected: Expected
}

#[derive(Debug, Clone)]
pub enum Expected {
    Exceptions { exceptions: Vec<Expected> },

    AnyExpression,
    NullableExpression,
    Expression { type_name: TypeName },

    Implements { fun: String, args: Vec<Expected> }
}

impl Constraint {
    pub fn new(ast: &AST, expected: Expected) -> Constraint {
        Constraint { ast: ast.clone(), expected }
    }
}

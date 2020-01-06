use crate::parser::ast::AST;
use crate::type_checker::constraints::expected::Expected;

#[derive(Debug)]
pub struct Constraint {
    ast:      AST,
    expected: Box<Expected>
}

impl Clone for Constraint {
    fn clone(&self) -> Self {
        Constraint { ast: self.ast.clone(), expected: self.expected.clone() }
    }
}

impl Constraint {
    pub fn new(ast: &AST, expected: &Expected) -> Constraint {
        Constraint { ast: ast.clone(), expected: Box::from(expected.clone()) }
    }
}

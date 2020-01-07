use crate::parser::ast::AST;
use crate::type_checker::type_name::TypeName;

#[derive(Clone, Debug)]
pub struct Constraints {
    pub constraints: Vec<Constraint>
}

impl Constraints {
    pub fn new() -> Constraints { Constraints { constraints: vec![] } }

    pub fn add(&self, left: &Expect, right: &Expect) -> Constraints {
        let mut constraints = self.constraints.clone();
        constraints.push(Constraint(left.clone(), right.clone()));
        Constraints { constraints }
    }
}

#[derive(Clone, Debug)]
pub struct Constraint(pub Expect, pub Expect);

#[derive(Clone, Debug)]
pub enum Expect {
    Any,
    AnyStatement,
    AnyExpression,

    Statement { ast: AST },
    Expression { ast: AST },
    NullableExpression { ast: AST },

    Type { type_name: TypeName }
}

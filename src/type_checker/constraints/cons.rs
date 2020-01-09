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
    Nullable { expect: Box<Expect> },
    Mutable { expect: Box<Expect> },

    Any { ast: AST },
    AnyExpression,
    Expression { ast: AST },

    Collection { ty: Option<Box<Expect>> },
    Truthy,

    Implements { name: String, args: Vec<Expect> },
    HasField { name: String },
    Type { type_name: TypeName }
}

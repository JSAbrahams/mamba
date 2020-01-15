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

impl From<Constraint> for Constraints {
    fn from(constraint: Constraint) -> Self { Constraints { constraints: vec![constraint] } }
}

#[derive(Clone, Debug)]
pub struct Constraint(pub Expect, pub Expect);

// TODO rework HasField

#[derive(Clone, Debug)]
pub enum Expect {
    Nullable { expect: Box<Expect> },
    Mutable { expect: Box<Expect> },
    Expression { ast: AST },
    ExpressionAny,
    Function { args: Vec<Expect> },

    Collection { ty: Box<Expect> },
    Truthy,

    RaisesAny,
    Raises { type_name: TypeName },

    Implements { type_name: TypeName, args: Vec<Expect> },
    HasField { name: String },

    Type { type_name: TypeName }
}

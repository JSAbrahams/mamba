use crate::type_checker::constraints::constraint::expected::Expected;

pub mod expected;

#[derive(Clone, Debug)]
pub struct Constraints {
    pub constraints: Vec<Constraint>
}

impl Constraints {
    pub fn new() -> Constraints { Constraints { constraints: vec![] } }

    pub fn append(&mut self, constraints: &Constraints) -> Constraints {
        let mut new_constr = self.constraints.clone();
        new_constr.append(&mut constraints.constraints.clone());
        Constraints { constraints: new_constr }
    }

    pub fn add_constraint(&self, constraint: &Constraint) -> Constraints {
        let mut constraints = self.constraints.clone();
        constraints.push(constraint.clone());
        Constraints { constraints }
    }

    pub fn push(&mut self, left: &Expected, right: &Expected) {
        self.constraints.push(Constraint(left.clone(), right.clone()))
    }

    pub fn add(&self, left: &Expected, right: &Expected) -> Constraints {
        let mut constraints = self.constraints.clone();
        constraints.push(Constraint(left.clone(), right.clone()));
        Constraints { constraints }
    }
}

impl From<&Constraint> for Constraints {
    fn from(constraint: &Constraint) -> Self {
        Constraints { constraints: vec![constraint.clone()] }
    }
}

#[derive(Clone, Debug)]
pub struct Constraint(pub Expected, pub Expected);

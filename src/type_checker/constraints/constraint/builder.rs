use crate::common::position::Position;
use crate::type_checker::constraints::constraint::expected::Expected;
use crate::type_checker::constraints::constraint::iterator::Constraints;
use crate::type_checker::constraints::constraint::Constraint;
use crate::type_checker::type_result::{TypeErr, TypeResult};

#[derive(Clone, Debug)]
pub struct ConstrBuilder {
    level:       usize,
    finished:    Vec<Vec<Constraint>>,
    constraints: Vec<Vec<Constraint>>
}

impl ConstrBuilder {
    pub fn new() -> ConstrBuilder {
        ConstrBuilder { level: 0, finished: vec![], constraints: vec![vec![]] }
    }

    pub fn new_set(&mut self, inherit: bool) {
        self.level += 1;
        self.constraints.push(if inherit {
            self.constraints[self.level - 1].clone()
        } else {
            vec![]
        });
    }

    pub fn exit_set(&mut self, pos: &Position) -> TypeResult<()> {
        if self.level == 0 {
            return Err(vec![TypeErr::new(pos, "Something went wrong")]);
        }

        self.finished.push(self.constraints.remove(self.level));
        self.level -= 1;
        Ok(())
    }

    pub fn add(&mut self, left: &Expected, right: &Expected) {
        self.constraints[self.level].push(Constraint::new(left.clone(), right.clone()));
    }

    pub fn all_constr(self) -> Vec<Constraints> {
        let mut finished = self.finished.clone();
        for level in 0..self.level {
            finished.push(self.constraints[level].clone())
        }

        finished.iter().map(|constraints| Constraints::new(constraints)).collect()
    }
}

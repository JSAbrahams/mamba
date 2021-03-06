use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::constraint::iterator::Constraints;
use crate::check::constrain::constraint::Constraint;
use crate::check::context::name::DirectName;
use crate::check::result::{TypeErr, TypeResult};
use crate::common::position::Position;

/// Constraint Builder.
///
/// Allows us to build sets of constraints.
/// This allows us to constrain different parts of the program which may rely on
/// the same logic, without interfering with each other. E.g. different
/// functions within the same class.
///
/// The level indicates how deep we are. A level of 0 indicates that we are at
/// the top-level of a script.
#[derive(Clone, Debug)]
pub struct ConstrBuilder {
    pub level:   usize,
    finished:    Vec<(Vec<DirectName>, Vec<Constraint>)>,
    constraints: Vec<(Vec<DirectName>, Vec<Constraint>)>
}

impl ConstrBuilder {
    pub fn new() -> ConstrBuilder {
        ConstrBuilder { level: 0, finished: vec![], constraints: vec![(vec![], vec![])] }
    }

    pub fn is_top_level(&self) -> bool { self.level == 0 }

    pub fn new_set_in_class(&mut self, inherit_class: bool, class: &DirectName) {
        self.new_set(false);
        if self.level > 0 && inherit_class {
            let mut previous = self.constraints[self.level - 1].0.clone();
            self.constraints[self.level].0.append(&mut previous);
        }
        self.constraints[self.level].0.push(class.clone());
    }

    /// Remove all constraints with where either parent or child is expected
    pub fn remove_expected(&mut self, expected: &Expected) {
        self.constraints[self.level].1 = self.constraints[self.level]
            .1
            .clone()
            .drain_filter(|con| {
                !con.parent.expect.structurally_eq(&expected.expect)
                    && !con.child.expect.structurally_eq(&expected.expect)
            })
            .collect()
    }

    pub fn new_set(&mut self, inherit: bool) {
        self.constraints.push(if inherit {
            (self.constraints[self.level].0.clone(), self.constraints[self.level].1.clone())
        } else {
            (vec![], vec![])
        });
        self.level += 1;
    }

    pub fn exit_set(&mut self, pos: &Position) -> TypeResult<()> {
        if self.level == 0 {
            return Err(vec![TypeErr::new(pos, "Cannot exit top-level set")]);
        }

        self.finished.push(self.constraints.remove(self.level));
        self.level -= 1;
        Ok(())
    }

    pub fn add_with_identifier(
        &mut self,
        msg: &str,
        parent: &Expected,
        child: &Expected,
        idens: &[String]
    ) {
        let mut constr = Constraint::new(msg, parent, child);
        constr.ids.append(&mut Vec::from(idens));
        self.add_constr(&constr);
    }

    pub fn add(&mut self, msg: &str, parent: &Expected, child: &Expected) {
        self.add_constr(&Constraint::new(msg, parent, child));
    }

    pub fn add_constr(&mut self, constraint: &Constraint) {
        self.constraints[self.level].1.push(constraint.clone())
    }

    // It is not redundant
    #[allow(clippy::redundant_clone)]
    pub fn all_constr(self) -> Vec<Constraints> {
        let mut finished = self.finished.clone();
        finished.append(&mut self.constraints.clone());
        finished.iter().map(Constraints::from).collect()
    }
}

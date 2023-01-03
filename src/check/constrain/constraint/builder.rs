use std::cmp::max;

use crate::check::constrain::constraint::Constraint;
use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::constraint::iterator::Constraints;
use crate::check::name::string_name::StringName;
use crate::check::result::{TypeErr, TypeResult};
use crate::common::position::Position;

const PADDING: usize = 2;

/// Constraint Builder.
///
/// Allows us to build sets of constraints.
/// This allows us to constrain different parts of the program which may rely on
/// the same logic, without interfering with each other. E.g. different
/// functions within the same class.
///
/// The level indicates how deep we are. A level of 0 indicates that we are at
/// the top-level of a script.
///
/// We use sets to type check all possible execution paths.
/// We can have multiple sets open at a time.
/// When a constraint is added, we add it to each open path.
#[derive(Debug)]
pub struct ConstrBuilder {
    pub level: usize,
    pub active_ceil: usize,

    constraints: Vec<(Vec<StringName>, Vec<Constraint>)>,
}

impl ConstrBuilder {
    pub fn new() -> ConstrBuilder {
        ConstrBuilder { level: 0, active_ceil: 1, constraints: vec![(vec![], vec![])] }
    }

    pub fn is_top_level(&self) -> bool { self.level == 0 }

    pub fn new_set_in_class(&mut self, class: &StringName) -> usize {
        let lvl = self.new_set();
        for i in self.level..self.active_ceil {
            self.constraints[i].0.push(class.clone());
        }
        lvl
    }

    /// Remove all constraints with where either parent or child is expected
    pub fn remove_expected(&mut self, expected: &Expected) {
        for i in self.level..self.active_ceil {
            self.constraints[i].1 = self.constraints[i]
                .1
                .clone()
                .drain_filter(|con| {
                    !con.left.expect.same_value(&expected.expect)
                        && !con.right.expect.same_value(&expected.expect)
                })
                .collect()
        }
    }

    /// Create new set, and create marker so that we know what set to exit to upon exit.
    pub fn new_set(&mut self) -> usize {
        self.constraints.push(
            (self.constraints[self.level].0.clone(), self.constraints[self.level].1.clone())
        );

        self.active_ceil += 1;
        return self.active_ceil;
    }

    pub fn exit_set_to(&mut self, level: usize, pos: Position) -> TypeResult<()> {
        let level = max(1, level);
        if self.active_ceil == 1 {
            return Err(vec![TypeErr::new(pos, "Cannot exit top-level set")]);
        } else if level > self.active_ceil {
            return Err(vec![TypeErr::new(pos, "Exiting constraint set which doesn't exist")]);
        }

        if level > self.level {
            self.active_ceil = level; // self.level untouched
        } else {
            self.level = level;
            self.active_ceil = self.level + 1;
        }

        Ok(())
    }

    /// Add new constraint to constraint builder with a message.
    pub fn add(&mut self, msg: &str, parent: &Expected, child: &Expected) {
        self.add_constr(&Constraint::new(msg, parent, child));
    }

    /// Add constraint to currently all op sets.
    /// The open sets are the sets at levels between the self.level and active ceiling.
    pub fn add_constr(&mut self, constraint: &Constraint) {
        let gap = String::from_utf8(vec![b' '; self.level * PADDING]).unwrap();
        for i in self.level..self.active_ceil {
            trace!("{gap}Constr[{}]: {} == {}, {}: {}", i, constraint.left.pos, constraint.right.pos, constraint.msg, constraint);
            self.constraints[i].1.push(constraint.clone())
        }
    }

    pub fn current_class(&self) -> Option<StringName> {
        let constraints = self.constraints[self.level].clone().0;
        constraints.last().cloned()
    }

    pub fn all_constr(self) -> Vec<Constraints> {
        self.constraints.iter().map(Constraints::from).collect()
    }
}

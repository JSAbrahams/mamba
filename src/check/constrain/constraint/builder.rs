use std::cmp::max;
use std::collections::HashMap;

use crate::check::constrain::constraint::Constraint;
use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::constraint::iterator::Constraints;
use crate::common::delimit::comma_delm;

pub type VarMapping = HashMap<String, usize>;

pub fn format_var_map(var: &str, offset: &usize) -> String {
    if *offset == 0usize {
        String::from(var)
    } else {
        format!("{var}@{offset}")
    }
}

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
    finished: Vec<Vec<Constraint>>,
    constraints: Vec<Vec<Constraint>>,

    pub var_mapping: VarMapping,
}

impl ConstrBuilder {
    pub fn new() -> ConstrBuilder {
        trace!("Created set at level {}", 0);
        ConstrBuilder { finished: vec![], constraints: vec![vec![]], var_mapping: HashMap::new() }
    }

    pub fn is_top_level(&self) -> bool { self.constraints.len() == 1 }

    /// Insert variable for mapping in current constraint set.
    ///
    /// This prevents shadowed variables from contaminating previous constraints.
    ///
    /// Differs from environment since environment is used to check that variables are defined at a
    /// certain location.
    pub fn insert_var(&mut self, var: &str) {
        let offset = self.var_mapping.get(var).map_or(0, |o| o + 1);
        self.var_mapping.insert(String::from(var), offset);
    }

    /// Create new set, and create marker so that we know what set to exit to upon exit.
    ///
    /// Output may also be ignored.
    /// Useful if we don't want to close the set locally but leave open.
    pub fn new_set(&mut self) -> usize {
        let inherited_constraints = self.constraints.last().expect("Can never be empty");
        self.constraints.push(inherited_constraints.clone());

        trace!("Created set at level {}", self.constraints.len() - 1);
        self.constraints.len()
    }

    /// Return to specified level given.
    ///
    /// - Error if already top-level.
    /// - Error if level greater than ceiling, as we cannot exit non-existent sets.
    pub fn exit_set_to(&mut self, level: usize) {
        let msg_exit = format!("Exit set to level {}", level - 1);

        let level = max(1, level);
        if level == 0 {
            panic!("Cannot exit top-level set");
        } else if level > self.constraints.len() {
            panic!("Exiting constraint set which doesn't exist\nlevel: {}, constraints: {}, finished: {}",
                   level, self.constraints.len(), self.finished.len());
        }

        for i in (level - 1..self.constraints.len()).rev() {
            // Equivalent to pop, but remove has better panic message for debugging
            self.finished.push(self.constraints.remove(i))
        }

        trace!("{msg_exit}: {} active sets, {} complete sets", self.constraints.len(), self.finished.len());
    }

    /// Add new constraint to constraint builder with a message.
    pub fn add(&mut self, msg: &str, parent: &Expected, child: &Expected) {
        self.add_constr(&Constraint::new(msg, parent, child));
    }

    /// Add constraint to currently all op sets.
    /// The open sets are the sets at levels between the self.level and active ceiling.
    pub fn add_constr(&mut self, constraint: &Constraint) {
        for constraints in &mut self.constraints {
            constraints.push(constraint.clone());
        }

        let lvls = comma_delm(0..self.constraints.len());
        trace!("Constr[{}]: {} == {}, {}: {}", lvls, constraint.left.pos, constraint.right.pos, constraint.msg, constraint);
    }

    pub fn all_constr(self) -> Vec<Constraints> {
        let (mut finished, mut constraints) = (self.finished, self.constraints);
        finished.append(&mut constraints);
        finished.iter().map(Constraints::from).collect()
    }
}

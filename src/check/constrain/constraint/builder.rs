use std::cmp::max;
use std::collections::HashMap;

use itertools::enumerate;

use crate::check::constrain::constraint::Constraint;
use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::constraint::iterator::Constraints;
use crate::check::name::string_name::StringName;
use crate::check::result::{TypeErr, TypeResult};
use crate::common::position::Position;

const PADDING: usize = 2;

pub type VarMapping = HashMap<String, (String, usize)>;

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
    finished: Vec<(Vec<StringName>, Vec<Constraint>)>,
    constraints: Vec<(Vec<StringName>, Vec<Constraint>, VarMapping)>,
}

impl ConstrBuilder {
    pub fn new() -> ConstrBuilder {
        ConstrBuilder { finished: vec![], constraints: vec![(vec![], vec![], HashMap::new())] }
    }

    pub fn is_top_level(&self) -> bool { self.constraints.len() == 1 }

    pub fn var_mappings(&self) -> VarMapping {
        let last = self.constraints.last().expect("Can never be empty");
        last.2.clone()
    }

    /// Insert variable for mapping in current constraint set.
    ///
    /// This prevents shadowed variables from contaminating previous constraints.
    ///
    /// Differs from environment since environment is used to check that variables are defined at a
    /// certain location.
    pub fn insert_var(&mut self, var: &str) {
        for constraint in &mut self.constraints {
            let mapping = constraint.2.clone();
            if let Some((var, offset)) = mapping.get(var) {
                constraint.2.insert(String::from(var), (var.clone(), offset + 1));
            }
        }
    }

    pub fn new_set_in_class(&mut self, class: &StringName) -> usize {
        let lvl = self.new_set();
        for constraints in &mut self.constraints {
            constraints.0.push(class.clone());
        }
        lvl
    }

    /// Create new set, and create marker so that we know what set to exit to upon exit.
    ///
    /// Output may also be ignored.
    /// Useful if we don't want to close the set locally but leave open.
    pub fn new_set(&mut self) -> usize {
        let inherited_constraints = self.constraints.last().expect("Can never be empty");
        self.constraints.push(inherited_constraints.clone());
        return self.constraints.len();
    }

    /// Return to specified level given.
    ///
    /// - Error if already top-level.
    /// - Error if level greater than ceiling, as we cannot exit non-existent sets.
    pub fn exit_set_to(&mut self, level: usize, pos: Position) -> TypeResult<()> {
        let level = max(1, level);
        if level == 0 {
            return Err(vec![TypeErr::new(pos, "Cannot exit top-level set")]);
        } else if level > self.constraints.len() {
            return Err(vec![TypeErr::new(pos, "Exiting constraint set which doesn't exist")]);
        }

        for i in (level..self.constraints.len()).rev() {
            // Equivalent to pop, but remove has better panic message for debugging
            let (class_names, constraints, _) = self.constraints.remove(i);
            self.finished.push((class_names, constraints))
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
        let gap = String::from_utf8(vec![b' '; self.constraints.len() * PADDING]).unwrap();

        for (i, constraints) in enumerate(&mut self.constraints) {
            trace!("{gap}Constr[{}]: {} == {}, {}: {}", i, constraint.left.pos, constraint.right.pos, constraint.msg, constraint);
            constraints.1.push(constraint.clone())
        }
    }

    pub fn current_class(&self) -> Option<StringName> {
        self.constraints.last().map(|constraints| {
            constraints.0.last()
        }).flatten().cloned()
    }

    pub fn all_constr(self) -> Vec<Constraints> {
        let (mut finished, constraints) = (self.finished, self.constraints);
        let mut constraints = constraints.into_iter().map(|(n, n1, _)| (n, n1)).collect();

        finished.append(&mut constraints);
        finished.iter().map(Constraints::from).collect()
    }
}

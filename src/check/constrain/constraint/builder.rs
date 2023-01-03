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
    finished: Vec<(Vec<StringName>, Vec<Constraint>)>,
    constraints: Vec<(Vec<StringName>, Vec<Constraint>)>,

    pub var_mapping: VarMapping,
}

impl ConstrBuilder {
    pub fn new() -> ConstrBuilder {
        ConstrBuilder { finished: vec![], constraints: vec![(vec![], vec![])], var_mapping: HashMap::new() }
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
        self.constraints.len()
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

        for i in (level - 1..self.constraints.len()).rev() {
            // Equivalent to pop, but remove has better panic message for debugging
            self.finished.push(self.constraints.remove(i))
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

    #[allow(clippy::map_flatten)] // flat_map does something else for Option here
    pub fn current_class(&self) -> Option<StringName> {
        self.constraints.iter().last().map(|(names, _)| names.iter().last()).flatten().cloned()
    }

    pub fn all_constr(self) -> Vec<Constraints> {
        let (mut finished, mut constraints) = (self.finished, self.constraints);
        finished.append(&mut constraints);
        finished.iter().map(Constraints::from).collect()
    }
}

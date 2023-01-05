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
    constraints: Vec<Vec<(Constraint, usize)>>,
    pub var_mapping: VarMapping,
}

impl ConstrBuilder {
    /// Create constraint builder with a single set.
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

    /// Get marker of current level without creating a new set.
    pub fn level(&self) -> usize { self.constraints.len() - 1 }

    /// Create new set, and get bookmark of previous set.
    pub fn new_set(&mut self) -> usize {
        let inherited_constraints = self.constraints.last().expect("Can never be empty");
        self.constraints.push(inherited_constraints.clone());

        trace!("Created set at level {}", self.constraints.len() - 1);
        self.constraints.len()
    }

    /// Create new set starting at stated level.
    ///
    /// We use this if we want to add to a new set without adding to a certain set of previous ones.
    /// Typically in match arms or if arms, where we want branches to be disjoint.
    /// At the same time, we want all branches to inherit from an older set.
    /// When inheriting, we also discard any constraints added while in a level we wish to skip.
    pub fn new_set_from(&mut self, level: usize) -> usize {
        self.exit_set_to(level);
        self.new_set()
    }

    /// Return to specified level.
    pub fn exit_set_to(&mut self, level: usize) {
        if level > self.constraints.len() {
            panic!("Exiting constraint set which doesn't exist\nlevel: {}, constraints: {}, finished: {}",
                   level, self.constraints.len(), self.finished.len());
        }

        for _ in level..self.level() {
            // filter count constr added at level higher than current
            let inherited_constraints: Vec<_> = self.constraints
                .get(level)
                .expect(format!("New set from {level} while self has {} sets", self.constraints.len()).as_str())
                .iter().cloned()
                .filter(|(_, lvl)| *lvl > level)
                .map(|(c, _)| c)
                .collect();

            self.finished.push(inherited_constraints)
        }
    }

    /// Add new constraint to constraint builder with a message.
    pub fn add(&mut self, msg: &str, parent: &Expected, child: &Expected) {
        self.add_constr(&Constraint::new(msg, parent, child));
    }

    /// Add constraint to currently all op sets.
    /// The open sets are the sets at levels between the self.level and active ceiling.
    pub fn add_constr(&mut self, constraint: &Constraint) {
        let lvl = self.level();
        for constraints in &mut self.constraints {
            constraints.push((constraint.clone(), lvl));
        }

        let lvls = comma_delm(0..=lvl);
        trace!("Constr[{}]: {} == {}, {}: {}", lvls, constraint.left.pos, constraint.right.pos, constraint.msg, constraint);
    }

    pub fn all_constr(self) -> Vec<Constraints> {
        let mut finished = self.finished;
        let mut constraints = self.constraints.into_iter().map(|constraints| {
            constraints.into_iter().map(|(constraint, _)| constraint).collect()
        }).collect();

        finished.append(&mut constraints);
        finished.iter().map(Constraints::from).collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::check::constrain::constraint::builder::ConstrBuilder;
    use crate::check::constrain::constraint::Constraint;
    use crate::check::constrain::constraint::expected::{Expect, Expected};
    use crate::check::name::Any;
    use crate::common::position::Position;

    macro_rules! constr {
        ($msg:expr) => {{
            Constraint::new(format!("{}", $msg).as_str(),
                            &Expected::new(Position::default(), &Expect::any()),
                            &Expected::new(Position::default(), &Expect::any()))
        }}
    }

    macro_rules! assert_eq_constr {
        ($left:expr, $right:expr) => {{
            let left = $left.iter().map(|c| c.msg.clone()).collect::<Vec<_>>();
            let right = $right.iter().map(|c| c.msg.clone()).collect::<Vec<_>>();
            assert_eq!(left, right);
        }}
    }

    #[test]
    fn all_constr_present() {
        let mut builder = ConstrBuilder::new();
        let (c1, c2, c3) = (constr!(1), constr!(2), constr!(3));

        builder.add_constr(&c1);
        builder.add_constr(&c2);
        builder.add_constr(&c3);

        let all_constr = builder.all_constr();
        assert_eq!(all_constr.len(), 1);

        assert_eq!(all_constr[0].constraints, vec![c1, c2, c3])
    }

    #[test]
    fn disjoint_sets() {
        let mut builder = ConstrBuilder::new();
        let (c1, c2, c3, c4) = (constr!(1), constr!(2), constr!(3), constr!(4));

        builder.add_constr(&c1); // anything before if branches (including cond)

        let old_set = builder.level();
        builder.add_constr(&c2); // then branch of if

        builder.new_set_from(old_set);
        builder.add_constr(&c3); // else branch of if

        builder.add_constr(&c4); // anything after if

        let all_constr = builder.all_constr();
        assert_eq!(all_constr.len(), 2);

        assert_eq_constr!(all_constr[0].constraints, [&c1, &c2, &c4]);
        assert_eq_constr!(all_constr[1].constraints, [&c1, &c3, &c4]);
    }
}

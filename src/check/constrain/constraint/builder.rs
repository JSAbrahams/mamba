use std::collections::HashMap;

use itertools::enumerate;

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

/// The constraint builder allows us to build sets of constraints.
///
/// This allows us to constrain different parts of the program which may rely on
/// the same logic, without interfering with each other. E.g. different
/// functions within the same class.
///
/// We use sets to type check all possible execution paths.
/// We can have multiple sets open at a time.
#[derive(Debug)]
pub struct ConstrBuilder {
    constraints: Vec<Vec<(Constraint, usize)>>,
    branch_point: usize,
    joined: bool,

    pub var_mapping: VarMapping,
}

impl ConstrBuilder {
    /// Create constraint builder with a single set present.
    pub fn new() -> ConstrBuilder {
        ConstrBuilder { branch_point: 0, joined: false, constraints: vec![vec![]], var_mapping: HashMap::new() }
    }

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

    /// Set new branch point
    pub fn branch_point(&mut self) {
        self.branch_point += 1;
        self.joined = false;
    }

    /// Create new set starting at stated level.
    ///
    /// We use this if we want to add to a new set without adding to a certain set of previous ones.
    /// Typically in match arms or if arms, where we want branches to be disjoint.
    /// At the same time, we want all branches to inherit from an older set.
    /// When inheriting, we also discard any constraints added while in a level we wish to skip.
    pub fn branch(&mut self) {
        let inherited_constraints: Vec<(Constraint, usize)> = if self.joined {
            self.constraints.last().expect("Is never empty").to_vec()
        } else {
            self.constraints.last().expect("Is never empty")
                .into_iter().filter(|(_, lvl)| *lvl < self.branch_point).cloned().collect()
        };

        self.constraints.push(inherited_constraints.clone());
    }

    /// Reset all branches so that they are again all added to.
    pub fn reset_branches(&mut self) {
        self.branch_point = self.constraints.len() - 1;
        self.joined = true;
    }

    /// Add new constraint to constraint builder with a message.
    pub fn add(&mut self, msg: &str, parent: &Expected, child: &Expected) {
        self.add_constr(&Constraint::new(msg, parent, child));
    }

    /// Add constraint up to last branch point, and latest branch
    pub fn add_constr(&mut self, constraint: &Constraint) {
        let mut lvls = vec![];
        let last_branch = self.constraints.len() - 1;
        if self.joined {
            for (i, constraints) in enumerate(&mut self.constraints) {
                constraints.push((constraint.clone(), self.branch_point));
                lvls.push(i);
            }
        } else {
            // only push to last branch
            self.constraints[last_branch].push((constraint.clone(), self.branch_point));
            lvls.push(last_branch);
        }

        let lvls = comma_delm(lvls);
        trace!("Constr[{}]: {} == {}, {}: {}", lvls, constraint.left.pos, constraint.right.pos, constraint.msg, constraint);
    }

    pub fn all_constr(self) -> Vec<Constraints> {
        let constraints: Vec<_> = self.constraints.into_iter()
            .map(|constraints| constraints.iter().map(|(c, _)| c.clone()).collect())
            .collect();

        constraints.iter().map(Constraints::from).collect()
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
        assert_eq_constr!(all_constr[0].constraints, vec![c1, c2, c3])
    }

    #[test]
    fn disjoint_set_if() {
        let mut builder = ConstrBuilder::new();
        let (c1, c2, c3, c4) = (constr!(1), constr!(2), constr!(3), constr!(4));

        builder.add_constr(&c1); // anything before if branches (including cond)

        builder.branch_point();
        builder.add_constr(&c2); // then branch of if

        builder.branch();
        builder.add_constr(&c3); // else branch of if

        builder.reset_branches();
        builder.add_constr(&c4); // anything after if

        let all_constr = builder.all_constr();
        assert_eq!(all_constr.len(), 2);

        assert_eq_constr!(all_constr[0].constraints, [&c1, &c2, &c4]);
        assert_eq_constr!(all_constr[1].constraints, [&c1, &c3, &c4]);
    }

    #[test]
    fn disjoint_set_match() {
        let mut builder = ConstrBuilder::new();
        let (c1, c2, c3, c4, c5) = (constr!(1), constr!(2), constr!(3), constr!(4), constr!(5));

        builder.add_constr(&c1); // anything before match branches (including expr)

        builder.branch_point();
        builder.add_constr(&c2); // first branch

        builder.branch();
        builder.add_constr(&c3); // second branch

        builder.branch();
        builder.add_constr(&c4); // third branch

        builder.reset_branches();
        builder.add_constr(&c5); // anything after match

        let all_constr = builder.all_constr();
        assert_eq!(all_constr.len(), 3);

        assert_eq_constr!(all_constr[0].constraints, [&c1, &c2, &c5]);
        assert_eq_constr!(all_constr[1].constraints, [&c1, &c3, &c5]);
        assert_eq_constr!(all_constr[2].constraints, [&c1, &c4, &c5]);
    }

    #[test]
    fn disjoint_set_nested_match() {
        let mut builder = ConstrBuilder::new();
        let (c1, c2, _, c4, c5) = (constr!(1), constr!(2), constr!(3), constr!(4), constr!(5));
        let (c31, c32, c33, c34, c35) = (constr!(31), constr!(32), constr!(33), constr!(34), constr!(35));

        builder.add_constr(&c1); // anything before match branches (including expr)

        builder.branch_point();
        builder.add_constr(&c2); // first branch

        builder.branch();
        {   // second branch
            builder.branch_point();
            builder.add_constr(&c31);

            builder.branch();
            builder.add_constr(&c32);

            builder.branch();
            builder.add_constr(&c33);

            builder.branch();
            builder.add_constr(&c34);

            builder.branch();
            builder.add_constr(&c35);
        }

        builder.branch();
        builder.add_constr(&c4); // third branch

        builder.reset_branches();
        builder.add_constr(&c5); // anything after match

        let all_constr = builder.all_constr();
        assert_eq!(all_constr.len(), 7);

        assert_eq_constr!(all_constr[0].constraints, [&c1, &c2, &c5]);

        assert_eq_constr!(all_constr[1].constraints, [&c1, &c31, &c5]);
        assert_eq_constr!(all_constr[2].constraints, [&c1, &c32, &c5]);
        assert_eq_constr!(all_constr[3].constraints, [&c1, &c33, &c5]);
        assert_eq_constr!(all_constr[4].constraints, [&c1, &c34, &c5]);
        assert_eq_constr!(all_constr[5].constraints, [&c1, &c35, &c5]);

        assert_eq_constr!(all_constr[6].constraints, [&c1, &c4, &c5]);
    }
}

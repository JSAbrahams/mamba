use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

use itertools::Itertools;

use crate::check::context::function::Function;
use crate::check::result::TypeResult;
use crate::common::delimit::comma_delm;
use crate::common::position::Position;
use crate::TypeErr;

#[derive(Debug, Eq)]
pub struct FunUnion {
    pub union: HashSet<Function>,
}

impl PartialEq for FunUnion {
    fn eq(&self, other: &Self) -> bool {
        self.union.clone().iter().sorted_by_key(|f| f.name.clone()).collect::<Vec<&Function>>()
            == other.union.clone().iter().sorted_by_key(|f| f.name.clone()).collect::<Vec<&Function>>()
    }
}

impl Hash for FunUnion {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.union.iter().sorted_by_key(|f| &f.name).for_each(|f| f.hash(state))
    }
}

impl From<&HashSet<Function>> for FunUnion {
    fn from(fun_set: &HashSet<Function>) -> Self {
        FunUnion { union: fun_set.clone() }
    }
}

impl From<&HashSet<FunUnion>> for FunUnion {
    fn from(fun_set: &HashSet<FunUnion>) -> Self {
        FunUnion { union: fun_set.iter().flat_map(|f| f.union.clone()).collect() }
    }
}

impl Display for FunUnion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{{}}}", comma_delm(&self.union))
    }
}

impl FunUnion {
    pub fn as_direct(&self, pos: Position) -> TypeResult<Function> {
        if self.union.len() == (1_usize) {
            Ok(self.union.iter().next().unwrap().clone())
        } else {
            let msg = format!("Expected single function but was {}", &self);
            Err(vec![TypeErr::new(pos, &msg)])
        }
    }
}

use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

use itertools::Itertools;

use crate::check::context::field::Field;
use crate::check::result::TypeResult;
use crate::common::delimit::comma_delm;
use crate::common::position::Position;
use crate::TypeErr;

#[derive(Debug, Eq)]
pub struct FieldUnion {
    pub union: HashSet<Field>,
}

impl PartialEq for FieldUnion {
    fn eq(&self, other: &Self) -> bool {
        self.union.clone().into_iter().sorted_by_key(|f| f.name.clone()).collect::<Vec<Field>>()
            == other.union.clone().into_iter().sorted_by_key(|f| f.name.clone()).collect::<Vec<Field>>()
    }
}

impl Hash for FieldUnion {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.union.iter().sorted_by_key(|f| &f.name).for_each(|f| f.hash(state))
    }
}

impl From<&HashSet<Field>> for FieldUnion {
    fn from(field_set: &HashSet<Field>) -> Self {
        FieldUnion { union: field_set.clone() }
    }
}

impl From<&HashSet<FieldUnion>> for FieldUnion {
    fn from(field_set: &HashSet<FieldUnion>) -> Self {
        FieldUnion { union: field_set.iter().flat_map(|f| f.union.clone()).collect() }
    }
}

impl Display for FieldUnion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{{}}}", comma_delm(&self.union))
    }
}

impl FieldUnion {
    pub fn as_direct(&self, pos: Position) -> TypeResult<Field> {
        if self.union.len() == (1_usize) {
            Ok(self.union.iter().next().unwrap().clone())
        } else {
            let msg = format!("Expected single field but was {}", &self);
            Err(vec![TypeErr::new(pos, &msg)])
        }
    }
}

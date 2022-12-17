use crate::check::ast::pos_name::PosNameMap;
use crate::check::name::{Any, Name, Union};
use crate::common::position::Position;

#[derive(Debug, Clone)]
pub struct Finished {
    pub(crate) pos_to_name: PosNameMap,
}

impl Finished {
    pub fn new() -> Finished {
        Finished { pos_to_name: PosNameMap::default() }
    }

    /// Push name associated with specific position in [AST].
    ///
    /// If already present at position, then union is created between current [Name] and given
    /// [Name].
    /// Ignores [Any] type, and trims from union.
    pub fn push_ty(&mut self, pos: Position, name: &Name) {
        if *name == Name::any() {
            return;
        }
        let name = name.trim_any();

        let name = self.pos_to_name.get(&pos).map_or(name.clone(), |s_name| s_name.union(&name));
        if self.pos_to_name.insert(pos, name.clone()).is_none() {
            trace!("{:width$}type at {}: {}", "", pos, name, width = 0);
        }
    }
}

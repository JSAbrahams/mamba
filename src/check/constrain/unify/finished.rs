use crate::check::ast::pos_name::PosNameMap;
use crate::check::context::{Context, LookupClass};
use crate::check::name::{Empty, Name, Union};
use crate::check::result::TypeResult;
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
    pub fn push_ty(&mut self, ctx: &Context, pos: Position, name: &Name) -> TypeResult<()> {
        // trim temp should not be needed, underlying issue with current logic
        let name = name.trim_any().trim_temp();
        if name == Name::empty() {
            return Ok(());
        }
        for class in &name.names {
            ctx.class(class, pos)?;
        }

        let name = self.pos_to_name.get(&pos)
            .map_or(name.clone(), |old_name| if old_name.is_interchangeable {
                old_name.clone()
            } else {
                old_name.union(&name)
            });

        if self.pos_to_name.insert(pos, name.clone()).is_none() {
            trace!("{:width$}type at {}: {}", "", pos, name, width = 0);
        }
        Ok(())
    }
}

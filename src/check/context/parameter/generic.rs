use std::convert::TryFrom;
use std::fmt::{Display, Error, Formatter};

use crate::check::context::name::stringname::StringName;
use crate::check::context::name::truename::TrueName;
use crate::check::result::{TypeErr, TypeResult};
use crate::parse::ast::{AST, Node};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct GenericParameter {
    pub is_py_type: bool,
    pub name: StringName,
    pub parent: Option<TrueName>,
}

impl Display for GenericParameter {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(
            f,
            "{}{}",
            self.name,
            if let Some(parent) = &self.parent {
                format!(" isa {}", parent)
            } else {
                String::new()
            }
        )
    }
}

impl TryFrom<&AST> for GenericParameter {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &AST) -> TypeResult<GenericParameter> {
        match &ast.node {
            Node::Generic { id, isa } => {
                let name = StringName::try_from(id)?;
                let parent = if let Some(isa) = isa { Some(TrueName::try_from(isa)?) } else { None };
                Ok(GenericParameter { is_py_type: false, name, parent })
            }
            _ => Err(vec![TypeErr::new(&ast.pos.clone(), "Expected generic")])
        }
    }
}

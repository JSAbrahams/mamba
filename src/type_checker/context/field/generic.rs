use std::convert::TryFrom;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::checker_result::{TypeErr, TypeResult};
use crate::type_checker::environment::name::{match_type, Identifier};
use crate::type_checker::ty_name::TypeName;
use std::collections::HashSet;

#[derive(Debug, Clone, Eq)]
pub struct GenericField {
    pub is_py_type: bool,
    pub name:       String,
    pub pos:        Position,
    pub private:    bool,
    pub mutable:    bool,
    pub in_class:   Option<TypeName>,
    pub type_name:  Option<TypeName>
}

pub struct GenericFields {
    pub fields: HashSet<GenericField>
}

impl PartialEq for GenericField {
    fn eq(&self, other: &Self) -> bool { self.name == other.name }
}

impl Hash for GenericField {
    fn hash<H: Hasher>(&self, state: &mut H) { self.name.hash(state) }
}

impl TryFrom<&AST> for GenericField {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &AST) -> TypeResult<GenericField> {
        match &ast.node {
            // TODO do something with forward
            Node::VariableDef { private, var, mutable, ty, .. } => {
                let name = field_name(var.deref())?;
                let type_name = match ty {
                    Some(ty) => Some(TypeName::try_from(ty.deref())?),
                    None => None
                };

                let pos = ast.pos.clone();
                Ok(GenericField {
                    is_py_type: false,
                    name,
                    mutable: *mutable,
                    pos,
                    in_class: None,
                    private: *private,
                    type_name
                })
            }
            _ => Err(vec![TypeErr::new(&ast.pos, "Expected variable")])
        }
    }
}

impl TryFrom<&AST> for GenericFields {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &AST) -> TypeResult<GenericFields> {
        Ok(GenericFields {
            fields: match &ast.node {
                // TODO do something with forward
                Node::VariableDef { private, var, ty, mutable, .. } => {
                    let identifier = Identifier::try_from(var.deref())?;
                    // TODO infer type if not present
                    match &ty {
                        Some(ty) => {
                            let type_name = TypeName::try_from(ty.deref())?;
                            Ok(match_type(&identifier, &type_name, &ast.pos)?
                                .iter()
                                .map(|(id, (inner_mut, type_name))| GenericField {
                                    is_py_type: false,
                                    name:       id.clone(),
                                    mutable:    *mutable || *inner_mut,
                                    pos:        ast.pos.clone(),
                                    private:    *private,
                                    type_name:  Some(type_name.clone()),
                                    in_class:   None
                                })
                                .collect())
                        }
                        None => Ok(identifier
                            .fields()
                            .iter()
                            .map(|(inner_mut, id)| GenericField {
                                is_py_type: false,
                                name:       id.clone(),
                                pos:        ast.pos.clone(),
                                private:    *private,
                                mutable:    *mutable || *inner_mut,
                                in_class:   None,
                                type_name:  None
                            })
                            .collect())
                    }
                }
                _ => Err(vec![TypeErr::new(&ast.pos, "Expected variable")])
            }?
        })
    }
}

impl GenericField {
    pub fn in_class(
        self,
        class: Option<&TypeName>,
        type_def: bool,
        pos: &Position
    ) -> TypeResult<GenericField> {
        if self.private && type_def {
            Err(vec![TypeErr::new(
                pos,
                &format!("Field {} cannot be private: In an type definition", self.name)
            )])
        } else {
            Ok(GenericField { in_class: class.cloned(), ..self })
        }
    }
}

fn field_name(ast: &AST) -> TypeResult<String> {
    match &ast.node {
        Node::Id { lit } => Ok(lit.clone()),
        _ => Err(vec![TypeErr::new(&ast.pos, "Expected valid identifier")])
    }
}

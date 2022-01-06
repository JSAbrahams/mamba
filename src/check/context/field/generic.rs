use std::collections::HashSet;
use std::convert::TryFrom;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

use crate::check::ident::Identifier;
use crate::check::name::match_name;
use crate::check::name::nameunion::NameUnion;
use crate::check::name::stringname::StringName;
use crate::check::result::{TypeErr, TypeResult};
use crate::common::position::Position;
use crate::parse::ast::{AST, Node};

#[derive(Debug, Clone, Eq)]
pub struct GenericField {
    pub is_py_type: bool,
    pub name: String,
    pub pos: Position,
    pub mutable: bool,
    pub in_class: Option<StringName>,
    pub ty: Option<NameUnion>,
}

pub struct GenericFields {
    pub fields: HashSet<GenericField>,
}

impl Hash for GenericField {
    fn hash<H: Hasher>(&self, state: &mut H) { self.name.hash(state) }
}

impl PartialEq for GenericField {
    fn eq(&self, other: &Self) -> bool { self.name == other.name }
}

impl TryFrom<&AST> for GenericField {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &AST) -> TypeResult<GenericField> {
        match &ast.node {
            // TODO do something with forward
            Node::VariableDef { var, mutable, ty, .. } => Ok(GenericField {
                is_py_type: false,
                name: field_name(var.deref())?,
                mutable: *mutable,
                pos: ast.pos.clone(),
                in_class: None,
                ty: match ty {
                    Some(ty) => Some(NameUnion::try_from(ty.deref())?),
                    None => None
                },
            }),
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
                Node::VariableDef { var, ty, mutable, .. } => {
                    let identifier = Identifier::try_from(var.deref())?;
                    // TODO infer type if not present
                    match &ty {
                        Some(ty) => {
                            let ty = NameUnion::try_from(ty.deref())?;
                            Ok(match_name(&identifier, &ty, &ast.pos)?
                                .iter()
                                .map(|(id, (inner_mut, ty))| GenericField {
                                    is_py_type: false,
                                    name: id.clone(),
                                    mutable: *mutable || *inner_mut,
                                    pos: ast.pos.clone(),
                                    ty: Some(ty.clone()),
                                    in_class: None,
                                })
                                .collect())
                        }
                        None => Ok(identifier
                            .fields()
                            .iter()
                            .map(|(inner_mut, id)| GenericField {
                                is_py_type: false,
                                name: id.clone(),
                                pos: ast.pos.clone(),
                                mutable: *mutable || *inner_mut,
                                in_class: None,
                                ty: None,
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
        class: Option<&StringName>,
        _type_def: bool,
        _pos: &Position,
    ) -> TypeResult<GenericField> {
        Ok(GenericField { in_class: class.cloned(), ..self })
    }
}

fn field_name(ast: &AST) -> TypeResult<String> {
    match &ast.node {
        Node::Id { lit } => Ok(lit.clone()),
        _ => Err(vec![TypeErr::new(&ast.pos, "Expected valid identifier")])
    }
}

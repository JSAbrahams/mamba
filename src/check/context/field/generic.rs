use std::collections::HashSet;
use std::convert::TryFrom;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

use crate::check::ident::Identifier;
use crate::check::name::match_name;
use crate::check::name::Name;
use crate::check::name::namevariant::NameVariant;
use crate::check::name::stringname::StringName;
use crate::check::name::truename::TrueName;
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
    pub ty: Option<Name>,
}

pub struct GenericFields {
    pub fields: HashSet<GenericField>,
}

impl Hash for GenericField {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}

impl PartialEq for GenericField {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl TryFrom<&AST> for GenericField {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &AST) -> TypeResult<GenericField> {
        match &ast.node {
            Node::VariableDef { var, mutable, ty, .. } => Ok(GenericField {
                is_py_type: false,
                name: field_name(var.deref())?,
                mutable: *mutable,
                pos: ast.pos.clone(),
                in_class: None,
                ty: match ty {
                    Some(ty) => Some(Name::try_from(ty.deref())?),
                    None => None,
                },
            }),
            _ => Err(vec![TypeErr::new(&ast.pos, "Expected variable")]),
        }
    }
}

impl TryFrom<&AST> for GenericFields {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &AST) -> TypeResult<GenericFields> {
        Ok(GenericFields {
            fields: match &ast.node {
                Node::VariableDef { var, ty, mutable, .. } => {
                    let identifier = Identifier::try_from(var.deref())?;
                    match &ty {
                        Some(ty) => {
                            let ty = Name::try_from(ty.deref())?;
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
                            .fields(&var.pos)?
                            .iter()
                            .map(|(inner_mut, name)| GenericField {
                                is_py_type: false,
                                name: name.clone(),
                                pos: ast.pos.clone(),
                                mutable: *mutable || *inner_mut,
                                in_class: None,
                                ty: None,
                            })
                            .collect()),
                    }
                }
                _ => Err(vec![TypeErr::new(&ast.pos, "Expected variable")]),
            }?,
        })
    }
}

impl GenericField {
    pub fn in_class(
        self,
        class: Option<&TrueName>,
        _type_def: bool,
        pos: &Position,
    ) -> TypeResult<GenericField> {
        if let Some(NameVariant::Single(class)) = class.map(|t| t.variant.clone()) {
            Ok(GenericField { in_class: Some(class), ..self })
        } else {
            Err(Vec::from(TypeErr::new(pos, &String::from("Field must be in class"))))
        }
    }
}

fn field_name(ast: &AST) -> TypeResult<String> {
    match &ast.node {
        Node::Id { lit } => Ok(lit.clone()),
        _ => {
            let msg = format!("Expected valid identifier, was '{}'", ast.node);
            Err(vec![TypeErr::new(&ast.pos, &msg)])
        }
    }
}

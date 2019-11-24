use std::collections::HashSet;
use std::convert::TryFrom;
use std::iter::FromIterator;
use std::ops::Deref;

use python_parser::ast::{Expression, SetItem};

use crate::parser::ast::{Node, AST};
use crate::type_checker::context::field::generic::GenericField;
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::environment::name::{match_type, Identifier};
use crate::type_checker::type_result::{TypeErr, TypeResult};

pub struct GenericFields {
    pub fields: HashSet<GenericField>
}

impl TryFrom<&AST> for GenericFields {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &AST) -> TypeResult<GenericFields> {
        Ok(GenericFields {
            fields: match &ast.node {
                // TODO do something with forward
                Node::VariableDef { private, id_maybe_type, .. } => match &id_maybe_type.node {
                    Node::IdType { _type, .. } => {
                        let identifier = Identifier::try_from(id_maybe_type.deref())?;
                        // TODO infer type if not present
                        match &_type {
                            Some(ty) => {
                                let type_name = TypeName::try_from(ty.deref())?;
                                Ok(match_type(&identifier, &type_name, &ast.pos)?
                                    .iter()
                                    .map(|(id, (mutable, type_name))| GenericField {
                                        is_py_type: false,
                                        name:       id.clone(),
                                        mutable:    *mutable,
                                        pos:        ast.pos.clone(),
                                        private:    *private,
                                        ty:         Some(type_name.clone())
                                    })
                                    .collect())
                            }
                            None => Ok(identifier
                                .fields()
                                .iter()
                                .map(|(mutable, id)| GenericField {
                                    is_py_type: false,
                                    name:       id.clone(),
                                    pos:        ast.pos.clone(),
                                    private:    *private,
                                    mutable:    *mutable,
                                    ty:         None
                                })
                                .collect())
                        }
                    }
                    _ => Err(vec![TypeErr::new(&id_maybe_type.pos, "Expected identifier")])
                },
                _ => Err(vec![TypeErr::new(&ast.pos, "Expected variable")])
            }?
        })
    }
}

impl From<(&Vec<Expression>, &Vec<Vec<Expression>>)> for GenericFields {
    fn from((ids, values): (&Vec<Expression>, &Vec<Vec<Expression>>)) -> GenericFields {
        GenericFields {
            fields: ids
                .iter()
                .zip(values)
                .map(|(id, values)| GenericFields::from((id, values)).fields)
                .flatten()
                .collect()
        }
    }
}

impl From<(&Vec<Expression>, &Vec<Expression>)> for GenericFields {
    fn from((ids, values): (&Vec<Expression>, &Vec<Expression>)) -> GenericFields {
        GenericFields {
            fields: ids
                .iter()
                .zip(values)
                .map(|(id, _)| GenericFields::from(id).fields)
                .flatten()
                .collect()
        }
    }
}

impl From<(&Expression, &Vec<Expression>)> for GenericFields {
    // TODO infer type from values
    fn from((id, _): (&Expression, &Vec<Expression>)) -> GenericFields { GenericFields::from(id) }
}

impl From<&Expression> for GenericFields {
    fn from(id: &Expression) -> GenericFields {
        GenericFields {
            fields: HashSet::from_iter(
                match id {
                    Expression::Name(name) => vec![GenericField {
                        is_py_type: true,
                        name:       name.clone(),
                        pos:        Default::default(),
                        private:    false,
                        mutable:    false,
                        in_class:   None,
                        ty:         None
                    }],
                    Expression::TupleLiteral(items) => items
                        .iter()
                        .filter(|item| if let SetItem::Unique(_) = item { true } else { false })
                        .filter(|item| match &item {
                            SetItem::Star(_) => false,
                            SetItem::Unique(expr) =>
                                if let Expression::Name(_) = expr {
                                    true
                                } else {
                                    false
                                },
                        })
                        .map(|item| match &item {
                            SetItem::Star(_) => unreachable!(),
                            SetItem::Unique(expression) => match expression {
                                Expression::Name(name) => GenericField {
                                    is_py_type: true,
                                    name:       name.clone(),
                                    pos:        Default::default(),
                                    private:    false,
                                    mutable:    false,
                                    in_class:   None,
                                    ty:         None
                                },
                                _ => unreachable!()
                            }
                        })
                        .collect(),
                    _ => vec![]
                }
                .into_iter()
            )
        }
    }
}

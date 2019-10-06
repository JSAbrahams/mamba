use std::convert::TryFrom;
use std::ops::Deref;

use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::context::field::generic::GenericField;
use crate::type_checker::context::type_name::actual::ActualTypeName;
use crate::type_checker::context::type_name::{python, TypeName};
use crate::type_checker::type_result::{TypeErr, TypeResult};

pub const SELF: &'static str = "self";

#[derive(Debug, Clone)]
pub struct ClassArgument {
    pub field:   Option<GenericField>,
    pub fun_arg: GenericFunctionArg
}

#[derive(Debug, Clone)]
pub struct GenericFunctionArg {
    pub is_py_type: bool,
    pub name:       String,
    pub pos:        Position,
    pub vararg:     bool,
    pub mutable:    bool,
    pub ty:         Option<TypeName>
}

impl GenericFunctionArg {
    pub fn in_class(self, class: Option<&ActualTypeName>) -> Result<Self, TypeErr> {
        if class.is_none() && self.name.as_str() == SELF {
            Err(TypeErr::new(&self.pos, "Cannot have self argument outside class"))
        } else if class.is_some() && self.name.as_str() == SELF && self.ty.is_none() {
            Ok(GenericFunctionArg { ty: Some(TypeName::from(class.unwrap())), ..self })
        } else {
            Ok(self)
        }
    }
}

impl TryFrom<&AST> for ClassArgument {
    type Error = Vec<TypeErr>;

    fn try_from(node_pos: &AST) -> TypeResult<ClassArgument> {
        match &node_pos.node {
            Node::VariableDef { id_maybe_type, .. } => Ok(ClassArgument {
                field:   Some(GenericField::try_from(node_pos)?),
                fun_arg: GenericFunctionArg::try_from(&AST {
                    pos:  node_pos.pos.clone(),
                    node: Node::FunArg {
                        vararg:        false,
                        id_maybe_type: id_maybe_type.clone(),
                        default:       None
                    }
                })?
            }),
            Node::FunArg { .. } => Ok(ClassArgument {
                field:   None,
                fun_arg: GenericFunctionArg::try_from(node_pos)?
            }),
            _ => Err(vec![TypeErr::new(&node_pos.pos, "Expected definition or function argument")])
        }
    }
}

impl TryFrom<&AST> for GenericFunctionArg {
    type Error = Vec<TypeErr>;

    fn try_from(node_pos: &AST) -> TypeResult<GenericFunctionArg> {
        match &node_pos.node {
            Node::FunArg { vararg, id_maybe_type, default, .. } => match &id_maybe_type.node {
                Node::IdType { id, mutable, _type } => {
                    let name = argument_name(id.deref())?;
                    Ok(GenericFunctionArg {
                        is_py_type: false,
                        name:       name.clone(),
                        vararg:     *vararg,
                        mutable:    *mutable,
                        pos:        node_pos.pos.clone(),
                        ty:         match _type {
                            Some(_type) => Some(TypeName::try_from(_type.deref())?),
                            None if name.as_str() == SELF => None,
                            None =>
                                if let Some(default) = default {
                                    Some(match &default.deref().node {
                                        Node::Str { .. } => TypeName::from(python::STRING),
                                        Node::Bool { .. } => TypeName::from(python::BOOLEAN),
                                        Node::Int { .. } => TypeName::from(python::INTEGER),
                                        Node::Real { .. } => TypeName::from(python::FLOAT),
                                        // TODO create system for identifying when a enum is an int
                                        // and when it is a float
                                        Node::ENum { .. } => TypeName::from(python::INTEGER),
                                        // TODO create system for inferring types for constructor
                                        // and function calls
                                        _ =>
                                            return Err(vec![TypeErr::new(
                                                &default.pos,
                                                "Can only infer type of literals"
                                            )]),
                                    })
                                } else {
                                    return Err(vec![TypeErr::new(
                                        &id.pos,
                                        "Non-self argument must have type if no inferrable default"
                                    )]);
                                },
                        }
                    })
                }
                _ => Err(vec![TypeErr::new(
                    &id_maybe_type.pos,
                    "Expected function argument identifier (and type)"
                )])
            },
            _ => Err(vec![TypeErr::new(&node_pos.pos, "Expected function argument")])
        }
    }
}

fn argument_name(ast: &AST) -> Result<String, TypeErr> {
    match &ast.node {
        Node::Id { lit } => Ok(lit.clone()),
        Node::_Self => Ok(String::from(SELF)),
        _ => Err(TypeErr::new(&ast.pos, "Expected identifier"))
    }
}

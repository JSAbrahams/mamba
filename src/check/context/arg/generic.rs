use std::convert::TryFrom;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

use crate::check::context::clss;
use crate::check::context::field::generic::GenericField;
use crate::check::name::Name;
use crate::check::name::stringname::StringName;
use crate::check::result::{TypeErr, TypeResult};
use crate::common::position::Position;
use crate::parse::ast::{AST, Node};

pub const SELF: &str = "self";

#[derive(Debug, Clone)]
pub struct ClassArgument {
    pub field: Option<GenericField>,
    pub fun_arg: GenericFunctionArg,
}

#[derive(Debug, Clone, Eq)]
pub struct GenericFunctionArg {
    pub is_py_type: bool,
    pub name: String,
    pub pos: Position,
    pub has_default: bool,
    pub vararg: bool,
    pub mutable: bool,
    pub ty: Option<Name>,
}

impl PartialEq for GenericFunctionArg {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.ty == other.ty && self.vararg == other.vararg
    }
}

impl Hash for GenericFunctionArg {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.ty.hash(state);
        self.vararg.hash(state);
    }
}

impl GenericFunctionArg {
    pub fn in_class(self, class: Option<&StringName>) -> TypeResult<GenericFunctionArg> {
        if self.name.as_str() == SELF {
            if class.is_none() {
                let msg = "Cannot have self argument outside class";
                return Err(vec![TypeErr::new(&self.pos, msg)]);
            }

            if self.ty.is_none() {
                return if let Some(class) = class {
                    Ok(GenericFunctionArg { ty: Some(Name::from(class)), ..self })
                } else {
                    Ok(self)
                };
            }
        }

        Ok(self)
    }
}

impl TryFrom<&AST> for ClassArgument {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &AST) -> TypeResult<ClassArgument> {
        match &ast.node {
            Node::VariableDef { mutable, var, expr: expression, ty, .. } => {
                let fun_arg = GenericFunctionArg {
                    is_py_type: false,
                    name: argument_name(var)?,
                    pos: ast.pos.clone(),
                    has_default: expression.is_some(),
                    vararg: false,
                    mutable: *mutable,
                    ty: if let Some(ty) = ty {
                        Some(Name::try_from(ty)?)
                    } else {
                        None
                    },
                };

                Ok(ClassArgument { field: Some(GenericField::try_from(ast)?), fun_arg })
            }
            Node::FunArg { .. } =>
                Ok(ClassArgument { field: None, fun_arg: GenericFunctionArg::try_from(ast)? }),
            _ => Err(vec![TypeErr::new(&ast.pos, "Expected definition or function argument")])
        }
    }
}

impl TryFrom<&AST> for GenericFunctionArg {
    type Error = Vec<TypeErr>;

    /// Construct FunctionArg from AST.
    fn try_from(ast: &AST) -> TypeResult<GenericFunctionArg> {
        match &ast.node {
            Node::FunArg { vararg, var, mutable, ty, default, .. } => {
                let name = argument_name(var.deref())?;
                Ok(GenericFunctionArg {
                    is_py_type: false,
                    name: name.clone(),
                    has_default: default.is_some(),
                    vararg: *vararg,
                    mutable: *mutable,
                    pos: ast.pos.clone(),
                    ty: match ty {
                        Some(ty) => Some(Name::try_from(ty.deref())?),
                        None if name.as_str() == SELF => None,
                        None => if let Some(default) = default {
                            Some(match &default.deref().node {
                                Node::Str { .. } => Name::from(clss::python::STRING_PRIMITIVE),
                                Node::Bool { .. } => Name::from(clss::python::BOOL_PRIMITIVE),
                                Node::Int { .. } => Name::from(clss::python::INT_PRIMITIVE),
                                Node::Real { .. } => Name::from(clss::python::FLOAT_PRIMITIVE),
                                Node::ENum { .. } => Name::from(clss::python::INT_PRIMITIVE),
                                _ => return Err(vec![TypeErr::new(
                                    &default.pos,
                                    "Can only infer type of literals",
                                )]),
                            })
                        } else {
                            return Err(vec![TypeErr::new(
                                &var.pos,
                                "Non-self argument must have type if no default present",
                            )]);
                        },
                    },
                })
            }
            _ => Err(vec![TypeErr::new(&ast.pos, "Expected function argument")])
        }
    }
}

pub fn argument_name(ast: &AST) -> Result<String, TypeErr> {
    match &ast.node {
        Node::Id { lit } => Ok(lit.clone()),
        Node::_Self => Ok(String::from(SELF)),
        _ => Err(TypeErr::new(&ast.pos, "Expected identifier in argument"))
    }
}

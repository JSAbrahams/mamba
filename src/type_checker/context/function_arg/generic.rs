use std::convert::TryFrom;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::checker_result::{TypeErr, TypeResult};
use crate::type_checker::context::field::generic::GenericField;
use crate::type_checker::context::type_name::python;
use crate::type_checker::ty_name::TypeName;

pub const SELF: &str = "self";

#[derive(Debug, Clone)]
pub struct ClassArgument {
    pub field:   Option<GenericField>,
    pub fun_arg: GenericFunctionArg
}

#[derive(Debug, Clone, Eq)]
pub struct GenericFunctionArg {
    pub is_py_type:  bool,
    pub name:        String,
    pub pos:         Position,
    pub has_default: bool,
    pub vararg:      bool,
    pub mutable:     bool,
    pub ty:          Option<TypeName>
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
    pub fn in_class(
        self,
        class: Option<&TypeName>,
        _: &Position
    ) -> TypeResult<GenericFunctionArg> {
        if class.is_none() && self.name.as_str() == SELF {
            Err(vec![TypeErr::new(&self.pos, "Cannot have self argument outside class")])
        } else if class.is_some() && self.name.as_str() == SELF && self.ty.is_none() {
            Ok(GenericFunctionArg { ty: class.cloned(), ..self })
        // TODO if self has type, check that class is parent of type
        } else {
            Ok(self)
        }
    }
}

impl TryFrom<&AST> for ClassArgument {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &AST) -> TypeResult<ClassArgument> {
        match &ast.node {
            Node::VariableDef { mutable, var, expression, ty, .. } => Ok(ClassArgument {
                field:   Some(GenericField::try_from(ast)?),
                fun_arg: GenericFunctionArg::try_from(&AST {
                    pos:  ast.pos.clone(),
                    node: Node::FunArg {
                        vararg:  false,
                        mutable: *mutable,
                        var:     var.clone(),
                        default: expression.clone(),
                        ty:      ty.clone()
                    }
                })?
            }),
            Node::FunArg { .. } =>
                Ok(ClassArgument { field: None, fun_arg: GenericFunctionArg::try_from(ast)? }),
            _ => Err(vec![TypeErr::new(&ast.pos, "Expected definition or function argument")])
        }
    }
}

impl TryFrom<&AST> for GenericFunctionArg {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &AST) -> TypeResult<GenericFunctionArg> {
        match &ast.node {
            Node::FunArg { vararg, var, mutable, ty, default, .. } => {
                let name = argument_name(var.deref())?;
                Ok(GenericFunctionArg {
                    is_py_type:  false,
                    name:        name.clone(),
                    has_default: default.is_some(),
                    vararg:      *vararg,
                    mutable:     *mutable,
                    pos:         ast.pos.clone(),
                    ty:          match ty {
                        Some(ty) => Some(TypeName::try_from(ty.deref())?),
                        None if name.as_str() == SELF => None,
                        None =>
                            if let Some(default) = default {
                                Some(match &default.deref().node {
                                    Node::Str { .. } => TypeName::from(python::STRING),
                                    Node::Bool { .. } => TypeName::from(python::BOOLEAN),
                                    Node::Int { .. } => TypeName::from(python::INTEGER),
                                    Node::Real { .. } => TypeName::from(python::FLOAT),
                                    Node::ENum { .. } => TypeName::from(python::INTEGER),
                                    _ =>
                                        return Err(vec![TypeErr::new(
                                            &default.pos,
                                            "Can only infer type of literals"
                                        )]),
                                })
                            } else {
                                return Err(vec![TypeErr::new(
                                    &var.pos,
                                    "Non-self argument must have type if no default present"
                                )]);
                            },
                    }
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
        _ => Err(TypeErr::new(&ast.pos, "Expected identifier"))
    }
}

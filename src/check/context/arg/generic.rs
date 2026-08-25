use std::convert::TryFrom;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

use crate::check::context::clss;
use crate::check::context::field::generic::GenericField;
use crate::check::name::string_name::StringName;
use crate::check::name::Name;
use crate::check::result::{TypeErr, TypeResult};
use crate::common::position::Position;
use crate::parse::ast::{Node, AST};

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
    /// Give a bare `self` argument the enclosing class as its type.
    ///
    /// Only ever called on an argument already known to belong to a class (see
    /// `GenericFunction::in_class`), so `self` always has a class to take its type from here.
    pub fn in_class(self, class: &StringName) -> GenericFunctionArg {
        if self.name.as_str() == SELF && self.ty.is_none() {
            GenericFunctionArg {
                ty: Some(Name::from(class)),
                ..self
            }
        } else {
            self
        }
    }
}

impl TryFrom<&AST> for ClassArgument {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &AST) -> TypeResult<ClassArgument> {
        match &ast.node {
            // Class arguments are always fields, stored on `self`.
            Node::FunArg { .. } => Ok(ClassArgument {
                field: Some(GenericField::try_from(ast)?),
                fun_arg: GenericFunctionArg::try_from(ast)?,
            }),
            _ => Err(vec![TypeErr::new(ast.pos, "Expected function argument")]),
        }
    }
}

impl TryFrom<&AST> for GenericFunctionArg {
    type Error = Vec<TypeErr>;

    /// Construct FunctionArg from AST.
    fn try_from(ast: &AST) -> TypeResult<GenericFunctionArg> {
        match &ast.node {
            Node::FunArg {
                vararg,
                var,
                mutable,
                ty,
                default,
                ..
            } => {
                let name = argument_name(var.deref())?;
                Ok(GenericFunctionArg {
                    is_py_type: false,
                    name: name.clone(),
                    has_default: default.is_some(),
                    vararg: *vararg,
                    mutable: *mutable,
                    pos: ast.pos,
                    ty: match ty {
                        Some(ty) => Some(Name::try_from(ty.deref())?),
                        None if name.as_str() == SELF => None,
                        None => {
                            if let Some(default) = default {
                                Some(match &default.deref().node {
                                    Node::Str { .. } => Name::from(clss::STRING),
                                    Node::Id { lit }
                                        if lit.as_str() == "True" || lit.as_str() == "False" =>
                                    {
                                        Name::from(clss::BOOL)
                                    }
                                    Node::Int { .. } => Name::from(clss::INT),
                                    Node::Real { .. } => Name::from(clss::FLOAT),
                                    Node::ENum { .. } => Name::from(clss::INT),
                                    _ => {
                                        return Err(vec![TypeErr::new(
                                            default.pos,
                                            "Can only infer type of literals",
                                        )])
                                    }
                                })
                            } else {
                                return Err(vec![TypeErr::new(
                                    var.pos,
                                    "Non-self argument must have type if no default present",
                                )]);
                            }
                        }
                    },
                })
            }
            _ => Err(vec![TypeErr::new(ast.pos, "Expected function argument")]),
        }
    }
}

pub fn argument_name(ast: &AST) -> TypeResult<String> {
    match &ast.node {
        Node::Id { lit } => Ok(lit.clone()),
        _ => Err(vec![TypeErr::new(
            ast.pos,
            "Expected identifier in argument",
        )]),
    }
}

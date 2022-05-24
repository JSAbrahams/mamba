use std::convert::TryFrom;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

use crate::check::context::arg::generic::GenericFunctionArg;
use crate::check::name::Name;
use crate::check::name::namevariant::NameVariant;
use crate::check::name::stringname::StringName;
use crate::check::name::truename::TrueName;
use crate::check::result::{TypeErr, TypeResult};
use crate::common::position::Position;
use crate::parse::ast::{AST, Node};

#[derive(Debug, Clone, Eq)]
pub struct GenericFunction {
    pub is_py_type: bool,
    pub name: StringName,
    pub pure: bool,
    pub pos: Position,
    pub arguments: Vec<GenericFunctionArg>,
    pub raises: Name,
    pub in_class: Option<StringName>,
    pub ret_ty: Option<Name>,
}

impl Hash for GenericFunction {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.arguments.hash(state);
        self.ret_ty.hash(state);
    }
}

impl PartialEq for GenericFunction {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.arguments == other.arguments && self.ret_ty == other.ret_ty
    }
}

impl GenericFunction {
    #[must_use]
    pub fn pure(self, pure: bool) -> Self { GenericFunction { pure: self.pure || pure, ..self } }

    pub fn in_class(
        self,
        in_class: Option<&TrueName>,
        _type_def: bool,
        pos: &Position,
    ) -> TypeResult<GenericFunction> {
        if let Some(NameVariant::Single(in_class)) = in_class.map(|t| t.variant.clone()) {
            Ok(GenericFunction {
                in_class: Some(in_class.clone()),
                arguments: self
                    .arguments
                    .iter()
                    .map(|arg| arg.clone().in_class(Some(&in_class)))
                    .collect::<Result<_, _>>()?,
                ..self
            })
        } else {
            Err(Vec::from(TypeErr::new(pos, &String::from("Function must be in class."))))
        }
    }
}

impl TryFrom<&AST> for GenericFunction {
    type Error = Vec<TypeErr>;

    /// Build a function signature from a
    /// [AST](crate::parser::ast::AST).
    ///
    /// # Failures
    ///
    /// If [AST](crate::parser::ast::AST)'s node is not the
    /// [FunDef](crate::parser::ast::Node::FunDef) variant of the
    /// [Node](crate::parser::ast::Node).
    fn try_from(ast: &AST) -> TypeResult<GenericFunction> {
        match &ast.node {
            // TODO add generics to function definitions
            Node::FunDef { pure, id, args: fun_args, ret: ret_ty, raises, .. } =>
                Ok(GenericFunction {
                    is_py_type: false,
                    name: function_name(id.deref())?,
                    pure: *pure,
                    pos: ast.pos.clone(),
                    arguments: {
                        let args: Vec<GenericFunctionArg> = fun_args
                            .iter()
                            .map(GenericFunctionArg::try_from)
                            .collect::<Result<_, _>>()?;

                        let mut has_default = false;
                        for arg in args.clone() {
                            if has_default && !arg.has_default {
                                return Err(vec![TypeErr::new(
                                    &arg.pos,
                                    "Cannot have argument with default followed by argument with \
                                     no default.",
                                )]);
                            }
                            has_default = arg.has_default;
                        }

                        args
                    },
                    ret_ty: match ret_ty {
                        Some(ty) => Some(Name::try_from(ty.as_ref())?),
                        None => None
                    },
                    in_class: None,
                    raises: Name::try_from(raises)?,
                }),
            _ => Err(vec![TypeErr::new(&ast.pos, "Expected function definition")])
        }
    }
}

pub fn function_name(ast: &AST) -> TypeResult<StringName> {
    match &ast.node {
        Node::Id { lit } => Ok(StringName::from(lit.as_str())),
        _ => Err(vec![TypeErr::new(&ast.pos, "Expected function truename")])
    }
}

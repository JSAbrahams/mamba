use std::convert::TryFrom;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

use crate::check::context::arg::generic::GenericFunctionArg;
use crate::check::context::function;
use crate::check::result::{TypeErr, TypeResult};
use crate::check::ty::name::actual::ActualTypeName;
use crate::check::ty::name::TypeName;
use crate::common::position::Position;
use crate::parse::ast::{Node, AST};

#[derive(Debug, Clone, Eq)]
pub struct GenericFunction {
    pub is_py_type: bool,
    pub name:       ActualTypeName,
    pub pure:       bool,
    pub private:    bool,
    pub pos:        Position,
    pub arguments:  Vec<GenericFunctionArg>,
    pub raises:     Vec<ActualTypeName>,
    pub in_class:   Option<TypeName>,
    pub ret_ty:     Option<TypeName>
}

impl PartialEq for GenericFunction {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.arguments == other.arguments
    }
}

impl Hash for GenericFunction {
    /// Hash depends on name and arguments.
    ///
    /// This means that we can have multiple functions with the same name within
    /// a type if they have different arguments, even though this isn't
    /// allowed in Python itself.
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.arguments.iter().for_each(|a| a.hash(state));
    }
}

impl GenericFunction {
    pub fn pure(self, pure: bool) -> Self { GenericFunction { pure: self.pure || pure, ..self } }

    pub fn in_class(
        self,
        class: Option<&TypeName>,
        type_def: bool,
        pos: &Position
    ) -> TypeResult<GenericFunction> {
        if self.private && type_def {
            Err(vec![TypeErr::new(
                pos,
                &format!("Function {} cannot be private: In an type definition", self.name)
            )])
        } else {
            Ok(GenericFunction {
                in_class: class.cloned(),
                arguments: self
                    .arguments
                    .iter()
                    .map(|arg| arg.clone().in_class(class, pos))
                    .collect::<Result<_, _>>()?,
                ..self
            })
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
            Node::FunDef { pure, id, fun_args, ret_ty, raises, private, .. } =>
                Ok(GenericFunction {
                    is_py_type: false,
                    name:       function_name(id.deref())?,
                    pure:       *pure,
                    private:    *private,
                    pos:        ast.pos.clone(),
                    arguments:  {
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
                                     no default."
                                )]);
                            }
                            has_default = arg.has_default;
                        }

                        args
                    },
                    ret_ty:     match ret_ty {
                        Some(ty) => Some(TypeName::try_from(ty.as_ref())?),
                        None => None
                    },
                    in_class:   None,
                    raises:     raises
                        .iter()
                        .map(ActualTypeName::try_from)
                        .collect::<Result<_, _>>()?
                }),
            _ => Err(vec![TypeErr::new(&ast.pos, "Expected function definition")])
        }
    }
}

pub fn function_name(ast: &AST) -> TypeResult<ActualTypeName> {
    Ok(ActualTypeName::new(
        match &ast.node {
            Node::Id { lit } => lit.clone(),
            Node::Init => String::from("init"),
            Node::SqrtOp => String::from("sqrt"),
            Node::GeOp => String::from(function::GE),
            Node::LeOp => String::from(function::LE),
            Node::EqOp => String::from(function::EQ),
            Node::AddOp => String::from(function::ADD),
            Node::SubOp => String::from(function::SUB),
            Node::PowOp => String::from(function::POW),
            Node::MulOp => String::from(function::MUL),
            Node::ModOp => String::from(function::MOD),
            Node::DivOp => String::from(function::DIV),
            Node::FDivOp => String::from(function::FDIV),

            _ => return Err(vec![TypeErr::new(&ast.pos, "Expected valid function name")])
        }
        .as_str(),
        &[]
    ))
}

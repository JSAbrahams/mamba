use std::convert::TryFrom;
use std::ops::Deref;

use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::context::function::concrete;
use crate::type_checker::context::function_arg::generic::GenericFunctionArg;
use crate::type_checker::context::type_name::actual::ActualTypeName;
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::type_result::{TypeErr, TypeResult};
use std::hash::{Hash, Hasher};

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
                &format!("Function {} cannot be private: In an interface", self.name)
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
                    name:       function_name(id.deref())?.single(&ast.pos)?,
                    pure:       *pure,
                    private:    *private,
                    pos:        ast.pos.clone(),
                    arguments:  fun_args
                        .iter()
                        .map(GenericFunctionArg::try_from)
                        .collect::<Result<_, _>>()?,
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

fn function_name(ast: &AST) -> TypeResult<TypeName> {
    Ok(TypeName::new(
        match &ast.node {
            Node::Id { lit } => lit.clone(),
            Node::Init => String::from("init"),
            Node::SqrtOp => String::from("sqrt"),
            Node::GeOp => String::from(concrete::GE),
            Node::LeOp => String::from(concrete::LE),
            Node::EqOp => String::from(concrete::EQ),
            Node::AddOp => String::from(concrete::ADD),
            Node::SubOp => String::from(concrete::SUB),
            Node::PowOp => String::from(concrete::POW),
            Node::MulOp => String::from(concrete::MUL),
            Node::ModOp => String::from(concrete::MOD),
            Node::DivOp => String::from(concrete::DIV),
            Node::FDivOp => String::from(concrete::FDIV),

            _ => return Err(vec![TypeErr::new(&ast.pos, "Expected valid function name")])
        }
        .as_str(),
        &[]
    ))
}

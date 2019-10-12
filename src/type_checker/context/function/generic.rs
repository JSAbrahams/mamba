use std::convert::TryFrom;
use std::ops::Deref;

use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::context::function::python;
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
    pub ret_ty:     Option<TypeName>
}

impl PartialEq for GenericFunction {
    fn eq(&self, other: &Self) -> bool { self.name == other.name }
}

impl Hash for GenericFunction {
    fn hash<H: Hasher>(&self, state: &mut H) { self.name.hash(state) }
}

impl GenericFunction {
    pub fn pure(self, pure: bool) -> Self { GenericFunction { pure: self.pure || pure, ..self } }

    pub fn in_class(self, class: Option<&ActualTypeName>) -> TypeResult<GenericFunction> {
        Ok(GenericFunction {
            arguments: self
                .arguments
                .iter()
                .map(|arg| arg.clone().in_class(class))
                .collect::<Result<_, _>>()?,
            ..self
        })
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
    fn try_from(node_pos: &AST) -> TypeResult<GenericFunction> {
        match &node_pos.node {
            // TODO add generics to function definitions
            Node::FunDef { pure, id, fun_args, ret_ty, raises, private, .. } =>
                Ok(GenericFunction {
                    is_py_type: false,
                    name:       function_name(id.deref())?,
                    pure:       *pure,
                    private:    *private,
                    pos:        node_pos.pos.clone(),
                    arguments:  fun_args
                        .iter()
                        .map(GenericFunctionArg::try_from)
                        .collect::<Result<_, _>>()?,
                    ret_ty:     match ret_ty {
                        Some(ty) => Some(TypeName::try_from(ty.as_ref())?),
                        None => None
                    },
                    raises:     raises
                        .iter()
                        .map(ActualTypeName::try_from)
                        .collect::<Result<_, _>>()?
                }),
            _ => Err(vec![TypeErr::new(&node_pos.pos, "Expected function definition")])
        }
    }
}

fn function_name(ast: &AST) -> TypeResult<ActualTypeName> {
    Ok(ActualTypeName::new(
        match &ast.node {
            Node::Id { lit } => lit.clone(),
            Node::Init => String::from("init"),

            Node::GeOp => String::from(python::GE),
            Node::LeOp => String::from(python::LE),
            Node::EqOp => String::from(python::EQ),
            Node::AddOp => String::from(python::ADD),
            Node::SubOp => String::from(python::SUB),
            Node::PowOp => String::from(python::POW),
            Node::MulOp => String::from(python::MUL),
            Node::ModOp => String::from(python::MOD),
            Node::DivOp => String::from(python::DIV),
            Node::FDivOp => String::from(python::FDIV),

            _ => return Err(vec![TypeErr::new(&ast.pos, "Expected valid function name")])
        }
        .as_str(),
        &vec![]
    ))
}
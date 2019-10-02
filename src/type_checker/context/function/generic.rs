use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::context::function::python;
use crate::type_checker::context::function_arg::generic::GenericFunctionArg;
use crate::type_checker::context::type_name::generic::GenericTypeName;
use crate::type_checker::type_result::TypeErr;
use std::convert::TryFrom;

#[derive(Debug, Clone, Eq)]
pub struct GenericFunction {
    pub is_py_type: bool,
    pub name:       GenericTypeName,
    pub pure:       bool,
    pub private:    bool,
    pub pos:        Position,
    pub arguments:  Vec<GenericFunctionArg>,
    pub raises:     Vec<GenericTypeName>,
    ret_ty:         Option<GenericTypeName>
}

impl GenericFunction {
    pub fn pure(self, pure: bool) -> Self {
        GenericFunction {
            is_py_type: self.is_py_type,
            name:       self.name,
            pure:       self.pure || pure,
            private:    self.private,
            pos:        self.pos,
            arguments:  self.arguments,
            raises:     self.raises,
            ret_ty:     self.ret_ty
        }
    }

    pub fn in_class(self, class: Option<&GenericTypeName>) -> Result<Self, TypeErr> {
        Ok(GenericFunction {
            is_py_type: self.is_py_type,
            name:       self.name,
            pure:       self.pure,
            private:    self.private,
            pos:        self.pos,
            arguments:  self
                .arguments
                .iter()
                .map(|arg| arg.clone().in_class(class))
                .collect::<Result<_, _>>()?,
            raises:     self.raises,
            ret_ty:     self.ret_ty
        })
    }
}

impl TryFrom<&AST> for GenericFunction {
    type Error = TypeErr;

    /// Build a function signature from a
    /// [AST](crate::parser::ast::AST).
    ///
    /// # Failures
    ///
    /// If [AST](crate::parser::ast::AST)'s node is not the
    /// [FunDef](crate::parser::ast::Node::FunDef) variant of the
    /// [Node](crate::parser::ast::Node).
    fn try_from(node_pos: &AST) -> Result<Self, Self::Error> {
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
                        Some(ty) => Some(GenericTypeName::try_from(ty.as_ref())?),
                        None => None
                    },
                    raises:     raises
                        .iter()
                        .map(GenericTypeName::try_from)
                        .collect::<Result<_, _>>()?
                }),
            _ => Err(TypeErr::new(&node_pos.pos, "Expected function definition"))
        }
    }
}

fn function_name(ast: &AST) -> Result<GenericTypeName, TypeErr> {
    Ok(GenericTypeName::from(&match &ast.node {
        Node::Id { lit } => Ok(lit.clone()),
        Node::Init => Ok(String::from("init")),

        Node::GeOp => Ok(String::from(python::GE)),
        Node::LeOp => Ok(String::from(python::LE)),
        Node::EqOp => Ok(String::from(python::EQ)),
        Node::AddOp => Ok(String::from(python::ADD)),
        Node::SubOp => Ok(String::from(python::SUB)),
        Node::PowOp => Ok(String::from(python::POW)),
        Node::MulOp => Ok(String::from(python::MUL)),
        Node::ModOp => Ok(String::from(python::MOD)),
        Node::DivOp => Ok(String::from(python::DIV)),
        Node::FDivOp => Ok(String::from(python::FDIV)),

        _ => Err(TypeErr::new(&ast.pos, "Expected valid function name"))
    }?))
}

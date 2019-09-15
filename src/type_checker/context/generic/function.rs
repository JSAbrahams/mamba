use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::context::generic::function_arg::GenericFunctionArg;
use crate::type_checker::context::generic::parameter::GenericParameter;
use crate::type_checker::context::generic::type_name::GenericTypeName;
use crate::type_checker::type_result::TypeErr;
use std::convert::TryFrom;
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct GenericFunction {
    pub name:      String,
    pub pure:      bool,
    pub private:   bool,
    pub pos:       Position,
    pub generics:  Vec<GenericParameter>,
    pub arguments: Vec<GenericFunctionArg>,
    pub raises:    Vec<GenericTypeName>,
    ret_ty:        Option<GenericTypeName>
}

impl GenericFunction {
    pub const ADD: &'static str = "+";
    pub const DIV: &'static str = "/";
    pub const EQ: &'static str = "=";
    pub const FDIV: &'static str = "//";
    pub const GE: &'static str = ">";
    pub const GEQ: &'static str = ">=";
    pub const LE: &'static str = "<";
    pub const LEQ: &'static str = "<=";
    pub const MOD: &'static str = "mod";
    pub const MUL: &'static str = "*";
    pub const NEQ: &'static str = "/=";
    pub const POW: &'static str = "^";
    pub const SUB: &'static str = "-";

    pub fn pure(self, pure: bool) -> Self {
        GenericFunction {
            name:      self.name,
            pure:      self.pure || pure,
            private:   self.private,
            pos:       self.pos,
            generics:  self.generics,
            arguments: self.arguments,
            raises:    self.raises,
            ret_ty:    self.ret_ty
        }
    }

    pub fn in_class(self, class: Option<GenericTypeName>) -> Result<Self, TypeErr> {
        Ok(GenericFunction {
            name:      self.name,
            pure:      self.pure,
            private:   self.private,
            pos:       self.pos,
            generics:  self.generics,
            arguments: self
                .arguments
                .iter()
                .map(|arg| arg.clone().in_class(class.clone()))
                .collect::<Result<_, _>>()?,
            raises:    self.raises,
            ret_ty:    self.ret_ty
        })
    }

    // TODO derive return type during type inference stage
    pub fn ty(&self) -> Result<GenericTypeName, TypeErr> {
        self.ret_ty
            .clone()
            .ok_or_else(|| TypeErr::new(&self.pos.clone(), "Function return type not given"))
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
                    name:      function_name(id.deref())?,
                    pure:      *pure,
                    private:   *private,
                    pos:       node_pos.pos.clone(),
                    generics:  vec![],
                    arguments: fun_args
                        .iter()
                        .map(GenericFunctionArg::try_from)
                        .collect::<Result<_, _>>()?,
                    ret_ty:    match ret_ty {
                        Some(ty) => Some(GenericTypeName::try_from(ty.as_ref())?),
                        None => None
                    },
                    raises:    raises
                        .iter()
                        .map(GenericTypeName::try_from)
                        .collect::<Result<_, _>>()?
                }),
            _ => Err(TypeErr::new(&node_pos.pos, "Expected function definition"))
        }
    }
}

fn function_name(ast: &AST) -> Result<String, TypeErr> {
    match &ast.node {
        Node::Id { lit } => Ok(lit.clone()),
        Node::Init => Ok(String::from("init")),

        Node::GeOp => Ok(String::from(GenericFunction::GE)),
        Node::LeOp => Ok(String::from(GenericFunction::LE)),
        Node::EqOp => Ok(String::from(GenericFunction::EQ)),
        Node::AddOp => Ok(String::from(GenericFunction::ADD)),
        Node::SubOp => Ok(String::from(GenericFunction::SUB)),
        Node::PowOp => Ok(String::from(GenericFunction::POW)),
        Node::MulOp => Ok(String::from(GenericFunction::MUL)),
        Node::ModOp => Ok(String::from(GenericFunction::MOD)),
        Node::DivOp => Ok(String::from(GenericFunction::DIV)),
        Node::FDivOp => Ok(String::from(GenericFunction::FDIV)),

        _ => Err(TypeErr::new(&ast.pos, "Expected valid function name"))
    }
}

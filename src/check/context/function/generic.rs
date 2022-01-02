use std::convert::TryFrom;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

use crate::check::context::arg::generic::GenericFunctionArg;
use crate::check::context::function;
use crate::check::context::name::{DirectName, NameUnion};
use crate::check::result::{TypeErr, TypeResult};
use crate::common::position::Position;
use crate::parse::ast::{Node, AST};

#[derive(Debug, Clone, Eq)]
pub struct GenericFunction {
    pub is_py_type: bool,
    pub name:       DirectName,
    pub pure:       bool,
    pub pos:        Position,
    pub arguments:  Vec<GenericFunctionArg>,
    pub raises:     NameUnion,
    pub in_class:   Option<DirectName>,
    pub ret_ty:     Option<NameUnion>
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
    pub fn pure(self, pure: bool) -> Self { GenericFunction { pure: self.pure || pure, ..self } }

    pub fn in_class(
        self,
        in_class: Option<&DirectName>,
        _type_def: bool
    ) -> TypeResult<GenericFunction> {
        Ok(GenericFunction {
            in_class: in_class.cloned(),
            arguments: self
                .arguments
                .iter()
                .map(|arg| arg.clone().in_class(in_class))
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
    fn try_from(ast: &AST) -> TypeResult<GenericFunction> {
        match &ast.node {
            // TODO add generics to function definitions
            Node::FunDef { pure, id, args: fun_args, ret: ret_ty, raises, .. } =>
                Ok(GenericFunction {
                    is_py_type: false,
                    name:       function_name(id.deref())?,
                    pure:       *pure,
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
                        Some(ty) => Some(NameUnion::try_from(ty.as_ref())?),
                        None => None
                    },
                    in_class:   None,
                    raises:     NameUnion::try_from(raises)?
                }),
            _ => Err(vec![TypeErr::new(&ast.pos, "Expected function definition")])
        }
    }
}

pub fn function_name(ast: &AST) -> TypeResult<DirectName> {
    Ok(DirectName::from(match &ast.node {
        Node::Id { lit } => lit.as_str(),
        Node::Init => "init",
        Node::SqrtOp => "sqrt",
        Node::GeOp => function::GE,
        Node::LeOp => function::LE,
        Node::EqOp => function::EQ,
        Node::AddOp => function::ADD,
        Node::SubOp => function::SUB,
        Node::PowOp => function::POW,
        Node::MulOp => function::MUL,
        Node::ModOp => function::MOD,
        Node::DivOp => function::DIV,
        Node::FDivOp => function::FDIV,
        _ => return Err(vec![TypeErr::new(&ast.pos, "Expected valid function name")])
    }))
}

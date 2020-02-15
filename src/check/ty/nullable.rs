use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Formatter};

use crate::check::context::clss::NONE;
use crate::check::context::name::Name::{Fun, Single, Tuple};
use crate::check::context::name::{Name, NameUnion};
use crate::check::context::Context;
use crate::check::result::{TypeErr, TypeResult};
use crate::check::ty::Type;
use crate::common::delimit::comma_delimited;
use crate::common::position::Position;
use crate::parse::ast::{Node, AST};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct NullableType {
    pub is_nullable: bool,
    pub name:        Name
}

impl Display for NullableType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let display_nullable = self.is_nullable && self.name != Name::from(NONE);
        write!(f, "{}{}", self.name, if display_nullable { "?" } else { "" })
    }
}

impl TryFrom<&AST> for NullableType {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &AST) -> TypeResult<NullableType> {
        match &ast.node {
            Node::QuestionOp { expr } =>
                Ok(NullableType { is_nullable: true, name: Name::try_from(expr)? }),
            _ => Ok(NullableType { is_nullable: false, name: Name::try_from(ast)? })
        }
    }
}

impl NullableType {
    pub fn is_superset(&self, other: &NullableType) -> bool {
        let neither_nullable = !self.is_nullable && !other.is_nullable;
        self.is_nullable || neither_nullable && self.name == other.name
    }

    pub fn function(
        &self,
        name: &Name,
        args: &[Type],
        ctx: &Context,
        pos: &Position
    ) -> TypeResult<(NameUnion, Option<NameUnion>)> {
        match &self.name {
            Single(..) => {
                let class = ctx.lookup_class(&self.name, pos)?;
                let function = class.function(name, pos)?;
                function.args_compatible(args, ctx, pos)?;
                Ok((function.raises, function.ret_ty))
            }
            Tuple(..) => {
                let msg = format!(
                    "Tuple {} does not implement a function {} with arguments ({}).",
                    self,
                    name,
                    comma_delimited(args)
                );
                Err(vec![TypeErr::new(pos, &msg)])
            }
            Fun(..) => {
                let msg = format!(
                    "Anonymous function {} does not implement a function {} with arguments ({}).",
                    self,
                    name,
                    comma_delimited(args)
                );
                Err(vec![TypeErr::new(pos, &msg)])
            }
        }
    }

    pub fn anon_function(
        &self,
        args: &[Type],
        ctx: &Context,
        pos: &Position
    ) -> TypeResult<NameUnion> {
        match &self.name {
            Single(..) => {
                let msg = format!(
                    "A '{}' is not an anonymous function with arguments ({}).",
                    self,
                    comma_delimited(args)
                );
                Err(vec![TypeErr::new(pos, &msg)])
            }
            Tuple(..) => {
                let msg = format!(
                    "Tuple {} is not an anonymous function with arguments ({}).",
                    self,
                    comma_delimited(args)
                );
                Err(vec![TypeErr::new(pos, &msg)])
            }
            Fun(a_args, ret_ty) =>
                if args == a_args {
                    Ok(*ret_ty.clone())
                } else {
                    let msg = format!(
                        "Anonymous function {} does not have arguments ({})",
                        self,
                        comma_delimited(args)
                    );
                    Err(vec![TypeErr::new(pos, &msg)])
                },
        }
    }

    pub fn field(
        &self,
        name: &Name,
        ctx: &Context,
        pos: &Position
    ) -> TypeResult<Option<NameUnion>> {
        match &self.name {
            Single(..) => {
                let field = ctx.lookup_field(name, pos)?;
                Ok(field.ty.clone())
            }
            Tuple(..) => {
                let msg = format!("Tuple {} does not have a field '{}'.", self, name);
                Err(vec![TypeErr::new(pos, &msg)])
            }
            Fun(..) => {
                let msg = format!("Anonymous function {} does not have a field '{}'.", self, name);
                Err(vec![TypeErr::new(pos, &msg)])
            }
        }
    }
}

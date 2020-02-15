use std::ops::Deref;

use crate::check::context::Context;
use crate::check::pass::modification::Modification;
use crate::check::result::TypeResult;
use crate::check::ty::Type;
use crate::parse::ast::{Node, AST};
use std::convert::TryFrom;

pub struct Constructor;

impl Constructor {
    pub fn new() -> Constructor { Constructor {} }
}

impl Modification for Constructor {
    fn modify(&self, ast: &AST, ctx: &Context) -> TypeResult<(AST, bool)> {
        match &ast.node {
            Node::FunctionCall { name, args } => {
                let ty = Type::try_from(name.deref())?;
                let args: Vec<(AST, bool)> =
                    args.iter().map(|arg| self.modify(arg, ctx)).collect::<Result<_, _>>()?;
                let (args, m_args): (Vec<AST>, Vec<bool>) = args.into_iter().unzip();
                let m_args = m_args.iter().any(|b| *b);

                match ctx.lookup_class(&ty, &ast.pos) {
                    Ok(_) => Ok((
                        AST {
                            node: Node::ConstructorCall { name: name.clone(), args },
                            ..ast.clone()
                        },
                        true
                    )),
                    Err(_) => Ok((
                        AST {
                            node: Node::FunctionCall { name: name.clone(), args },
                            ..ast.clone()
                        },
                        m_args
                    ))
                }
            }
            _ => self.recursion(ast, &ctx)
        }
    }
}

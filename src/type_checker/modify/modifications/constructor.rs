use std::ops::Deref;

use crate::parser::ast::{Node, AST};
use crate::type_checker::context::Context;
use crate::type_checker::modify::modifications::Modification;
use crate::type_checker::type_name::TypeName;
use crate::type_checker::type_result::TypeResult;
use std::convert::TryFrom;

pub struct Constructor;

impl Constructor {
    pub fn new() -> Constructor { Constructor {} }
}

impl Modification for Constructor {
    fn modify(&self, ast: &AST, ctx: &Context) -> TypeResult<(AST, bool)> {
        match &ast.node {
            Node::FunctionCall { name, args } => {
                let type_name = TypeName::try_from(name.deref())?;
                let args: Vec<(AST, bool)> =
                    args.iter().map(|arg| self.modify(arg, ctx)).collect::<Result<_, _>>()?;
                let (args, m_args): (Vec<AST>, Vec<bool>) = args.into_iter().unzip();
                let m_args = m_args.iter().any(|b| *b);

                match ctx.lookup(&type_name, &ast.pos) {
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

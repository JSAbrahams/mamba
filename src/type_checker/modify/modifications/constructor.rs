use std::convert::TryFrom;
use std::ops::Deref;

use crate::parser::ast::{Node, AST};
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::context::Context;
use crate::type_checker::modify::modifications::Modification;
use crate::type_checker::type_result::TypeResult;

pub struct Constructor;

impl Constructor {
    pub fn new() -> Constructor { Constructor {} }
}

impl Modification for Constructor {
    fn modify(&self, ast: &AST, ctx: &Context) -> TypeResult<AST> {
        match &ast.node {
            Node::FunctionCall { name, args } => {
                let type_name = TypeName::try_from(name.deref())?;
                let args =
                    args.iter().map(|arg| self.modify(arg, ctx)).collect::<Result<_, _>>()?;

                match ctx.lookup(&type_name, &ast.pos) {
                    Ok(_) => Ok(AST {
                        node: Node::FunctionCall { name: name.clone(), args },
                        ..ast.clone()
                    }),
                    Err(_) => Ok(AST {
                        node: Node::ConstructorCall { name: name.clone(), args },
                        ..ast.clone()
                    })
                }
            }
            _ => Ok(ast.clone())
        }
    }
}

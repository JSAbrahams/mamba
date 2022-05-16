use crate::generate::ast::node::Core;
use crate::generate::convert::common::convert_vec;
use crate::generate::convert::convert_node;
use crate::generate::convert::state::{Imports, State};
use crate::generate::result::GenResult;
use crate::parse::ast::{AST, Node};

pub fn convert_call(ast: &AST, imp: &mut Imports, state: &State) -> GenResult {
    Ok(match &ast.node {
        Node::PropertyCall { instance, property } => Core::PropertyCall {
            object: Box::from(convert_node(instance, imp, state)?),
            property: Box::from(convert_node(property, imp, state)?),
        },
        Node::FunctionCall { name, args } => Core::FunctionCall {
            function: Box::from(convert_node(name, imp, state)?),
            args: convert_vec(args, imp, state)?,
        },
        other => panic!("Expected call flow but was: {:?}.", other)
    })
}

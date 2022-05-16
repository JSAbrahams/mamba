use crate::convert::ast::node::Core;
use crate::convert::desugar::common::desugar_vec;
use crate::convert::desugar::desugar_node;
use crate::convert::desugar::state::{Imports, State};
use crate::convert::result::ConvertResult;
use crate::parse::ast::{AST, Node};

pub fn desugar_call(ast: &AST, imp: &mut Imports, state: &State) -> ConvertResult {
    Ok(match &ast.node {
        Node::PropertyCall { instance, property } => Core::PropertyCall {
            object: Box::from(desugar_node(instance, imp, state)?),
            property: Box::from(desugar_node(property, imp, state)?),
        },
        Node::FunctionCall { name, args } => Core::FunctionCall {
            function: Box::from(desugar_node(name, imp, state)?),
            args: desugar_vec(args, imp, state)?,
        },
        other => panic!("Expected call flow but was: {:?}.", other)
    })
}

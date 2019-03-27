use crate::core::construct::Core;
use crate::desugar::node::desugar_node;
use crate::desugar::util::desugar_vec;
use crate::parser::ast::ASTNode;

pub fn desugar_control_flow(node: &ASTNode) -> Core {
    match node {
        ASTNode::IfElse { cond, then, _else } => match _else {
            Some(_else) => Core::IfElse {
                cond:  desugar_vec(cond),
                then:  Box::from(desugar_node(then)),
                _else: Box::from(desugar_node(_else))
            },
            None => Core::If { cond: desugar_vec(cond), then: Box::from(desugar_node(then)) }
        },
        ASTNode::Match { cond, cases } =>
            Core::Match { cond: desugar_vec(cond), cases: desugar_vec(cases) },
        ASTNode::Case { cond, body } =>
            Core::Case { cond: Box::from(desugar_node(cond)), body: Box::from(desugar_node(body)) },
        ASTNode::While { cond, body } =>
            Core::While { cond: desugar_vec(cond), body: Box::from(desugar_node(body)) },
        ASTNode::For { expr, collection, body } => Core::For {
            exprs:      desugar_vec(expr),
            collection: Box::from(desugar_node(collection)),
            body:       Box::from(desugar_node(body))
        },

        ASTNode::Break => Core::Break,
        ASTNode::Continue => Core::Continue,
        other => panic!("Expected definition but was: {:?}.", other)
    }
}

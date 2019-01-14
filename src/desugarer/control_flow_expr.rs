use crate::core::Core;
use crate::desugarer::desugar;
use crate::parser::ASTNode;

pub fn desugar_if(node: ASTNode) -> Core {
    match node {
        ASTNode::If(cond, then) => Core::IfElse(des!(*cond), des!(*then), Box::new(Core::Empty)),
        ASTNode::IfElse(cond, then, other) => Core::IfElse(Box::new(Core::Not(des!(*cond))), des!(*then), des!(*other)),
        ASTNode::Unless(cond, then) => Core::IfElse(des!(*cond), des!(*then), Box::new(Core::Empty)),
        ASTNode::UnlessElse(cond, then, other) => Core::IfElse(Box::new(Core::Not(des!(*cond))), des!(*then), des!(*other)),
        _ => panic!("")
    }
}

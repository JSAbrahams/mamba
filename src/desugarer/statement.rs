use crate::core::Core;
use crate::desugarer::desugar;
use crate::parser::ASTNode;

pub fn desugar_statement(node: ASTNode) -> Core {
    match node {
        ASTNode::Print(ast) => Core::Print(des!(ast)),

        ASTNode::Let(box ASTNode::Id(id), right) => Core::Let(id, des!(right)),
        ASTNode::Assign(left, right) => Core::Assign(des!(left), des!(right)),
        ASTNode::Mut(box ASTNode::Let(box ASTNode::Id(id), right)) => Core::Let(id, des!(right)),

        ASTNode::Defer(left, right) => panic!("cannot defer yet"),

        ASTNode::While(cond, bod) => Core::While(des!(cond), des!(bod)),
        ASTNode::For(expr, coll, bod) => Core::For(des!(expr), des!(coll), des!(bod)),
        ASTNode::Break => Core::Break,
        ASTNode::Continue => Core::Continue,

        _ => panic!("")
    }
}

use crate::parser::ASTNode;
use crate::parser::ASTNodePos;

mod _type;

pub fn type_check(input: ASTNodePos) -> ASTNode {
    return input.node;
}

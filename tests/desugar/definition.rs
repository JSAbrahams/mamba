use mamba::core::construct::Core;
use mamba::desugar::desugar;
use mamba::parser::ast::ASTNode;
use mamba::parser::ast::ASTNodePos;

#[test]
fn reassign_verify() {
    let left = to_pos!(ASTNode::Id { lit: String::from("something") });
    let right = to_pos!(ASTNode::Id { lit: String::from("other") });
    let reassign = to_pos!(ASTNode::Reassign { left, right });

    let (left, right) = match desugar(&reassign) {
        Core::Assign { left, right } => (left, right),
        other => panic!("Expected reassign but was {:?}", other)
    };

    assert_eq!(*left, Core::Id { lit: String::from("something") });
    assert_eq!(*right, Core::Id { lit: String::from("other") });
}

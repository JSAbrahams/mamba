use mamba::common::position::EndPoint;
use mamba::common::position::Position;
use mamba::core::construct::Core;
use mamba::desugar::desugar;
use mamba::parser::ast::ASTNode;
use mamba::parser::ast::ASTNodePos;

#[test]
fn type_alias_empty_verify() {
    let type_def = to_pos!(ASTNode::TypeAlias {
        _type:      Box::from(to_pos!(ASTNode::Pass)),
        conditions: vec![]
    });
    assert_eq!(desugar(&type_def).unwrap(), Core::Empty);
}

#[test]
fn type_tup_empty_verify() {
    let type_def = to_pos!(ASTNode::TypeTup { types: vec![] });
    assert_eq!(desugar(&type_def).unwrap(), Core::Empty);
}

#[test]
fn type_fun_empty_verify() {
    let type_def = to_pos!(ASTNode::TypeFun {
        args:   vec![to_pos_unboxed!(ASTNode::Pass)],
        ret_ty: Box::from(to_pos!(ASTNode::Pass))
    });
    assert_eq!(desugar(&type_def).unwrap(), Core::Empty);
}

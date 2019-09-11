use mamba::common::position::EndPoint;
use mamba::common::position::Position;
use mamba::core::construct::Core;
use mamba::desugar::desugar;
use mamba::parser::ast::Node;
use mamba::parser::ast::AST;

#[test]
fn type_alias_empty_verify() {
    let type_def =
        to_pos!(Node::TypeAlias { _type: Box::from(to_pos!(Node::Pass)), conditions: vec![] });
    assert_eq!(desugar(&type_def).unwrap(), Core::Empty);
}

#[test]
fn type_tup_empty_verify() {
    let type_def = to_pos!(Node::TypeTup { types: vec![] });
    assert_eq!(desugar(&type_def).unwrap(), Core::Empty);
}

#[test]
fn type_fun_empty_verify() {
    let type_def = to_pos!(Node::TypeFun {
        args:   vec![to_pos_unboxed!(Node::Pass)],
        ret_ty: Box::from(to_pos!(Node::Pass))
    });
    assert_eq!(desugar(&type_def).unwrap(), Core::Empty);
}

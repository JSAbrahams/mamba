use mamba::core::construct::Core;
use mamba::desugar::desugar;
use mamba::parser::ast::ASTNode;
use mamba::parser::ast::ASTNodePos;
use std::panic;

#[test]
fn handle_empty_verify() {
    let expr_or_stmt = to_pos!(ASTNode::Id { lit: String::from("my_fun") });
    let handle = to_pos!(ASTNode::Handle { expr_or_stmt, cases: vec![] });

    let (_try, except) = match desugar(&handle) {
        Core::TryExcept { _try, except } => (_try, except),
        other => panic!("Expected reassign but was {:?}", other)
    };

    assert_eq!(*_try, Core::Id { lit: String::from("my_fun") });
    assert!(except.is_empty());
}

#[test]
fn handle_verify() {
    let expr_or_stmt = to_pos!(ASTNode::Id { lit: String::from("my_fun") });
    let cond = to_pos!(ASTNode::IdType {
        id:      to_pos!(ASTNode::Id { lit: String::from("err") }),
        mutable: false,
        _type:   Some(to_pos!(ASTNode::Type {
            id:       to_pos!(ASTNode::Id { lit: String::from("my_type") }),
            generics: vec![]
        }))
    });
    let body = to_pos!(ASTNode::Int { lit: String::from("9999") });
    let case = to_pos_unboxed!(ASTNode::Case { cond, body });
    let handle = to_pos!(ASTNode::Handle { expr_or_stmt, cases: vec![case] });

    let (_try, except) = match desugar(&handle) {
        Core::TryExcept { _try, except } => (_try, except),
        other => panic!("Expected reassign but was {:?}", other)
    };

    assert_eq!(*_try, Core::Id { lit: String::from("my_fun") });
    assert_eq!(except.len(), 1);
    match &except[0] {
        Core::Except { id, class, body } => {
            assert_eq!(**id, Core::Id { lit: String::from("err") });
            assert_eq!(**class, Core::Id { lit: String::from("my_type") });
            assert_eq!(**body, Core::Int { int: String::from("9999") });
        }
        other => panic!("Expected except case but was {:?}", other)
    }
}

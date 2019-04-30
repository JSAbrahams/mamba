use mamba::core::construct::Core;
use mamba::desugar::desugar;
use mamba::parser::ast::ASTNode;
use mamba::parser::ast::ASTNodePos;

#[test]
fn if_verify() {
    let cond = to_pos_unboxed!(ASTNode::Id { lit: String::from("cond") });
    let then = to_pos!(ASTNode::Id { lit: String::from("then") });
    let if_stmt = to_pos!(ASTNode::IfElse { cond: vec![cond], then, _else: None });

    let (core_cond, core_then) = match desugar(&if_stmt) {
        Core::If { cond, then } => (cond, then),
        other => panic!("Expected reassign but was {:?}", other)
    };

    assert_eq!(core_cond.len(), 1);
    assert_eq!(core_cond[0], Core::Id { lit: String::from("cond") });
    assert_eq!(*core_then, Core::Id { lit: String::from("then") });
}

#[test]
fn if_else_verify() {
    let cond = to_pos_unboxed!(ASTNode::Id { lit: String::from("cond") });
    let then = to_pos!(ASTNode::Id { lit: String::from("then") });
    let _else = to_pos!(ASTNode::Id { lit: String::from("else") });
    let if_stmt = to_pos!(ASTNode::IfElse { cond: vec![cond], then, _else: Some(_else) });

    let (core_cond, core_then, core_else) = match desugar(&if_stmt) {
        Core::IfElse { cond, then, _else } => (cond, then, _else),
        other => panic!("Expected reassign but was {:?}", other)
    };

    assert_eq!(core_cond.len(), 1);
    assert_eq!(core_cond[0], Core::Id { lit: String::from("cond") });
    assert_eq!(*core_then, Core::Id { lit: String::from("then") });
    assert_eq!(*core_else, Core::Id { lit: String::from("else") });
}

#[test]
fn while_verify() {
    let cond = to_pos_unboxed!(ASTNode::Id { lit: String::from("cond") });
    let body = to_pos!(ASTNode::ENum { num: String::from("num"), exp: String::from("") });
    let while_stmt = to_pos!(ASTNode::While { cond: vec![cond], body });

    let (core_cond, core_body) = match desugar(&while_stmt) {
        Core::While { cond, body } => (cond, body),
        other => panic!("Expected reassign but was {:?}", other)
    };

    assert_eq!(core_cond.len(), 1);
    assert_eq!(core_cond[0], Core::Id { lit: String::from("cond") });
    assert_eq!(*core_body, Core::ENum { num: String::from("num"), exp: String::from("0") });
}

#[test]
fn match_verify() {
    let cond = to_pos_unboxed!(ASTNode::Id { lit: String::from("cond") });
    let case_1_cond = to_pos!(ASTNode::Id { lit: String::from("case_1_cond") });
    let case_1_body = to_pos!(ASTNode::Id { lit: String::from("case_1_body") });
    let case_2_cond = to_pos!(ASTNode::Id { lit: String::from("case_2_cond") });
    let case_2_body = to_pos!(ASTNode::Id { lit: String::from("case_2_body") });
    let match_stmt = to_pos!(ASTNode::Match {
        cond:  vec![cond],
        cases: vec![
            to_pos_unboxed!(ASTNode::Case { cond: case_1_cond, body: case_1_body }),
            to_pos_unboxed!(ASTNode::Case { cond: case_2_cond, body: case_2_body })
        ]
    });

    let (core_cond, core_cases) = match desugar(&match_stmt) {
        Core::Match { cond, cases } => (cond, cases),
        other => panic!("Expected reassign but was {:?}", other)
    };

    assert_eq!(core_cond.len(), 1);
    assert_eq!(core_cond[0], Core::Id { lit: String::from("cond") });
    assert_eq!(core_cases[0], Core::Case {
        cond: Box::from(Core::Id { lit: String::from("case_1_cond") }),
        body: Box::from(Core::Id { lit: String::from("case_1_body") })
    });
    assert_eq!(core_cases[1], Core::Case {
        cond: Box::from(Core::Id { lit: String::from("case_2_cond") }),
        body: Box::from(Core::Id { lit: String::from("case_2_body") })
    });
}

#[test]
fn for_verify() {
    let expr_1 = to_pos_unboxed!(ASTNode::Id { lit: String::from("expr_1") });
    let expr_2 = to_pos_unboxed!(ASTNode::Id { lit: String::from("expr_2") });
    let collection = to_pos!(ASTNode::Id { lit: String::from("collection") });
    let body = to_pos!(ASTNode::Id { lit: String::from("body") });
    let for_stmt = to_pos!(ASTNode::For { expr: vec![expr_1, expr_2], collection, body });

    let (core_exprs, core_collection, core_body) = match desugar(&for_stmt) {
        Core::For { exprs, collection, body } => (exprs, collection, body),
        other => panic!("Expected reassign but was {:?}", other)
    };

    assert_eq!(core_exprs.len(), 2);
    assert_eq!(core_exprs[0], Core::Id { lit: String::from("expr_1") });
    assert_eq!(core_exprs[1], Core::Id { lit: String::from("expr_2") });
    assert_eq!(*core_collection, Core::Id { lit: String::from("collection") });
    assert_eq!(*core_body, Core::Id { lit: String::from("body") });
}

use mamba::core::construct::Core;
use mamba::desugarer::desugar;
use mamba::parser::ast::ASTNode;
use mamba::parser::ast::ASTNodePos;

#[test]
fn break_verify() {
    let _break = to_pos!(ASTNode::Break);
    let core = desugar(&_break);
    assert_eq!(core, Core::Break);
}

#[test]
fn continue_verify() {
    let _continue = to_pos!(ASTNode::Continue);
    let core = desugar(&_continue);
    assert_eq!(core, Core::Continue);
}

#[test]
fn pass_verify() {
    let pass = to_pos!(ASTNode::Pass);
    let core = desugar(&pass);
    assert_eq!(core, Core::Pass);
}

#[test]
fn print_verify() {
    let expr = to_pos!(ASTNode::Str { lit: String::from("a") });
    let print_stmt = to_pos!(ASTNode::Print { expr });

    let expr_core = match desugar(&print_stmt) {
        Core::Print { expr } => expr,
        other => panic!("Expected print but got: {:?}", other)
    };

    assert_eq!(*expr_core, Core::Str { _str: String::from("a") });
}

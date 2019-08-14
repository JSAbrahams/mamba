use mamba::lexer::token::Token::*;
use mamba::lexer::tokenize;
use mamba::parser::ast::ASTNode;
use mamba::parser::parse;

macro_rules! verify_is_operation {
    ($op:ident, $ast_tree:expr) => {{
        match $ast_tree.node {
            ASTNode::File { statements, .. } => {
                match &statements.first().expect("script empty.").node {
                    ASTNode::$op { left, right } => (left.clone(), right.clone()),
                    other =>
                        panic!("first element script was not op: {}, but was: {:?}", $op, other),
                }
            }
            _ => panic!("ast_tree was not script.")
        }
    }};
}

macro_rules! verify_is_un_operation {
    ($op:ident, $ast_tree:expr) => {{
        match $ast_tree.node {
            ASTNode::File { statements, .. } => {
                match &statements.first().expect("script empty.").node {
                    ASTNode::$op { expr } => expr.clone(),
                    _ => panic!("first element script was not tuple.")
                }
            }
            _ => panic!("ast_tree was not script.")
        }
    }};
}

#[test]
fn addition_verify() {
    let source = String::from("a + b");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let (left, right) = verify_is_operation!(Add, ast_tree);
    assert_eq!(left.node, ASTNode::Id { lit: String::from("a") });
    assert_eq!(right.node, ASTNode::Id { lit: String::from("b") });
}

#[test]
fn addition_unary_verify() {
    let source = String::from("+ b");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let expr = verify_is_un_operation!(AddU, ast_tree);
    assert_eq!(expr.node, ASTNode::Id { lit: String::from("b") });
}

#[test]
fn subtraction_verify() {
    let source = String::from("a - False");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let (left, right) = verify_is_operation!(Sub, ast_tree);
    assert_eq!(left.node, ASTNode::Id { lit: String::from("a") });
    assert_eq!(right.node, ASTNode::Bool { lit: false });
}

#[test]
fn subtraction_unary_verify() {
    let source = String::from("- c");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let expr = verify_is_un_operation!(SubU, ast_tree);
    assert_eq!(expr.node, ASTNode::Id { lit: String::from("c") });
}

#[test]
fn multiplication_verify() {
    let source = String::from("True * b");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let (left, right) = verify_is_operation!(Mul, ast_tree);
    assert_eq!(left.node, ASTNode::Bool { lit: true });
    assert_eq!(right.node, ASTNode::Id { lit: String::from("b") });
}

#[test]
fn division_verify() {
    let source = String::from("10.0 / fgh");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let (left, right) = verify_is_operation!(Div, ast_tree);
    assert_eq!(left.node, ASTNode::Real { lit: String::from("10.0") });
    assert_eq!(right.node, ASTNode::Id { lit: String::from("fgh") });
}

#[test]
fn floor_division_verify() {
    let source = String::from("10.0 // fgh");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let (left, right) = verify_is_operation!(FDiv, ast_tree);
    assert_eq!(left.node, ASTNode::Real { lit: String::from("10.0") });
    assert_eq!(right.node, ASTNode::Id { lit: String::from("fgh") });
}

#[test]
fn power_verify() {
    let source = String::from("chopin ^ liszt");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let (left, right) = verify_is_operation!(Pow, ast_tree);
    assert_eq!(left.node, ASTNode::Id { lit: String::from("chopin") });
    assert_eq!(right.node, ASTNode::Id { lit: String::from("liszt") });
}

#[test]
fn mod_verify() {
    let source = String::from("chopin mod 3E10");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let (left, right) = verify_is_operation!(Mod, ast_tree);
    assert_eq!(left.node, ASTNode::Id { lit: String::from("chopin") });
    assert_eq!(right.node, ASTNode::ENum { num: String::from("3"), exp: String::from("10") });
}

#[test]
fn is_verify() {
    let source = String::from("p is q");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let (left, right) = verify_is_operation!(Is, ast_tree);
    assert_eq!(left.node, ASTNode::Id { lit: String::from("p") });
    assert_eq!(right.node, ASTNode::Id { lit: String::from("q") });
}

#[test]
fn isnt_verify() {
    let source = String::from("p isnt q");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let (left, right) = verify_is_operation!(IsN, ast_tree);
    assert_eq!(left.node, ASTNode::Id { lit: String::from("p") });
    assert_eq!(right.node, ASTNode::Id { lit: String::from("q") });
}

#[test]
fn isa_verify() {
    let source = String::from("lizard isa animal");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let (left, right) = verify_is_operation!(IsA, ast_tree);
    assert_eq!(left.node, ASTNode::Id { lit: String::from("lizard") });
    assert_eq!(right.node, ASTNode::Id { lit: String::from("animal") });
}

#[test]
fn isnta_verify() {
    let source = String::from("i isnta natural");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let (left, right) = verify_is_operation!(IsNA, ast_tree);
    assert_eq!(left.node, ASTNode::Id { lit: String::from("i") });
    assert_eq!(right.node, ASTNode::Id { lit: String::from("natural") });
}

#[test]
fn equality_verify() {
    let source = String::from("i = s");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let (left, right) = verify_is_operation!(Eq, ast_tree);
    assert_eq!(left.node, ASTNode::Id { lit: String::from("i") });
    assert_eq!(right.node, ASTNode::Id { lit: String::from("s") });
}

#[test]
fn le_verify() {
    let source = String::from("one < two");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let (left, right) = verify_is_operation!(Le, ast_tree);
    assert_eq!(left.node, ASTNode::Id { lit: String::from("one") });
    assert_eq!(right.node, ASTNode::Id { lit: String::from("two") });
}

#[test]
fn leq_verify() {
    let source = String::from("two_hundred <= three");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let (left, right) = verify_is_operation!(Leq, ast_tree);
    assert_eq!(left.node, ASTNode::Id { lit: String::from("two_hundred") });
    assert_eq!(right.node, ASTNode::Id { lit: String::from("three") });
}

#[test]
fn ge_verify() {
    let source = String::from("r > 10");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let (left, right) = verify_is_operation!(Ge, ast_tree);
    assert_eq!(left.node, ASTNode::Id { lit: String::from("r") });
    assert_eq!(right.node, ASTNode::Int { lit: String::from("10") });
}

#[test]
fn geq_verify() {
    let source = String::from("4 >= 10");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let (left, right) = verify_is_operation!(Geq, ast_tree);
    assert_eq!(left.node, ASTNode::Int { lit: String::from("4") });
    assert_eq!(right.node, ASTNode::Int { lit: String::from("10") });
}

#[test]
fn in_verify() {
    let source = String::from("one in my_set");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let (left, right) = verify_is_operation!(In, ast_tree);
    assert_eq!(left.node, ASTNode::Id { lit: String::from("one") });
    assert_eq!(right.node, ASTNode::Id { lit: String::from("my_set") });
}

#[test]
fn and_verify() {
    let source = String::from("one and three");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let (left, right) = verify_is_operation!(And, ast_tree);
    assert_eq!(left.node, ASTNode::Id { lit: String::from("one") });
    assert_eq!(right.node, ASTNode::Id { lit: String::from("three") });
}

#[test]
fn or_verify() {
    let source = String::from("one or \"asdf\"");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let (left, right) = verify_is_operation!(Or, ast_tree);
    assert_eq!(left.node, ASTNode::Id { lit: String::from("one") });
    assert_eq!(right.node, ASTNode::Str { lit: String::from("asdf") });
}

#[test]
fn not_verify() {
    let source = String::from("not some_cond");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let expr = verify_is_un_operation!(Not, ast_tree);
    assert_eq!(expr.node, ASTNode::Id { lit: String::from("some_cond") });
}

#[test]
fn sqrt_verify() {
    let source = String::from("sqrt some_num");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let expr = verify_is_un_operation!(Sqrt, ast_tree);
    assert_eq!(expr.node, ASTNode::Id { lit: String::from("some_num") });
}

#[test]
fn b_and_verify() {
    let source = String::from("one _and_ three");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let (left, right) = verify_is_operation!(BAnd, ast_tree);
    assert_eq!(left.node, ASTNode::Id { lit: String::from("one") });
    assert_eq!(right.node, ASTNode::Id { lit: String::from("three") });
}

#[test]
fn b_or_verify() {
    let source = String::from("one _or_ \"asdf\"");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let (left, right) = verify_is_operation!(BOr, ast_tree);
    assert_eq!(left.node, ASTNode::Id { lit: String::from("one") });
    assert_eq!(right.node, ASTNode::Str { lit: String::from("asdf") });
}

#[test]
fn b_xor_verify() {
    let source = String::from("one _xor_ \"asdf\"");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let (left, right) = verify_is_operation!(BXOr, ast_tree);
    assert_eq!(left.node, ASTNode::Id { lit: String::from("one") });
    assert_eq!(right.node, ASTNode::Str { lit: String::from("asdf") });
}

#[test]
fn b_ones_complement_verify() {
    let source = String::from("_not_ \"asdf\"");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let expr = verify_is_un_operation!(BOneCmpl, ast_tree);
    assert_eq!(expr.node, ASTNode::Str { lit: String::from("asdf") });
}

#[test]
fn b_lshift_verify() {
    let source = String::from("one << \"asdf\"");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let (left, right) = verify_is_operation!(BLShift, ast_tree);
    assert_eq!(left.node, ASTNode::Id { lit: String::from("one") });
    assert_eq!(right.node, ASTNode::Str { lit: String::from("asdf") });
}

#[test]
fn brshift_verify() {
    let source = String::from("one >> \"asdf\"");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let (left, right) = verify_is_operation!(BRShift, ast_tree);
    assert_eq!(left.node, ASTNode::Id { lit: String::from("one") });
    assert_eq!(right.node, ASTNode::Str { lit: String::from("asdf") });
}

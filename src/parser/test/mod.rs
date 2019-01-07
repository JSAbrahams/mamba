use super::*;

macro_rules! vec_from {
     ( $( $x:expr ),* ) => {
        {
            let mut ast_nodes = Vec::new();
            $(
                ast_nodes.push($x);
            )*

            ASTNode::Program(Vec::new(), Box::new(ASTNode::Do(ast_nodes)))
        }
    };
}

#[test]
fn int() {
    let tokens = vec![Token::Int(10.to_string())];
    let parsed = parse(tokens);

    assert_eq!(vec_from!(ASTNode::Int(10.to_string())), parsed.unwrap());
}

#[test]
fn real() {
    let tokens = vec![Token::Real(10.4.to_string())];
    let parsed = parse(tokens);

    assert_eq!(vec_from!(ASTNode::Real(10.4.to_string())), parsed.unwrap());
}

#[test]
fn string() {
    let tokens = vec![Token::Str("hello".to_string())];
    let parsed = parse(tokens);

    assert_eq!(vec_from!(ASTNode::Str("hello".to_string())), parsed.unwrap());
}

#[test]
fn addition() {
    let tokens = vec![Token::Real(3.5.to_string()), Token::Add, Token::Int(7.to_string())];
    let parsed = parse(tokens);

    assert_eq!(
        vec_from!(ASTNode::Add(Box::from(ASTNode::Real(3.5.to_string())),
        Box::from(ASTNode::Int(7.to_string())))), parsed.unwrap())
}

#[test]
fn order_of_operation() {
    let tokens = vec![Token::Int(3.to_string()), Token::Add, Token::Int(10.to_string()),
                      Token::Mul, Token::Real(20.2.to_string())];
    let parsed = parse(tokens);

    assert_eq!(
        vec_from!(ASTNode::Add(Box::from(ASTNode::Int(3.to_string())), Box::from(ASTNode::Mul(
        Box::from(ASTNode::Int(10.to_string())), Box::from(ASTNode::Real(20.2.to_string())))))),
        parsed.unwrap())
}

#[test]
fn unary_expression() {
    let tokens = vec![Token::Add, Token::Real(3.14.to_string())];
    let parsed = parse(tokens);

    assert_eq!(vec_from!(ASTNode::AddU(Box::new(ASTNode::Real(3.14.to_string())))), parsed.unwrap())
}

#[test]
fn unary_negative_expression() {
    let tokens = vec![Token::Sub, Token::Real(3.14.to_string())];
    let parsed = parse(tokens);

    assert_eq!(vec_from!(ASTNode::SubU(Box::new(ASTNode::Real(3.14.to_string())))), parsed.unwrap())
}

#[test]
fn if_statement() {
    let tokens = vec![Token::If, Token::Bool(true), Token::Then,
                      Token::Int(10.to_string())];
    let parsed = parse(tokens);

    assert_eq!(vec_from!(ASTNode::If(Box::from(ASTNode::Bool(true)),
    Box::from(ASTNode::Int(10.to_string())))), parsed.unwrap())
}

#[test]
fn if_statement_with_else() {
    let tokens = vec![Token::If, Token::Bool(true), Token::Then, Token::Int(10.to_string()),
                      Token::Else, Token::Int(20.to_string())];
    let parsed = parse(tokens);

    assert_eq!(vec_from!(ASTNode::IfElse(Box::from(ASTNode::Bool(true)),
    Box::from(ASTNode::Int(10.to_string())), Box::from(ASTNode::Int(20.to_string())))), parsed.unwrap())
}

#[test]
fn simple_assignment() {
    let tokens = vec![Token::Let, Token::Id("a".to_string()), Token::Assign,
                      Token::Real(3.14.to_string())];
    let parsed = parse(tokens);

    assert_eq!(vec_from!(ASTNode::Assign(Box::new(ASTNode::Id("a".to_string())),
     Box::new(ASTNode::Real(3.14.to_string())))), parsed.unwrap())
}

#[test]
fn simple_mutable_assignment() {
    let tokens = vec![Token::Mut, Token::Let, Token::Id("a".to_string()),
                      Token::Assign, Token::Real(3.14.to_string())];
    let parsed = parse(tokens);

    assert_eq!(vec_from!(
    ASTNode::Mut(Box::new(ASTNode::Assign(Box::new(ASTNode::Id("a".to_string())),
    Box::new(ASTNode::Real(3.14.to_string())))))), parsed.unwrap())
}

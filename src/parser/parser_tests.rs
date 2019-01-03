use super::*;

macro_rules! vec_from {
     ( $( $x:expr ),* ) => {
        {
            let mut ast_nodes = Vec::new();
            $(
                ast_nodes.push(Box::from($x));
            )*
            ASTNode::Do(ast_nodes)
        }
    };
}

#[test]
fn number() {
    let tokens = vec![Token::Num(10.0)];
    let parsed = parse(tokens);

    assert_eq!(vec_from!(ASTNode::Num(10.0)), parsed.unwrap());
}

#[test]
fn string() {
    let tokens = vec![Token::Str("hello".to_string())];
    let parsed = parse(tokens);

    assert_eq!(vec_from!(ASTNode::Str("hello".to_string())), parsed.unwrap());
}

#[test]
fn addition() {
    let tokens = vec![Token::Num(3.5), Token::Add, Token::Num(7.0)];
    let parsed = parse(tokens);

    assert_eq!(
        vec_from!(ASTNode::Add(Box::from(ASTNode::Num(3.5)), Box::from(ASTNode::Num(7.0)))),
        parsed.unwrap()
    )
}

#[test]
fn order_of_operation() {
    let tokens = vec![Token::Num(3.0), Token::Add, Token::Num(10.0),
                      Token::Mul, Token::Num(20.0)];
    let parsed = parse(tokens);

    assert_eq!(
        vec_from!(ASTNode::Add(Box::from(ASTNode::Num(3.0)), Box::from(ASTNode::Mul(
        Box::from(ASTNode::Num(10.0)), Box::from(ASTNode::Num(20.0)))))),
        parsed.unwrap()
    )
}

#[test]
fn unary_expression() {
    let tokens = vec![Token::Add, Token::Num(3.14)];
    let parsed = parse(tokens);

    assert_eq!(vec_from!(ASTNode::AddU(Box::new(ASTNode::Num(3.14)))), parsed.unwrap())
}

#[test]
fn unary_negative_expression() {
    let tokens = vec![Token::Sub, Token::Num(3.14)];
    let parsed = parse(tokens);

    assert_eq!(vec_from!(ASTNode::SubU(Box::new(ASTNode::Num(3.14)))), parsed.unwrap())
}

#[test]
fn if_statement() {
    let tokens = vec![Token::If, Token::Bool(true), Token::Then, Token::Num(10.0)];
    let parsed = parse(tokens);

    assert_eq!(vec_from!(ASTNode::If(Box::from(ASTNode::Bool(true)),
    Box::from(ASTNode::Num(10.0)))), parsed.unwrap())
}

#[test]
fn if_statement_with_else() {
    let tokens = vec![Token::If, Token::Bool(true), Token::Then, Token::Num(10.0),
                      Token::Else, Token::Num(20.0)];
    let parsed = parse(tokens);

    assert_eq!(vec_from!(ASTNode::IfElse(Box::from(ASTNode::Bool(true)),
    Box::from(ASTNode::Num(10.0)), Box::from(ASTNode::Num(20.0)))), parsed.unwrap())
}

#[test]
fn simple_assignment() {
    let tokens = vec![Token::Let, Token::Id("a".to_string()), Token::Assign,
                      Token::Num(3.14)];
    let parsed = parse(tokens);

    assert_eq!(vec_from!(ASTNode::Assign(Box::new(ASTNode::Id("a".to_string())),
     Box::new(ASTNode::Num(3.14)))), parsed.unwrap())
}

#[test]
fn simple_mutable_assignment() {
    let tokens = vec![Token::Mut, Token::Let, Token::Id("a".to_string()),
                      Token::Assign, Token::Num(3.14)];
    let parsed = parse(tokens);

    assert_eq!(vec_from!(
    ASTNode::Mut(Box::new(ASTNode::Assign(Box::new(ASTNode::Id("a".to_string())),
    Box::new(ASTNode::Num(3.14)))))), parsed.unwrap())
}

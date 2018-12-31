use super::*;

#[test]
fn parse_number() {
    let tokens = vec![Token::Num(10.0)];
    let parsed = parse(tokens);

    assert_eq!(ASTNode::Do(vec![Box::from(ASTNode::Num(10.0))]), parsed.unwrap());
}

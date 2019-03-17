use mamba::lexer::token::Token;
use mamba::lexer::token::TokenPos;
use mamba::lexer::tokenize;

#[test]
fn parse_from() {
    let source = String::from("from i use b");
    let tokens = tokenize(&source).unwrap();
    assert_eq!(tokens, vec![TokenPos { line: 1, pos: 1, token: Token::From },
                            TokenPos { line: 1, pos: 6, token: Token::Id(String::from("i")) },
                            TokenPos { line: 1, pos: 8, token: Token::Use },
                            TokenPos { line: 1, pos: 12, token: Token::Id(String::from("b")) }]);
}

use mamba::lexer::token::Token;
use mamba::lexer::token::TokenPos;
use mamba::lexer::tokenize;

#[test]
fn parse_from() {
    let source = String::from("from i use b");
    let tokens = tokenize(&source).unwrap();
    assert_eq!(tokens,
               vec![TokenPos { line: 1, pos: 1, token: Token::From },
                    TokenPos { line: 1, pos: 6, token: Token::Id(String::from("i")) },
                    TokenPos { line: 1, pos: 8, token: Token::Use },
                    TokenPos { line: 1, pos: 12, token: Token::Id(String::from("b")) }]);
}

#[test]
fn parse_operators() {
    let source = String::from("+ - * / ^ mod sqrt i");
    let tokens = tokenize(&source).unwrap();
    assert_eq!(tokens,
               vec![TokenPos { line: 1, pos: 1, token: Token::Add },
                    TokenPos { line: 1, pos: 3, token: Token::Sub },
                    TokenPos { line: 1, pos: 5, token: Token::Mul },
                    TokenPos { line: 1, pos: 7, token: Token::Div },
                    TokenPos { line: 1, pos: 9, token: Token::Pow },
                    TokenPos { line: 1, pos: 11, token: Token::Mod },
                    TokenPos { line: 1, pos: 15, token: Token::Sqrt },
                    TokenPos { line: 1, pos: 20, token: Token::Id(String::from("i")) }
               ]);
}

#[test]
fn test_comparison() {
    let source = String::from("< > <= >= = /= is isnt i");
    let tokens = tokenize(&source).unwrap();
    assert_eq!(tokens,
               vec![TokenPos { line: 1, pos: 1, token: Token::Le },
                    TokenPos { line: 1, pos: 3, token: Token::Ge },
                    TokenPos { line: 1, pos: 5, token: Token::Leq },
                    TokenPos { line: 1, pos: 8, token: Token::Geq },
                    TokenPos { line: 1, pos: 11, token: Token::Eq },
                    TokenPos { line: 1, pos: 13, token: Token::Neq },
                    TokenPos { line: 1, pos: 16, token: Token::Is },
                    TokenPos { line: 1, pos: 19, token: Token::IsN },
                    TokenPos { line: 1, pos: 24, token: Token::Id(String::from("i")) }
               ]);
}

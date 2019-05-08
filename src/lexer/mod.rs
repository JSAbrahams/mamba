use crate::lexer::common::State;
use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;

pub mod token;

#[macro_use]
mod common;

/// Convert a given string to a sequence of
/// [TokenPos](crate::lexer::token::TokenPos), each containing a
/// [Token](crate::lexer::token::Token), in addition to line number and
/// position. Note that line number and position are 1-indexed.
///
/// Should never panic.
///
/// # Examples
///
/// ```
/// # use mamba::lexer::tokenize;
/// # use mamba::lexer::token::Token;
/// # use mamba::lexer::token::TokenPos;
/// let source = "a <- 2";
/// let tokens = tokenize(&source).unwrap();
///
/// assert_eq!(tokens[0], TokenPos { line: 1, pos: 1, token: Token::Id(String::from("a")) });
/// assert_eq!(tokens[1], TokenPos { line: 1, pos: 3, token: Token::Assign });
/// assert_eq!(tokens[2], TokenPos { line: 1, pos: 6, token: Token::Int(String::from("2")) });
/// ```
///
/// # Failures
///
/// Fails if it encounters an unrecognized character.
///
/// ```
/// # use mamba::lexer::tokenize;
/// // The '$' character on its own is meaningless in Mamba.
/// let source = "$";
/// let result = tokenize(&source);
/// assert_eq!(result.is_err(), true);
/// ```
#[allow(clippy::cyclomatic_complexity)]
pub fn tokenize(input: &str) -> Result<Vec<TokenPos>, String> {
    let mut it = input.chars().peekable();
    let mut tokens = Vec::new();
    let mut state = State::new();

    while let Some(c) = it.next() {
        let token_pos = match c {
            ':' => create(&state, Token::DoublePoint),
            '(' => create(&state, Token::LRBrack),
            ')' => create(&state, Token::RRBrack),
            '[' => create(&state, Token::LSBrack),
            ']' => create(&state, Token::RSBrack),
            '{' => create(&state, Token::LCBrack),
            '}' => create(&state, Token::RCBrack),

            '\n' => create(&state, Token::NL),
            '\r' => match it.next() {
                Some('\n') => create(&state, Token::NL),
                _ => panic!("Create good error message.")
            },

            '#' => {
                let mut comment = String::new();
                while it.peek().is_some() && it.peek().unwrap().is_numeric() {
                    comment.push(it.next().unwrap());
                }
                create(&state, Token::Comment(comment))
            }

            '0'..='9' => {
                let mut number = c.to_string();
                while it.peek().is_some() && it.peek().unwrap().is_numeric() {
                    number.push(it.next().unwrap());
                }
                create(&state, Token::Int(number))
            }
            'a'..='z' | 'A'..='Z' | '_' => unimplemented!(),

            other => panic!("Create good error message.")
        };

        tokens.push(token_pos);
    }

    Ok(tokens)
}

fn create(mut state: &State, token: Token) -> TokenPos {
    TokenPos { line: state.line, pos: state.pos, token }
}

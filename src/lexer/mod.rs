use crate::lexer::common::State;
use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use std::iter::Peekable;

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
            '|' => create(&state, Token::Ver),
            '\n' => create(&state, Token::NL),
            '\r' => match it.next() {
                Some('\n') => create(&state, Token::NL),
                _ => panic!("Create good error message.")
            },
            '.' => match it.peek() {
                Some('.') => match (it.next(), it.peek()) {
                    (_, Some('=')) => next_and_create(it, &state, Token::RangeIncl),
                    (..) => create(&state, Token::Range)
                },
                _ => create(&state, Token::Point)
            },
            '<' => match it.peek() {
                Some('-') => next_and_create(it, &state, Token::Assign),
                Some('=') => next_and_create(it, &state, Token::Leq),
                _ => create(&state, Token::Le)
            },
            '>' => match it.peek() {
                Some('=') => next_and_create(it, &state, Token::Geq),
                _ => create(&state, Token::Ge)
            },
            '+' => create(&state, Token::Add),
            '-' => match it.peek() {
                Some('>') => next_and_create(it, &state, Token::To),
                _ => create(&state, Token::Sub)
            },
            '*' => create(&state, Token::Mul),
            '/' => match it.peek() {
                Some('=') => next_and_create(it, &state, Token::Neq),
                _ => create(&state, Token::Div)
            },
            '\\' => create(&state, Token::BSlash),
            '^' => create(&state, Token::Pow),
            '=' => match it.peek() {
                Some('>') => next_and_create(it, &state, Token::BTo),
                _ => create(&state, Token::Eq)
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
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut id_or_operation = c.to_string();
                while let Some(c) = it.peek() {
                    match c {
                        'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => id_or_operation.push(*c),
                        _ => break
                    }
                }
                create(&state, as_op_or_id(id_or_operation))
            }
            '"' => {
                let mut string = String::new();
                while let Some(c) = it.next() {
                    if c == '"' {
                        break;
                    }
                    string.push(c)
                }
                create(&state, Token::Str(string))
            }
            ' ' => {
                &state.space();
                None
            }
            other => panic!("Create good error message.")
        };

        if token_pos.is_some() {
            tokens.push(token_pos.unwrap());
        }
    }

    Ok(tokens)
}

fn next_and_create(mut it: &Peekable<Char>, mut state: &State, token: Token) -> Option<TokenPos> {
    it.next();
    state.token(token);
    Some(TokenPos { line: state.line, pos: state.pos, token })
}

fn create(mut state: &State, token: Token) -> Option<TokenPos> {
    state.token(token);
    Some(TokenPos { line: state.line, pos: state.pos, token })
}

fn as_op_or_id(mut string: String) -> Token {
    match string.as_ref() {
        "_" => Token::Underscore,

        "from" => Token::From,
        "type" => Token::Type,
        "stateful" => Token::Stateful,
        "stateless" => Token::Stateless,
        "as" => Token::As,
        "private" => Token::Private,

        "import" => Token::Import,
        "forward" => Token::Forward,
        "self" => Token::_Self,
        "vararg" => Token::Vararg,
        "init" => Token::Init,

        "def" => Token::Def,
        "ofmut" => Token::OfMut,
        "mut" => Token::Mut,
        "and" => Token::And,
        "or" => Token::Or,
        "not" => Token::Not,
        "is" => Token::Is,
        "isa" => Token::IsA,
        "isnt" => Token::IsN,
        "isnta" => Token::IsNA,
        "mod" => Token::Mod,
        "sqrt" => Token::Sqrt,
        "while" => Token::While,
        "foreach" => Token::For,

        "if" => Token::If,
        "else" => Token::Else,
        "match" => Token::Match,
        "with" => Token::With,
        "continue" => Token::Continue,
        "break" => Token::Break,
        "return" => Token::Ret,
        "then" => Token::Then,
        "do" => Token::Do,

        "in" => Token::In,

        "raises" => Token::Raises,
        "handle" => Token::Handle,
        "retry" => Token::Retry,
        "when" => Token::When,

        "True" => Token::Bool(true),
        "False" => Token::Bool(false),
        "print" => Token::Print,

        "undefined" => Token::Undefined,
        "pass" => Token::Pass,

        _ => Token::Id(string)
    }
}

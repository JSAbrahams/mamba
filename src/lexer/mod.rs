use crate::lexer::common::State;
use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use std::iter::Peekable;
use std::str::Chars;

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
#[allow(clippy::cyclomatic_complexity, clippy::while_let_on_iterator)]
pub fn tokenize(input: &str) -> Result<Vec<TokenPos>, String> {
    let mut it = input.chars().peekable();
    let mut tokens = Vec::new();
    let mut state = State::new();

    while let Some(c) = it.next() {
        let token_pos = match c {
            ',' => create(&mut state, Token::Comma),
            ':' => create(&mut state, Token::DoublePoint),
            '(' => create(&mut state, Token::LRBrack),
            ')' => create(&mut state, Token::RRBrack),
            '[' => create(&mut state, Token::LSBrack),
            ']' => create(&mut state, Token::RSBrack),
            '{' => create(&mut state, Token::LCBrack),
            '}' => create(&mut state, Token::RCBrack),
            '|' => create(&mut state, Token::Ver),
            '\n' => create(&mut state, Token::NL),
            '\r' => match it.next() {
                Some('\n') => create(&mut state, Token::NL),
                _ => return Err(String::from("Create good error message."))
            },
            '.' => match it.peek() {
                Some('.') => match (it.next(), it.peek()) {
                    (_, Some('=')) => next_and_create(&mut it, &mut state, Token::RangeIncl),
                    (..) => create(&mut state, Token::Range)
                },
                _ => create(&mut state, Token::Point)
            },
            '<' => match it.peek() {
                Some('<') => next_and_create(&mut it, &mut state, Token::BLShift),
                Some('-') => next_and_create(&mut it, &mut state, Token::Assign),
                Some('=') => next_and_create(&mut it, &mut state, Token::Leq),
                _ => create(&mut state, Token::Le)
            },
            '>' => match it.peek() {
                Some('>') => next_and_create(&mut it, &mut state, Token::BRShift),
                Some('=') => next_and_create(&mut it, &mut state, Token::Geq),
                _ => create(&mut state, Token::Ge)
            },
            '+' => create(&mut state, Token::Add),
            '-' => match it.peek() {
                Some('>') => next_and_create(&mut it, &mut state, Token::To),
                _ => create(&mut state, Token::Sub)
            },
            '*' => create(&mut state, Token::Mul),
            '/' => match it.peek() {
                Some('/') => next_and_create(&mut it, &mut state, Token::FDiv),
                Some('=') => next_and_create(&mut it, &mut state, Token::Neq),
                _ => create(&mut state, Token::Div)
            },
            '\\' => create(&mut state, Token::BSlash),
            '^' => create(&mut state, Token::Pow),
            '=' => match it.peek() {
                Some('>') => next_and_create(&mut it, &mut state, Token::BTo),
                _ => create(&mut state, Token::Eq)
            },
            '#' => {
                let mut comment = String::new();
                while it.peek().is_some()
                    && *it.peek().unwrap() != '\n'
                    && *it.peek().unwrap() != '\r'
                {
                    comment.push(it.next().unwrap());
                }
                create(&mut state, Token::Comment(comment))
            }
            '?' => match it.peek() {
                Some('.') => next_and_create(&mut it, &mut state, Token::QuestCall),
                Some('o') => match (it.next(), it.peek()) {
                    (_, Some('r')) => next_and_create(&mut it, &mut state, Token::QuestOr),
                    _ => return Err(String::from("Create good error message."))
                },
                _ => create(&mut state, Token::Quest)
            },
            '0'..='9' => {
                let mut number = c.to_string();
                let mut exp = String::new();
                let mut float = false;
                let mut e_num = false;

                while let Some(&c) = it.peek() {
                    match c {
                        '0'...'9' if !e_num => {
                            number.push(c);
                            it.next();
                        }
                        '0'...'9' if e_num => {
                            exp.push(c);
                            it.next();
                        }
                        'E' if e_num => break,
                        'E' => {
                            e_num = true;
                            it.next();
                        }
                        '.' if float || e_num => break,
                        '.' => {
                            number.push(c);
                            float = true;
                            it.next();
                        }
                        _ => break
                    }
                }
                create(
                    &mut state,
                    if e_num {
                        Token::ENum(number, exp)
                    } else if float {
                        Token::Real(number)
                    } else {
                        Token::Int(number)
                    }
                )
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut id_or_operation = c.to_string();
                while let Some(c) = it.peek() {
                    match c {
                        'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => {
                            id_or_operation.push(*c);
                            it.next();
                        }
                        _ => break
                    }
                }
                create(&mut state, as_op_or_id(id_or_operation))
            }
            '"' => {
                let mut string = String::new();
                let mut back_slash = false;
                while let Some(c) = it.next() {
                    if !back_slash && c == '"' {
                        break;
                    }
                    string.push(c);
                    back_slash = c == '\\';
                }
                create(&mut state, Token::Str(string))
            }
            ' ' => {
                state.space();
                vec![]
            }

            other => return Err(format!("Create good error message: {}.", other))
        };

        for tp in token_pos {
            tokens.push(tp);
        }
    }

    Ok(tokens)
}

fn next_and_create(it: &mut Peekable<Chars>, state: &mut State, token: Token) -> Vec<TokenPos> {
    it.next();
    create(state, token)
}

fn create(state: &mut State, token: Token) -> Vec<TokenPos> { state.token(token) }

fn as_op_or_id(string: String) -> Token {
    match string.as_ref() {
        "_" => Token::Underscore,

        "from" => Token::From,
        "type" => Token::Type,
        "class" => Token::Class,
        "pure" => Token::Pure,
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
        "for" => Token::For,
        "step" => Token::Step,

        "_and_" => Token::BAnd,
        "_or_" => Token::BOr,
        "_xor_" => Token::BXOr,
        "_not_" => Token::BOneCmpl,

        "if" => Token::If,
        "else" => Token::Else,
        "match" => Token::Match,
        "continue" => Token::Continue,
        "break" => Token::Break,
        "return" => Token::Ret,
        "then" => Token::Then,
        "do" => Token::Do,
        "with" => Token::With,

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

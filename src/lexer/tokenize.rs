use std::iter::Peekable;
use std::str::Chars;

use crate::lexer::common::State;
use crate::lexer::lex_result::{LexErr, LexResult};
use crate::lexer::token::{Lex, Token};

pub fn into_tokens(c: char, it: &mut Peekable<Chars>, state: &mut State) -> LexResult {
    match c {
        ',' => create(state, Token::Comma),
        ':' => create(state, Token::DoublePoint),
        '(' => create(state, Token::LRBrack),
        ')' => create(state, Token::RRBrack),
        '[' => create(state, Token::LSBrack),
        ']' => create(state, Token::RSBrack),
        '{' => create(state, Token::LCBrack),
        '}' => create(state, Token::RCBrack),
        '|' => create(state, Token::Ver),
        '\n' => create(state, Token::NL),
        '\r' => match it.next() {
            Some('\n') => create(state, Token::NL),
            _ => Err(LexErr::new(
                state.line,
                state.pos,
                None,
                "return carriage not followed by newline"
            ))
        },
        '.' => match it.peek() {
            Some('.') => match (it.next(), it.peek()) {
                (_, Some('=')) => next_and_create(it, state, Token::RangeIncl),
                _ => create(state, Token::Range)
            },
            _ => create(state, Token::Point)
        },
        '<' => match it.peek() {
            Some('<') => next_and_create(it, state, Token::BLShift),
            Some('-') => next_and_create(it, state, Token::Assign),
            Some('=') => next_and_create(it, state, Token::Leq),
            _ => create(state, Token::Le)
        },
        '>' => match it.peek() {
            Some('>') => next_and_create(it, state, Token::BRShift),
            Some('=') => next_and_create(it, state, Token::Geq),
            _ => create(state, Token::Ge)
        },
        '+' => create(state, Token::Add),
        '-' => match it.peek() {
            Some('>') => next_and_create(it, state, Token::To),
            _ => create(state, Token::Sub)
        },
        '*' => create(state, Token::Mul),
        '/' => match it.peek() {
            Some('/') => next_and_create(it, state, Token::FDiv),
            Some('=') => next_and_create(it, state, Token::Neq),
            _ => create(state, Token::Div)
        },
        '\\' => create(state, Token::BSlash),
        '^' => create(state, Token::Pow),
        '=' => match it.peek() {
            Some('>') => next_and_create(it, state, Token::BTo),
            _ => create(state, Token::Eq)
        },
        '#' => {
            let mut comment = String::new();
            while it.peek().is_some() && *it.peek().unwrap() != '\n' && *it.peek().unwrap() != '\r'
            {
                comment.push(it.next().unwrap());
            }
            create(state, Token::Comment(comment))
        }
        '?' => create(state, Token::Question),
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
                state,
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
            create(state, as_op_or_id(id_or_operation))
        }
        '"' => {
            let mut string = String::new();
            let mut back_slash = false;
            for c in it {
                if !back_slash && c == '"' {
                    break;
                }
                string.push(c);
                back_slash = c == '\\';
            }
            create(state, Token::Str(string))
        }
        ' ' => {
            state.space();
            Ok(vec![])
        }
        other => Err(LexErr::new(
            state.line,
            state.pos,
            None,
            format!("unrecognized character: {}", other).as_ref()
        ))
    }
}

fn next_and_create(
    it: &mut Peekable<Chars>,
    state: &mut State,
    token: Token
) -> LexResult<Vec<Lex>> {
    it.next();
    create(state, token)
}

fn create(state: &mut State, token: Token) -> LexResult<Vec<Lex>> { Ok(state.token(token)) }

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
        "raise" => Token::Raise,
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

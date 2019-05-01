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
#[allow(clippy::cyclomatic_complexity)]
pub fn tokenize(input: &str) -> Result<Vec<TokenPos>, String> {
    let mut it = input.chars().peekable();
    let mut tokens = Vec::new();

    let mut cur_ind = 0;
    let mut line_ind = 0;
    let mut cons_spaces = 0;
    let mut last_nl = true;
    let mut cons_nl = 0;

    let mut line = 1;
    let mut pos = 1;

    macro_rules! next_pos_and_tp {
        ($amount:expr, $tok:path) => {{
            next_pos!($amount, $tok);
            it.next();
        }};
        ($fun:path) => {{
            next_pos!($fun);
            it.next();
        }};
    };

    macro_rules! indentation {
        () => {{
            let mut indent_pos = 1;
            for _ in cur_ind..line_ind {
                tokens.push(TokenPos { line, pos: indent_pos, token: Token::Indent });
                indent_pos += 4;
            }
            for _ in line_ind..cur_ind {
                tokens.push(TokenPos { line, pos, token: Token::Dedent });
            }

            for _ in 1..cons_nl {
                tokens.push(TokenPos { line, pos, token: Token::NL });
            }

            cur_ind = line_ind;
            cons_nl = 0;
            last_nl = false;
        }};
    }

    macro_rules! next_pos {
        ($amount:expr, $tok:path) => {{
            indentation!();
            pos += $amount;
            tokens.push(TokenPos { line, pos, token: $tok });
        }};
        ($fun:path) => {{
            indentation!();
            tokens.push(TokenPos { line, pos, token: $fun(&mut it, &mut pos) });
        }};
    };

    macro_rules! next_line_and_tp {
        () => {{
            if !last_nl {
                cur_ind = line_ind;
                tokens.push(TokenPos { line, pos, token: Token::NL });
            }

            cons_nl += 1;
            line += 1;
            pos = 1;

            line_ind = 0;
            last_nl = true;
            it.next();
        }};
    };

    macro_rules! increase_indent {
        () => {{
            pos += 4;
            line_ind += 1;
        }};
    };

    while let Some(&c) = it.peek() {
        match c {
            '.' => match (it.next(), it.peek()) {
                (_, Some('.')) => match (it.next(), it.peek()) {
                    (_, Some('=')) => next_pos_and_tp!(3, Token::RangeIncl),
                    _ => next_pos!(2, Token::Range)
                },
                _ => next_pos!(1, Token::Point)
            },
            ':' => next_pos_and_tp!(1, Token::DoublePoint),
            '_' => next_pos!(get_id_or_op),
            ',' => next_pos_and_tp!(1, Token::Comma),
            '(' => next_pos_and_tp!(1, Token::LRBrack),
            ')' => next_pos_and_tp!(1, Token::RRBrack),
            '[' => next_pos_and_tp!(1, Token::LSBrack),
            ']' => next_pos_and_tp!(1, Token::RSBrack),
            '{' => next_pos_and_tp!(1, Token::LCBrack),
            '}' => next_pos_and_tp!(1, Token::RCBrack),
            '?' => match (it.next(), it.peek()) {
                (_, Some('.')) => next_pos_and_tp!(1, Token::QuestCall),
                (_, Some('o')) => match (it.next(), it.peek()) {
                    (_, Some('r')) => {
                        next_pos_and_tp!(2, Token::QuestOr);
                        pos += 3
                    }
                    (_, Some(other)) => return Err(format!("Expected `?or`. Was '{}'.", other)),
                    (_, None) => return Err("Expected `?or`.".to_string())
                },
                _ => next_pos!(0, Token::Quest)
            },
            '|' => next_pos_and_tp!(1, Token::Ver),
            '\n' => next_line_and_tp!(),
            '\r' => match (it.next(), it.peek()) {
                (_, Some('\n')) => next_line_and_tp!(),
                (_, Some(other)) =>
                    return Err(format!("Expected newline after carriage return. Was '{}'.", other)),
                (_, None) => return Err("File ended with carriage return.".to_string())
            },
            '\t' => increase_indent!(),
            '<' | '>' | '+' | '-' | '*' | '/' | '\\' | '^' | '=' => next_pos!(get_operator),
            '0'...'9' => next_pos!(get_number),
            '"' => next_pos!(get_string),
            'a'...'z' | 'A'...'Z' => next_pos!(get_id_or_op),
            '#' => ignore_comment(&mut it),
            ' ' => {
                pos += 1;
                cons_spaces += 1;
                if cons_spaces == 4 {
                    cons_spaces = 0;
                    increase_indent!();
                    pos -= 4;
                }
                it.next();
                continue;
            }
            c => return Err(format!("Unrecognized character: '{}'.", c))
        }

        cons_spaces = 0;
    }

    Ok(tokens)
}

fn ignore_comment(it: &mut Peekable<Chars>) {
    while let Some(c) = it.peek() {
        match c {
            '\n' => break,
            _ => {
                it.next();
                continue;
            }
        }
    }
}

fn get_operator(it: &mut Peekable<Chars>, pos: &mut i32) -> Token {
    *pos += 1;
    match it.next() {
        Some('<') => match it.peek() {
            Some('=') => next_and!(it, pos, Token::Leq),
            Some('-') => next_and!(it, pos, Token::Assign),
            _ => Token::Le
        },
        Some('>') => match it.peek() {
            Some('=') => next_and!(it, pos, Token::Geq),
            _ => Token::Ge
        },
        Some('+') => Token::Add,
        Some('-') => match it.peek() {
            Some('>') => next_and!(it, pos, Token::To),
            _ => Token::Sub
        },
        Some('/') => match it.peek() {
            Some('=') => next_and!(it, pos, Token::Neq),
            _ => Token::Div
        },
        Some('\\') => Token::BSlash,
        Some('*') => Token::Mul,
        Some('^') => Token::Pow,
        Some('=') => match it.peek() {
            Some('>') => next_and!(it, pos, Token::BTo),
            _ => Token::Eq
        },
        _ => panic!("get operator received a character it shouldn't have.")
    }
}

fn get_number(it: &mut Peekable<Chars>, pos: &mut i32) -> Token {
    let mut num = String::new();
    let mut exp = String::new();
    let mut e_found = false;
    let mut comma = false;

    match it.next() {
        Some(digit @ '0'...'9') => num.push(digit),
        _ => panic!("get number received a character it shouldn't have.")
    }
    *pos += 1;

    while let Some(&c) = it.peek() {
        match c {
            '0'...'9' if !e_found => next_and!(it, pos, num.push(c)),
            '0'...'9' if e_found => next_and!(it, pos, exp.push(c)),
            'e' | 'E' if e_found => break,
            'e' | 'E' => next_and!(it, pos, e_found = true),
            '.' if comma || e_found => break,
            '.' => next_and!(it, pos, {
                num.push(c);
                comma = true;
            }),
            _ => break
        }
    }

    match (e_found, comma) {
        (true, _) => Token::ENum(num, exp),
        (false, true) => Token::Real(num),
        (false, false) => Token::Int(num)
    }
}

fn get_string(it: &mut Peekable<Chars>, pos: &mut i32) -> Token {
    it.next();
    let mut result = String::new();
    let mut last_backslash = false;

    while let Some(&c) = it.peek() {
        match c {
            '"' => next_and!(it, pos, {
                if !last_backslash {
                    break;
                } else {
                    result.push(c)
                };
                last_backslash = false
            }),
            '\\' => next_and!(it, pos, {
                result.push(c);
                last_backslash = true
            }),
            _ => next_and!(it, pos, {
                result.push(c);
                last_backslash = false
            })
        }
    }

    *pos += 1; // for closing " character
    Token::Str(result)
}

fn get_id_or_op(it: &mut Peekable<Chars>, pos: &mut i32) -> Token {
    let mut result = String::new();

    while let Some(&c) = it.peek() {
        match c {
            'a'...'z' | 'A'...'Z' | '0'...'9' | '_' => next_and!(it, pos, result.push(c)),
            _ => break
        }
    }

    match result.as_ref() {
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

        _ => Token::Id(result)
    }
}

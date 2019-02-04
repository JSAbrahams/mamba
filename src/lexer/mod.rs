use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use std::iter::Peekable;
use std::str::Chars;

pub mod token;

#[macro_use]
macro_rules! next_and { ($it:expr, $pos:expr, $stmt:stmt) => {{ $it.next(); *$pos += 1; $stmt }} }

pub fn tokenize(input: String) -> Result<Vec<TokenPos>, String> {
    let mut it = input.chars().peekable();
    let mut tokens = Vec::new();

    let mut current_indent = 0;
    let mut this_line_indent = 0;
    let mut consecutive_spaces = 0;
    let mut last_is_newline = true;
    let mut consecutive_newlines = 0;

    let mut line = 1;
    let mut pos = 1;

    macro_rules! next_pos_and_tp {
    ($amount:expr, $tok:path) => {{
        for _ in current_indent..this_line_indent {
            tokens.push(TokenPos { line, pos, token: Token::Indent });
        }
        for _ in this_line_indent..current_indent {
            tokens.push(TokenPos { line, pos, token: Token::Dedent });
        }
        for _ in 1..consecutive_newlines {
            tokens.push(TokenPos { line, pos, token: Token::NL });
        }

        it.next();
        tokens.push(TokenPos { line, pos, token: $tok });
        pos += $amount;

        current_indent = this_line_indent;
        consecutive_newlines = 0;
        last_is_newline = false;
    }};
    ($fun:path) => {{
        for _ in current_indent..this_line_indent {
            tokens.push(TokenPos { line, pos, token: Token::Indent });
        }
        for _ in this_line_indent..current_indent {
            tokens.push(TokenPos { line, pos, token: Token::Dedent });
        }
        for _ in 1..consecutive_newlines {
            tokens.push(TokenPos { line, pos, token: Token::NL });
        }

        tokens.push(TokenPos { line, pos, token: $fun(&mut it, &mut pos) });

        current_indent = this_line_indent;
        consecutive_newlines = 0;
        last_is_newline = false;
    }}};

    macro_rules! next_line_and_tp { () => {{
        it.next();
        consecutive_newlines += 1;
        line += 1;
        pos = 1;

        if !last_is_newline {
            current_indent = this_line_indent;
            tokens.push(TokenPos { line, pos, token: Token::NL });
        }

        this_line_indent = 0;
        last_is_newline = true;
    }}};

    macro_rules! increase_indent { () => {{
        pos += 4;
        this_line_indent += 1;
    }}};

    while let Some(&c) = it.peek() {
        match c {
            '.' => next_pos_and_tp!(1, Token::Point),
            ':' => {
                match it.peek() {
                    Some(':') => {
                        it.next();
                        next_pos_and_tp!(2, Token::DDoublePoint)
                    }
                    _ => next_pos_and_tp!(1, Token::DoublePoint),
                }
            }
            '_' => next_pos_and_tp!(1, Token::Underscore),
            ',' => next_pos_and_tp!(1, Token::Comma),
            '(' => next_pos_and_tp!(1, Token::LRBrack),
            ')' => next_pos_and_tp!(1, Token::RRBrack),
            '[' => next_pos_and_tp!(1, Token::LSBrack),
            ']' => next_pos_and_tp!(1, Token::RSBrack),
            '{' => next_pos_and_tp!(1, Token::LCBrack),
            '}' => next_pos_and_tp!(1, Token::RCBrack),
            '?' => match it.peek() {
                Some('o') => {
                    it.next();
                    match it.next() {
                        Some('r') => next_pos_and_tp!(3, Token::QuestOr),
                        Some(other) =>
                            return Err(format!("Expected `?or`. Was '{}'.", other)),
                        None => return Err("Expected `?or`.".to_string())
                    }
                }
                _ => next_pos_and_tp!(1, Token::Quest)
            }

            '|' => next_pos_and_tp!(1, Token::Ver),
            '\n' => next_line_and_tp!(),
            '\r' => {
                it.next();
                match it.peek() {
                    Some('\n') => next_line_and_tp!(),
                    Some(other) => return Err(format!("Expected newline after carriage return. \
                    Was '{}'.", other)),
                    None => return Err("File ended with carriage return.".to_string())
                }
            }
            '\t' => increase_indent!(),
            '<' | '>' | '+' | '-' | '*' | '/' | '^' => next_pos_and_tp!(get_operator),
            '0'...'9' => next_pos_and_tp!(get_number),
            '"' => next_pos_and_tp!(get_string),
            'a'...'z' | 'A'...'Z' => next_pos_and_tp!(get_id_or_op),
            '#' => ignore_comment(&mut it),
            ' ' => {
                pos += 1;
                consecutive_spaces += 1;
                if consecutive_spaces == 4 {
                    consecutive_spaces = 0;
                    increase_indent!();
                    pos -= 4;
                }
                it.next();
                continue;
            }
            c => return Err(format!("Unrecognized character: '{}'.", c)),
        }

        consecutive_spaces = 0;
    }

    return Ok(tokens);
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
    return match it.next() {
        Some('<') => match it.peek() {
            Some('=') => next_and!(it, pos, Token::Leq),
            Some('-') => next_and!(it, pos, Token::Assign),
            _ => Token::Le
        }
        Some('>') => match it.peek() {
            Some('=') => next_and!(it, pos, Token::Geq),
            _ => Token::Ge
        }
        Some('+') => Token::Add,
        Some('-') => match it.peek() {
            Some('>') => next_and!(it, pos, Token::To),
            _ => Token::Sub
        }
        Some('/') => Token::Div,
        Some('*') => Token::Mul,
        Some('^') => Token::Pow,
        _ => panic!("get operator received a character it shouldn't have.")
    };
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
            '.' => next_and!(it, pos, { num.push(c); comma = true; }),
            _ => break
        }
    }

    return match (e_found, comma) {
        (true, _) => Token::ENum(num, exp),
        (false, true) => Token::Real(num),
        (false, false) => Token::Int(num)
    };
}

fn get_string(it: &mut Peekable<Chars>, pos: &mut i32) -> Token {
    it.next();
    let mut result = String::new();
    let mut last_backslash = false;

    while let Some(&c) = it.peek() {
        match c {
            '"' => next_and!(it, pos, {
                if !last_backslash { break; } else { result.push(c) };
                last_backslash = false
            }),
            '\\' => next_and!(it, pos, { result.push(c); last_backslash = true }),
            _ => next_and!(it, pos, { result.push(c); last_backslash = false })
        }
    }

    return Token::Str(result);
}

fn get_id_or_op(it: &mut Peekable<Chars>, pos: &mut i32) -> Token {
    let mut result = String::new();
    *pos += 1;

    while let Some(&c) = it.peek() {
        match c {
            'a'...'z' | 'A'...'Z' | '0'...'9' | '_' => next_and!(it, pos, result.push(c)),
            _ => break
        }
    }

    return match result.as_ref() {
        "from" => Token::From,
        "util.md" => Token::Util,
        "type" => Token::Type,
        "as" => Token::As,
        "isa" => Token::IsA,
        "constructor" => Token::Constructor,
        "private" => Token::Private,

        "use" => Token::Use,
        "useall" => Token::UseAll,
        "class" => Token::Class,
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
        "isnt" => Token::IsN,
        "isnta" => Token::IsNA,
        "eq" => Token::Eq,
        "neq" => Token::Neq,
        "mod" => Token::Mod,
        "sqrt" => Token::Sqrt,
        "while" => Token::While,
        "foreach" => Token::For,
        "where" => Token::Where,

        "if" => Token::If,
        "then" => Token::Then,
        "else" => Token::Else,
        "when" => Token::When,
        "do" => Token::Do,
        "continue" => Token::Continue,
        "break" => Token::Break,
        "return" => Token::Ret,

        "in" => Token::In,
        "to" => Token::Range,
        "toincl" => Token::RangeIncl,

        "raises" => Token::Raises,
        "handle" => Token::Handle,
        "retry" => Token::Retry,

        "true" => Token::Bool(true),
        "false" => Token::Bool(false),
        "print" => Token::Print,
        "println" => Token::PrintLn,

        "undefined" => Token::Undefined,
        
        _ => Token::Id(result)
    };
}

use std::iter::Peekable;
use std::str::Chars;

#[derive(PartialEq)]
#[derive(Debug)]
pub enum Token {
    Class,
    As,
    From,
    Use,
    UseAll,

    Fun,
    To,
    Point,
    Comma,
    DoublePoint,

    Id(String),
    Mut,
    Assign,
    Let,

    Real(String),
    Int(String),
    ENum(String, String),
    Str(String),
    Bool(bool),

    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Mod,

    Ge,
    Geq,
    Le,
    Leq,

    Eq,
    Is,
    IsN,
    Neq,
    And,
    Or,
    Not,

    LPar,
    RPar,
    NL,
    Ind,

    Loop,
    While,
    For,
    Where,
    In,
    If,
    Unless,
    When,
    Then,
    Else,
    Do,
    Continue,
    Break,
    Ret,

    Print,
}

#[macro_use]
macro_rules! next_and { ($it:expr, $stmt:stmt) => {{ $it.next(); $stmt }} }

pub fn tokenize(input: String) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut it = input.chars().peekable();
    let mut consecutive_spaces = 0;

    while let Some(c) = it.next() {
        match c {
            '.' => tokens.push(Token::Point),
            ':' => tokens.push(Token::DoublePoint),
            ',' => tokens.push(Token::Comma),
            '(' => tokens.push(Token::LPar),
            ')' => tokens.push(Token::RPar),
            '\n' => tokens.push(Token::NL),
            '\r' => match it.next() {
                Some('\n') => tokens.push(Token::NL),
                Some(other) =>
                    return Err(format!("Expected newline after carriage return. Was {}", other)),
                None => return Err("File ended with carriage return".to_string())
            }
            '\t' => tokens.push(Token::Ind),

            '<' | '>' | '+' | '-' | '*' | '/' | '^' =>
                tokens.push(get_operator(c, &mut it)),
            '0'...'9' => tokens.push(get_number(c, &mut it)),
            '"' => tokens.push(get_string(&mut it)),
            'a'...'z' | 'A'...'Z' => tokens.push(get_id_or_op(c, &mut it)),

            ' ' => {
                consecutive_spaces += 1;
                if consecutive_spaces == 4 {
                    consecutive_spaces = 0;
                    tokens.push(Token::Ind);
                }
                continue;
            }
            c => return Err(format!("Unrecognized character whilst tokenizing: '{}'.", c)),
        }

        consecutive_spaces = 0;
    }

    return Ok(tokens);
}

fn get_operator(current: char, it: &mut Peekable<Chars>) -> Token {
    return match current {
        '<' => match it.peek() {
            Some('=') => next_and!(it, Token::Leq),
            Some('-') => next_and!(it, Token::Assign),
            _ => Token::Le
        }
        '>' => match it.peek() {
            Some('=') => next_and!(it, Token::Geq),
            _ => Token::Ge
        }
        '+' => Token::Add,
        '-' => match it.peek() {
            Some('>') => next_and!(it, Token::To),
            _ => Token::Sub
        }
        '/' => Token::Div,
        '*' => Token::Mul,
        '^' => Token::Pow,
        _ => panic!("get operator received a token it shouldn't have.")
    };
}

fn get_number(current: char, it: &mut Peekable<Chars>) -> Token {
    let mut num = String::new();
    let mut exp = String::new();
    let mut e_found = false;
    let mut comma = false;

    match current {
        '0'...'9' => num.push(current),
        _ => panic!("get number received a token it shouldn't have.")
    }

    while let Some(&c) = it.peek() {
        match c {
            '0'...'9' if !e_found => next_and!(it, num.push(c)),
            '0'...'9' if e_found => next_and!(it, exp.push(c)),

            'e' | 'E' if e_found => break,
            'e' | 'E' => next_and!(it, e_found = true),

            '.' if comma || e_found => break,
            '.' => next_and!(it, { num.push(c); comma = true; }),

            _ => break
        }
    }

    return match (e_found, comma) {
        (true, _) => Token::ENum(num, exp),
        (false, true) => Token::Real(num),
        (false, false) => Token::Int(num)
    };
}

fn get_string(it: &mut Peekable<Chars>) -> Token {
    let mut result = String::new();
    while let Some(c) = it.next() {
        match c {
            '"' => break,
            _ => result.push(c)
        }
    }

    return Token::Str(result);
}

fn get_id_or_op(current: char, it: &mut Peekable<Chars>) -> Token {
    let mut result = String::from(current.to_string());
    while let Some(&c) = it.peek() {
        match c {
            'a'...'z' | 'A'...'Z' | '0'...'9' | '_' => next_and!(it, result.push(c)),
            _ => break
        }
    }

    return match result.as_ref() {
        "as" => Token::As,
        "from" => Token::From,
        "use" => Token::Use,
        "useall" => Token::UseAll,

        "class" => Token::Class,
        "fun" => Token::Fun,

        "let" => Token::Let,
        "mutable" => Token::Mut,

        "and" => Token::And,
        "or" => Token::Or,
        "not" => Token::Not,
        "is" => Token::Is,
        "isnot" => Token::IsN,
        "equals" => Token::Eq,
        "notequals" => Token::Neq,
        "mod" => Token::Mod,

        "loop" => Token::Loop,
        "while" => Token::While,
        "for" => Token::For,
        "where" => Token::Where,
        "in" => Token::In,
        "if" => Token::If,
        "then" => Token::Then,
        "else" => Token::Else,
        "unless" => Token::Unless,
        "when" => Token::When,
        "do" => Token::Do,
        "continue" => Token::Continue,
        "break" => Token::Break,
        "return" => Token::Ret,

        "true" => Token::Bool(true),
        "false" => Token::Bool(false),

        "print" => Token::Print,

        _ => Token::Id(result)
    };
}

#[cfg(test)]
mod test;

use std::iter::Peekable;
use std::str::Chars;

#[derive(PartialEq)]
#[derive(Debug)]
pub enum Token {
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
    NEq,
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

pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut it = input.chars().peekable();

    while let Some(c) = it.next() {
        match c {
            '(' => tokens.push(Token::LPar),
            ')' => tokens.push(Token::RPar),
            '\n' => tokens.push(Token::NL),
            '\t' => tokens.push(Token::Ind),

            '<' | '>' | '+' | '-' | '*' | '/' | '^' =>
                tokens.push(get_operator(c, &mut it)),
            '0'...'9' | '.' | 'e' => tokens.push(get_number(c, &mut it)),
            '"' => tokens.push(get_string(&mut it)),
            'a'...'z' | 'A'...'Z' => tokens.push(get_id_or_op(c, &mut it)),

            ' ' => (),
            c => return Err(format!("Unrecognized character whilst tokenizing: '{}'.", c)),
        }
    }

    return Ok(tokens);
}

fn get_operator(current: char, it: &mut Peekable<Chars>) -> Token {
    return match current {
        '<' => match it.peek() {
            Some('=') => {
                it.next();
                Token::Leq
            }
            Some('-') => {
                it.next();
                Token::Assign
            }
            _ => Token::Le
        }
        '>' => match it.next() {
            Some('=') => {
                it.next();
                Token::Geq
            }
            _ => Token::Ge
        }
        '+' => Token::Add,
        '-' => Token::Sub,
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
        '.' => comma = true,
        'e' => e_found = true,
        _ => panic!("get number received a token it shouldn't have.")
    }

    while let Some(&c) = it.peek() {
        match c {
            '0'...'9' if !e_found => next_and!(it, num.push(c)),
            '0'...'9' if e_found => next_and!(it, exp.push(c)),

            'e' if e_found => break,
            'e' => next_and!(it, e_found = true),

            '.' if comma || e_found => break,
            '.' => {
                num.push(c);
                next_and!(it,comma = true)
            }

            _ => break
        }
    }

    return if e_found {
        if num.is_empty() { num.push('0') }
        Token::ENum(num, exp)
    } else if comma {
        Token::Real(num)
    } else {
        Token::Int(num)
    };
}

fn get_string(it: &mut Peekable<Chars>) -> Token {
    let mut result = String::new();

    while let Some(&c) = it.peek() {
        it.next();
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
            'a'...'z' | 'A'...'Z' | '0'...'9' => {
                it.next();
                result.push(c)
            }
            _ => break
        }
    }

    return match result.as_ref() {
        "let" => Token::Let,
        "mutable" => Token::Mut,

        "and" => Token::And,
        "or" => Token::Or,
        "not" => Token::Not,
        "is" => Token::Is,
        "isnot" => Token::IsN,
        "equals" => Token::Eq,
        "notequals" => Token::NEq,
        "mod" => Token::Mod,

        "loop" => Token::Loop,
        "while" => Token::While,
        "for" => Token::For,
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
mod lexer_tests;

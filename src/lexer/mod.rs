use std::iter::Peekable;
use std::str::Chars;

#[derive(PartialEq)]
#[derive(Debug)]
pub enum Token {
    Id(String),
    Str(String),
    Num(f64),
    Bool(bool),

    Assign,

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
    If,
    Unless,
    When,
    Then,
    Do,
    Continue,
    Break,

    Print
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut it = input.chars().peekable();

    while let Some(c) = it.next() {
        match c {
            '(' => tokens.push(Token::LPar),
            ')' => tokens.push(Token::RPar),
            '\n' => tokens.push(Token::NL),
            '\t' => tokens.push(Token::Ind),

            '<' | '>' | '+' | '-' | '*' | '/' | '^' => match get_operator(c, &mut it) {
                Ok(op) => tokens.push(op),
                Err(err) => return Err(err)
            },
            '0'...'9' => match get_number(c, &mut it) {
                Ok(num) => tokens.push(num),
                Err(err) => return Err(err)
            },
            '"' => match get_string(&mut it) {
                Ok(str) => tokens.push(str),
                Err(err) => return Err(err)
            },
            'a'...'z' | 'A'...'Z' => match get_id_or_op(c, &mut it) {
                Ok(id_or_op) => tokens.push(id_or_op),
                Err(err) => return Err(err)
            },

            ' ' => (),
            c => return Err(format!("Unrecognized character whilst tokenizing: '{}'.", c)),
        }
    }

    return Ok(tokens);
}

fn get_operator(current: char, it: &mut Peekable<Chars>) -> Result<Token, String> {
    return match current {
        '<' => match it.peek() {
            Some('=') => {
                it.next();
                Ok(Token::Leq)
            }
            Some('-') => {
                it.next();
                Ok(Token::Assign)
            }
            _ => Ok(Token::Le)
        }
        '>' => match it.next() {
            Some('=') => {
                it.next();
                Ok(Token::Geq)
            }
            _ => Ok(Token::Ge)
        }
        '+' => Ok(Token::Add),
        '-' => Ok(Token::Sub),
        '/' => Ok(Token::Div),
        '*' => Ok(Token::Mul),
        '^' => Ok(Token::Pow),
        op => Err(format!("Unexpected operator whilst tokenizing: '{}'.", op))
    };
}

fn get_number(current: char, it: &mut Peekable<Chars>) -> Result<Token, String> {
    let mut num = String::from(current.to_string());

    while let Some(&c) = it.peek() {
        match c {
            '0'...'9' | '.' => {
                it.next();
                num.push(c)
            }
            _ => break
        }
    }

    let result = num.parse::<f64>();
    if result.is_err() {
        return Err(format!("Error whilst tokenizing number: '{}'.", result.unwrap_err()));
    } else {
        return Ok(Token::Num(result.unwrap()));
    }
}

fn get_string(it: &mut Peekable<Chars>) -> Result<Token, String> {
    let mut result = String::new();

    while let Some(&c) = it.peek() {
        it.next();
        match c {
            '"' => break,
            _ => result.push(c)
        }
    }

    return Ok(Token::Str(result));
}

fn get_id_or_op(current: char, it: &mut Peekable<Chars>) -> Result<Token, String> {
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

    return Ok(match result.as_ref() {
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
        "if" => Token::If,
        "then" => Token::Then,
        "unless" => Token::Unless,
        "when" => Token::When,
        "do" => Token::Do,
        "continue" => Token::Continue,
        "break" => Token::Break,

        "true" => Token::Bool(true),
        "false" => Token::Bool(false),

        "print" => Token::Print,

        _ => Token::Id(result)
    });
}

#[cfg(test)]
mod lexer_tests;

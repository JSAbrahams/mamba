use std::iter::Peekable;
use std::str::Chars;

#[derive(PartialEq)]
#[derive(Debug)]
pub enum Token {
    Id(String),
    Str(String),
    Num(f64),
    Bool(bool),

    ASSIGN,

    ADD,
    SUB,
    MUL,
    DIV,
    POW,
    MOD,
    GE,
    GEQ,
    LE,
    LEQ,
    EQUALS,
    IS,
    ISNOT,
    NOTEQUALS,
    AND,
    OR,
    NOT,

    LPAREN,
    RPAREN,
    NEWLINE,
    INDENT,

    LOOP,
    WHILE,
    IF,
    UNLESS,
    WHEN,
    THEN,
    DO,
    CONTINUELOOP,
    EXITLOOP,
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut it = input.chars().peekable();

    while let Some(c) = it.next() {
        match c {
            '(' => tokens.push(Token::LPAREN),
            ')' => tokens.push(Token::RPAREN),
            '\n' => tokens.push(Token::NEWLINE),
            '\t' => tokens.push(Token::INDENT),

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
                Ok(Token::LEQ)
            }
            Some('-') => {
                it.next();
                Ok(Token::ASSIGN)
            }
            _ => Ok(Token::LE)
        }
        '>' => match it.next() {
            Some('=') => {
                it.next();
                Ok(Token::GEQ)
            }
            _ => Ok(Token::GE)
        }
        '+' => Ok(Token::ADD),
        '-' => Ok(Token::SUB),
        '/' => Ok(Token::DIV),
        '*' => Ok(Token::MUL),
        '^' => Ok(Token::POW),
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
        "and" => Token::AND,
        "or" => Token::OR,
        "not" => Token::NOT,
        "is" => Token::IS,
        "isnot" => Token::ISNOT,
        "equals" => Token::EQUALS,
        "notequals" => Token::NOTEQUALS,
        "mod" => Token::MOD,

        "loop" => Token::LOOP,
        "while" => Token::WHILE,
        "if" => Token::IF,
        "then" => Token::THEN,
        "unless" => Token::UNLESS,
        "when" => Token::WHEN,
        "do" => Token::DO,
        "continueloop" => Token::CONTINUELOOP,
        "exitloop" => Token::EXITLOOP,

        "true" => Token::Bool(true),
        "false" => Token::Bool(false),

        _ => Token::Id(result)
    });
}

#[cfg(test)]
mod lexer_tests;

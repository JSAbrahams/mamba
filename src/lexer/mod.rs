use std::iter::Peekable;
use std::str::Chars;

#[derive(PartialEq)]
#[derive(Debug)]
pub enum Token {
    Id(String),
    String(String),
    Num(f64),

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
    while let Some(&c) = it.peek() {
        match c {
            '(' => {
                tokens.push(Token::LPAREN);
                it.next();
            }
            ')' => {
                tokens.push(Token::RPAREN);
                it.next();
            }
            '\n' => {
                tokens.push(Token::NEWLINE);
                it.next();
            }
            '\t' => {
                tokens.push(Token::INDENT);
                it.next();
            }

            '<' | '>' | '+' | '-' | '*' => match get_operator(&mut it) {
                Ok(op) => tokens.push(op),
                Err(err) => return Err(err)
            },
            '0'...'9' => match get_number(&mut it) {
                Ok(op) => tokens.push(op),
                Err(err) => return Err(err)
            },
            '"' => match get_string(&mut it) {
                Ok(op) => tokens.push(op),
                Err(err) => return Err(err)
            },
            'a'...'z' | 'A'...'Z' => match get_id_or_op(&mut it) {
                Ok(op) => tokens.push(op),
                Err(err) => return Err(err)
            },

            ' ' => (),
            c => return Err(format!("Unrecognized character whilst tokenizing: {}.", c)),
        }
    }

    return Ok(tokens);
}

fn get_operator(it: &mut Peekable<Chars>) -> Result<Token, String> {
    return match it.next() {
        Some('<') => match it.peek() {
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
        Some('>') => match it.peek() {
            Some('=') => {
                it.next();
                Ok(Token::GEQ)
            }
            _ => Ok(Token::GE)
        }
        Some('+') => Ok(Token::ADD),
        Some('-') => Ok(Token::SUB),
        Some('/') => Ok(Token::DIV),
        Some('*') => Ok(Token::MUL),
        Some('^') => Ok(Token::POW),
        Some(op) => Err(format!("Unexpected operator whilst tokenizing: {}.", op)),
        None => Err("No character found whilst trying to tokenize operator.".to_string())
    };
}

fn get_number(it: &mut Peekable<Chars>) -> Result<Token, String> {
    let mut num = String::new();
    let mut comma = false;

    loop {
        match it.next() {
            Some(c) => match c {
                '.' if (comma) => break,
                '.' => {
                    comma = true;
                    num.push(c);
                }
                '0'...'9' => num.push(c),
                _ => break
            }
            None => break
        }
    }

    let result = num.parse::<f64>();
    if result.is_err() {
        return Err(format!("Error whilst tokenizing number: {}.", result.unwrap_err()));
    } else {
        return Ok(Token::Num(result.unwrap()));
    }
}

fn get_string(it: &mut Peekable<Chars>) -> Result<Token, String> {
    it.next(); // skip " character
    let mut result = String::new();

    loop {
        match it.next() {
            Some(c) => match c {
                '"' => {
                    it.next();
                    break;
                }
                _ => result.push(c)
            }
            None => return Err("Unexpected end of string.".to_string())
        }
    }

    return Ok(Token::String(result));
}

fn get_id_or_op(it: &mut Peekable<Chars>) -> Result<Token, String> {
    let mut result = String::new();

    loop {
        match it.next() {
            Some(c) => match c {
                'a'...'z' | 'A'...'Z' | '0'...'9' => result.push(c),
                _ => break
            }
            None => break
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

        _ => Token::Id(result)
    });
}

#[cfg(test)]
mod lexer_tests;

use std::iter::Peekable;
use std::str::Chars;

enum Token {
    Id(char),
    Num(f64),
    String(String),

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
}

fn token(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();

    let mut it = input.chars().peekable();
    while let Some(&c) = it.peek() {
        match c {
            '(' => tokens.push(Token::LPAREN),
            ')' => tokens.push(Token::RPAREN),
            '\n' => tokens.push(Token::NEWLINE),
            '\t' => tokens.push(Token::INDENT),

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

        it.next();
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
    let mut end = false;

    while !end {
        let next = it.next();
        match next {
            Some(c) =>
                match c {
                    '.' => {
                        if comma { end = true; }
                        comma = true;
                        match it.next() {
                            Some(n) => num.push(n),
                            None => end = true
                        }
                    }
                    '0'...'9' => num.push(c),
                    ' ' => end = true,
                    _ => ()
                }
            None => end = true
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
    let mut end = false;

    while !end {
        let next = it.next();
        match next {
            Some(c) =>
                match c {
                    '"' => end = true,
                    _ => result.push(c)
                }
            None => return Err("Unexpected end of string.".to_string())
        }
    }

    return Ok(Token::String(result));
}

fn get_id_or_op(it: &mut Peekable<Chars>) -> Result<Token, String> {
    let mut end = false;
    let mut result = String::new();

    while !end {
        let next = it.next();
        match next {
            Some(c) =>
                match c {
                    'a'...'z' | 'A'...'Z' | '0'...'9' => result.push(c),
                    ' ' => end = true,
                    _ => end = true
                }
            None => end = true
        }
    }

    return Ok(match result.as_ref() {
        "and" => Token::AND,
        "or" => Token::OR,
        "is" => Token::IS,
        "isnot" => Token::ISNOT,
        "equals" => Token::EQUALS,
        "notequals" => Token::NOTEQUALS,
        "mod" => Token::MOD,

        _ => Token::String(result)
    });
}

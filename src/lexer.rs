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

fn token(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();

    let mut it = input.chars().peekable();
    while let Some(&c) = it.peek() {
        match c {
            '(' => tokens.push(Token::LPAREN),
            ')' => tokens.push(Token::RPAREN),
            '\n' => tokens.push(Token::NEWLINE),
            '\t' => tokens.push(Token::INDENT),

            '<' | '>' | '+' | '-' | '*' => tokens.push(get_operator(&mut it)),
            '0'...'9' => tokens.push(get_number(&mut it)),
            '"' => tokens.push(get_string(&mut it)),

            'a'...'z' | 'A'...'Z' => tokens.push(get_id_or_op(&mut it)),
            ' ' => (),
            _ => (),
        }

        it.next();
    }

    return tokens;
}

fn get_operator(it: &mut Peekable<Chars>) -> Token {
    return match it.next() {
        Some('<') => match it.peek() {
            Some('=') => {
                it.next();
                Token::LEQ
            }
            _ => Token::LE
        }
        Some('>') => match it.peek() {
            Some('=') => {
                it.next();
                Token::GEQ
            }
            _ => Token::GE
        }
        Some('+') => Token::ADD,
        Some('-') => Token::SUB,
        Some('/') => Token::DIV,
        Some('*') => Token::MUL,
        Some('^') => Token::POW,
        Some(_) => {
            print!("error");
            Token::Num(0.0)
        }
        None => {
            print!("error");
            Token::Num(0.0)
        }
    };
}

fn get_number(it: &mut Peekable<Chars>) -> Token {
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
    if result.is_err() { print!("error") }
    return match result.ok() {
        Some(num) => Token::Num(num),
        None => {
            print!("error");
            Token::Num(0.0)
        }
    };
}

fn get_string(it: &mut Peekable<Chars>) -> Token {
    it.next();
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
            None => end = true
        }
    }

//    if it.peek() != '"' { return Err(format!("Unexpected end of string.")); }

    return Token::String(result);
}

fn get_id_or_op(it: &mut Peekable<Chars>) -> Token {
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

    return match result.as_ref() {
        "and" => Token::AND,
        "or" => Token::OR,
        "is" => Token::IS,
        "isnot" => Token::ISNOT,
        "equals" => Token::EQUALS,
        "notequals" => Token::NOTEQUALS,
        "mod" => Token::MOD,

        _ => Token::String(result)
    };
}

use std::fmt;
use std::iter::Peekable;
use std::str::Chars;

#[derive(PartialEq, Debug, Clone)]
pub struct TokenPos {
    pub line: i32,
    pub pos: i32,
    pub token: Token,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Token {
    Class,
    As,
    From,
    Use,
    UseAll,
    Forward,

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

    While,
    For,
    Where,
    Map,
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

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = match *self {
            Token::Class => "'class'",
            Token::As => "'as'",
            Token::From => "'from'",
            Token::Use => "'use'",
            Token::UseAll => "'useall'",
            Token::Forward => "'forward'",

            Token::Fun => "'fun'",
            Token::To => "->",
            Token::Point => ".",
            Token::Comma => ",",
            Token::DoublePoint => "_",

            Token::Id(_) => "<identifier>",
            Token::Mut => "'mutable'",
            Token::Assign => "'<-'",
            Token::Let => "'let'",

            Token::Real(_) => "<real>",
            Token::Int(_) => "<int>",
            Token::ENum(_, _) => "<e-num>",
            Token::Str(_) => "<string>",
            Token::Bool(_) => "<bool>",

            Token::Add => "'+'",
            Token::Sub => "'-'",
            Token::Mul => "'*'",
            Token::Div => "'\\'",
            Token::Pow => "'^'",
            Token::Mod => "'mod'",

            Token::Ge => "'>'",
            Token::Geq => "'>='",
            Token::Le => "'<'",
            Token::Leq => "'<='",

            Token::Eq => "'equals'",
            Token::Is => "'is'",
            Token::IsN => "'isnot'",
            Token::Neq => "'notequal'",
            Token::And => "'and'",
            Token::Or => "'or'",
            Token::Not => "'not'",

            Token::LPar => "'('",
            Token::RPar => "')'",
            Token::NL => "<newline>",
            Token::Ind => "<indent>",

            Token::While => "'while'",
            Token::For => "'for'",
            Token::Where => "'where'",
            Token::Map => "'map'",
            Token::In => "'in'",
            Token::If => "'if'",
            Token::Unless => "'unless'",
            Token::When => "'when'",
            Token::Then => "'then'",
            Token::Else => "'else'",
            Token::Do => "'do'",
            Token::Continue => "'continue'",
            Token::Break => "'break'",
            Token::Ret => "'return'",

            Token::Print => "'print'",
        };

        write!(f, "{}", string)
    }
}

#[macro_use]
macro_rules! next_and { ($it:expr, $pos:expr, $stmt:stmt) => {{ $it.next(); *$pos += 1; $stmt }} }

pub fn tokenize(input: String) -> Result<Vec<TokenPos>, String> {
    let mut tokens = Vec::new();
    let mut it = input.chars().peekable();
    let mut consecutive_spaces = 0;

    let mut line = 1;
    let mut pos = 1;

    while let Some(c) = it.next() {
        match c {
            '.' => tokens.push(TokenPos { line, pos, token: Token::Point }),
            ':' => tokens.push(TokenPos { line, pos, token: Token::DoublePoint }),
            ',' => tokens.push(TokenPos { line, pos, token: Token::Comma }),
            '(' => tokens.push(TokenPos { line, pos, token: Token::LPar }),
            ')' => tokens.push(TokenPos { line, pos, token: Token::RPar }),
            '\n' => {
                line += 1;
                tokens.push(TokenPos { line, pos, token: Token::NL })
            }
            '\r' => match it.next() {
                Some('\n') => {
                    line += 1;
                    tokens.push(TokenPos { line, pos, token: Token::NL })
                }
                Some(other) => return Err(format!("Expected newline after carriage return. Was {}",
                                                  other)),
                None => return Err("File ended with carriage return".to_string())
            }
            '\t' => tokens.push(TokenPos { line, pos, token: Token::Ind }),
            '<' | '>' | '+' | '-' | '*' | '/' | '^' =>
                tokens.push(TokenPos { line, pos, token: get_operator(c, &mut it, &mut pos) }),
            '0'...'9' => tokens.push(TokenPos { line, pos, token: get_number(c, &mut it, &mut pos) }),
            '"' => tokens.push(TokenPos { line, pos, token: get_string(&mut it, &mut pos) }),
            'a'...'z' | 'A'...'Z' => tokens.push(TokenPos { line, pos, token: get_id_or_op(c, &mut it, &mut pos) }),
            ' ' => {
                consecutive_spaces += 1;
                if consecutive_spaces == 4 {
                    consecutive_spaces = 0;
                    tokens.push(TokenPos { line, pos, token: Token::Ind });
                }
                continue;
            }
            c => return Err(format!("Unrecognized character whilst tokenizing: '{}'.", c)),
        }

        pos += 1;
        consecutive_spaces = 0;
    }

    return Ok(tokens);
}

fn get_operator(current: char, it: &mut Peekable<Chars>, pos: &mut i32) -> Token {
    *pos += 1;
    return match current {
        '<' => match it.peek() {
            Some('=') => next_and!(it, pos, Token::Leq),
            Some('-') => next_and!(it, pos, Token::Assign),
            _ => Token::Le
        }
        '>' => match it.peek() {
            Some('=') => next_and!(it, pos, Token::Geq),
            _ => Token::Ge
        }
        '+' => Token::Add,
        '-' => match it.peek() {
            Some('>') => next_and!(it, pos, Token::To),
            _ => Token::Sub
        }
        '/' => Token::Div,
        '*' => Token::Mul,
        '^' => Token::Pow,
        _ => panic!("get operator received a token it shouldn't have.")
    };
}

fn get_number(current: char, it: &mut Peekable<Chars>, pos: &mut i32) -> Token {
    let mut num = String::new();
    let mut exp = String::new();
    let mut e_found = false;
    let mut comma = false;

    *pos += 1;
    match current {
        '0'...'9' => num.push(current),
        _ => panic!("get number received a token it shouldn't have.")
    }

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
    let mut result = String::new();
    while let Some(&c) = it.peek() {
        match c {
            '"' => next_and!(it, pos, break),
            _ => next_and!(it, pos, result.push(c))
        }
    }

    return Token::Str(result);
}

fn get_id_or_op(current: char, it: &mut Peekable<Chars>, pos: &mut i32) -> Token {
    let mut result = String::from(current.to_string());
    while let Some(&c) = it.peek() {
        match c {
            'a'...'z' | 'A'...'Z' | '0'...'9' | '_' => next_and!(it, pos, result.push(c)),
            _ => break
        }
    }

    return match result.as_ref() {
        "as" => Token::As,
        "from" => Token::From,
        "use" => Token::Use,
        "useall" => Token::UseAll,
        "class" => Token::Class,
        "forward" => Token::Forward,

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
        "while" => Token::While,
        "for" => Token::For,
        "where" => Token::Where,
        "map" => Token::Map,

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

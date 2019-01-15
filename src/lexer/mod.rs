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
    Type,
    Class,
    As,
    From,
    Use,
    UseAll,
    Forward,
    Self_,

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
    Sqrt,

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
    LBrack,
    RBrack,
    Ver,

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
        let string_representation = match self.clone() {
            Token::Type => "'type'".to_string(),
            Token::Class => "'class'".to_string(),
            Token::As => "'as'".to_string(),
            Token::From => "'from'".to_string(),
            Token::Use => "'use'".to_string(),
            Token::UseAll => "'useall'".to_string(),
            Token::Forward => "'forward'".to_string(),
            Token::Self_ => "'self'".to_string(),

            Token::Fun => "'fun'".to_string(),
            Token::To => "'->'".to_string(),
            Token::Point => "'.'".to_string(),
            Token::Comma => "'.to_string(),'".to_string(),
            Token::DoublePoint => "':'".to_string(),

            Token::Id(id) => format!("<identifier>: {}", id),
            Token::Mut => "'mutable'".to_string(),
            Token::Assign => "'<-'".to_string(),
            Token::Let => "'let'".to_string(),

            Token::Real(real) =>  format!("<real>: '{}'", real),
            Token::Int(int) =>  format!("<int>: '{}'", int),
            Token::ENum(int, exp) =>  format!("<e-number>: '{}e{}'", int, exp),
            Token::Str(string) => format!("<string>: '{}'", string),
            Token::Bool(boolean) =>  format!("<bool>: '{}'", boolean),

            Token::Add => "'+'".to_string(),
            Token::Sub => "'-'".to_string(),
            Token::Mul => "'*'".to_string(),
            Token::Div => "'/'".to_string(),
            Token::Pow => "'^'".to_string(),
            Token::Mod => "'mod'".to_string(),
            Token::Sqrt => "'sqrt'".to_string(),

            Token::Ge => "'>'".to_string(),
            Token::Geq => "'>='".to_string(),
            Token::Le => "'<'".to_string(),
            Token::Leq => "'<='".to_string(),

            Token::Eq => "'equals'".to_string(),
            Token::Is => "'is'".to_string(),
            Token::IsN => "'isnot'".to_string(),
            Token::Neq => "'notequal'".to_string(),
            Token::And => "'and'".to_string(),
            Token::Or => "'or'".to_string(),
            Token::Not => "'not'".to_string(),

            Token::LPar => "'('".to_string(),
            Token::RPar => "')'".to_string(),
            Token::LBrack => "'['".to_string(),
            Token::RBrack => "']'".to_string(),
            Token::Ver => "'|'".to_string(),

            Token::NL => "<newline>".to_string(),
            Token::Ind => "<indent>".to_string(),

            Token::While => "'while'".to_string(),
            Token::For => "'for'".to_string(),
            Token::Where => "'where'".to_string(),
            Token::Map => "'map'".to_string(),
            Token::In => "'in'".to_string(),
            Token::If => "'if'".to_string(),
            Token::Unless => "'unless'".to_string(),
            Token::When => "'when'".to_string(),
            Token::Then => "'then'".to_string(),
            Token::Else => "'else'".to_string(),
            Token::Do => "'do'".to_string(),
            Token::Continue => "'continue'".to_string(),
            Token::Break => "'break'".to_string(),
            Token::Ret => "'return'".to_string(),

            Token::Print => "'print'".to_string(),
        };

        write!(f, "{}", string_representation)
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
            '.' => {
                tokens.push(TokenPos { line, pos, token: Token::Point });
                pos += 1;
            }
            ':' => {
                tokens.push(TokenPos { line, pos, token: Token::DoublePoint });
                pos += 1;
            }
            ',' => {
                tokens.push(TokenPos { line, pos, token: Token::Comma });
                pos += 1;
            }
            '(' => {
                tokens.push(TokenPos { line, pos, token: Token::LPar });
                pos += 1;
            }
            ')' => {
                tokens.push(TokenPos { line, pos, token: Token::RPar });
                pos += 1;
            }
            '[' => {
                tokens.push(TokenPos { line, pos, token: Token::LBrack });
                pos += 1;
            }
            ']' => {
                tokens.push(TokenPos { line, pos, token: Token::RBrack });
                pos += 1;
            }
            '|' => {
                tokens.push(TokenPos { line, pos, token: Token::Ver });
                pos += 1;
            }
            '\n' => {
                tokens.push(TokenPos { line, pos, token: Token::NL });
                line += 1;
                pos = 1;
            }
            '\r' => match it.next() {
                Some('\n') => {
                    tokens.push(TokenPos { line, pos, token: Token::NL });
                    line += 1;
                    pos = 1;
                }
                Some(other) => return Err(format!("Expected newline after carriage return. Was {}",
                                                  other)),
                None => return Err("File ended with carriage return".to_string())
            }
            '\t' => tokens.push(TokenPos { line, pos, token: Token::Ind }),
            '<' | '>' | '+' | '-' | '*' | '/' | '^' =>
                tokens.push(TokenPos { line, pos, token: get_operator(c, &mut it, &mut pos) }),
            '0'...'9' =>
                tokens.push(TokenPos { line, pos, token: get_number(c, &mut it, &mut pos) }),
            '"' => tokens.push(TokenPos { line, pos, token: get_string(&mut it, &mut pos) }),
            'a'...'z' | 'A'...'Z' =>
                tokens.push(TokenPos { line, pos, token: get_id_or_op(c, &mut it, &mut pos) }),
            ' ' => {
                pos += 1;
                consecutive_spaces += 1;
                if consecutive_spaces == 4 {
                    consecutive_spaces = 0;
                    tokens.push(TokenPos { line, pos, token: Token::Ind });
                }
                continue;
            }
            c => return Err(format!("Unrecognized character whilst tokenizing: '{}'.", c)),
        }

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
        _ => panic!("get operator received a character it shouldn't have.")
    };
}

fn get_number(current: char, it: &mut Peekable<Chars>, pos: &mut i32) -> Token {
    let mut num = String::new();
    let mut exp = String::new();
    let mut e_found = false;
    let mut comma = false;

    match current {
        '0'...'9' => num.push(current),
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
    *pos += 1;

    while let Some(&c) = it.peek() {
        match c {
            'a'...'z' | 'A'...'Z' | '0'...'9' | '_' => next_and!(it, pos, result.push(c)),
            _ => break
        }
    }

    return match result.as_ref() {
        "type" => Token::Type,
        "as" => Token::As,
        "from" => Token::From,
        "use" => Token::Use,
        "useall" => Token::UseAll,
        "class" => Token::Class,
        "forward" => Token::Forward,
        "self" => Token::Self_,

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
        "sqrt" => Token::Sqrt,
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

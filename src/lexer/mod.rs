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
    Util,
    IsA,
    Constructor,

    As,
    Use,
    UseAll,
    Forward,
    _Self,

    Fun,
    Point,
    Comma,
    DoublePoint,
    DDoublePoint,

    Id(String),
    Mut,
    Assign,
    Def,

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
    LCurl,
    RCurl,
    Ver,

    NL,
    Indent,
    Dedent,

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
            Token::Util => "'util'".to_string(),
            Token::Type => "'type'".to_string(),
            Token::Class => "'class'".to_string(),
            Token::IsA => "'isa'".to_string(),
            Token::Constructor => "'constructor'".to_string(),

            Token::As => "'as'".to_string(),
            Token::Use => "'use'".to_string(),
            Token::UseAll => "'useall'".to_string(),
            Token::Forward => "'forward'".to_string(),
            Token::_Self => "'self'".to_string(),

            Token::Fun => "'fun'".to_string(),
            Token::Point => "'.'".to_string(),
            Token::Comma => "'.to_string(),'".to_string(),
            Token::DoublePoint => "':'".to_string(),
            Token::DDoublePoint => "'::'".to_string(),

            Token::Id(id) => format!("<identifier>: {}", id),
            Token::Mut => "'mutable'".to_string(),
            Token::Assign => "'<-'".to_string(),
            Token::Def => "'def'".to_string(),

            Token::Real(real) => format!("<real>: '{}'", real),
            Token::Int(int) => format!("<int>: '{}'", int),
            Token::ENum(int, exp) => format!("<e-number>: '{}e{}'", int, exp),
            Token::Str(string) => format!("<string>: '{}'", string),
            Token::Bool(boolean) => format!("<bool>: '{}'", boolean),

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
            Token::LCurl => "'{'".to_string(),
            Token::RCurl => "'}'".to_string(),
            Token::Ver => "'|'".to_string(),

            Token::NL => "<newline>".to_string(),
            Token::Indent => "<indent>".to_string(),
            Token::Dedent => "<dedent>".to_string(),

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

    let mut current_indent = 0;
    let mut this_line_indent = 0;
    let mut consecutive_spaces = 0;

    let mut line = 1;
    let mut pos = 1;

    macro_rules! next_pos_and_tp { ($amount:expr, $tok:path) => {{
        let tp = TokenPos { line, pos, token: $tok };
        pos += $amount;
        tp
    }} }

    while let Some(c) = it.next() {
        match c {
            '.' => tokens.push(next_pos_and_tp!(1, Token::Point)),
            ':' => match it.peek() {
                Some(':') => {
                    it.next();
                    tokens.push(next_pos_and_tp!(2, Token::DDoublePoint))
                }
                _ => tokens.push(next_pos_and_tp!(1, Token::DoublePoint)),
            }
            ',' => tokens.push(next_pos_and_tp!(1, Token::Comma)),
            '(' => tokens.push(next_pos_and_tp!(1, Token::LPar)),
            ')' => tokens.push(next_pos_and_tp!(1, Token::RPar)),
            '[' => tokens.push(next_pos_and_tp!(1, Token::LBrack)),
            ']' => tokens.push(next_pos_and_tp!(1, Token::RBrack)),
            '{' => tokens.push(next_pos_and_tp!(1, Token::LCurl)),
            '}' => tokens.push(next_pos_and_tp!(1, Token::RCurl)),
            '|' => tokens.push(next_pos_and_tp!(1, Token::Ver)),
            '\n' => {
                tokens.push(next_pos_and_tp!(1, Token::NL));
                line += 1;
                pos = 1;
            }
            '\r' => match it.next() {
                Some('\n') => {
                    tokens.push(next_pos_and_tp!(1, Token::NL));
                    line += 1;
                    pos = 1;
                }
                Some(other) =>
                    return Err(format!("Expected newline after carriage return. Was {}", other)),
                None => return Err("File ended with carriage return".to_string())
            }
            '\t' => this_line_indent += 1,
            '<' | '>' | '+' | '-' | '*' | '/' | '^' =>
                tokens.push(TokenPos { line, pos, token: get_operator(c, &mut it, &mut pos) }),
            '0'...'9' =>
                tokens.push(TokenPos { line, pos, token: get_number(c, &mut it, &mut pos) }),
            '"' => tokens.push(TokenPos { line, pos, token: get_string(&mut it, &mut pos) }),
            'a'...'z' | 'A'...'Z' =>
                tokens.push(TokenPos { line, pos, token: get_id_or_op(c, &mut it, &mut pos) }),
            '#' => ignore_comment(&mut it),
            ' ' => {
                pos += 1;
                consecutive_spaces += 1;
                if consecutive_spaces == 4 {
                    consecutive_spaces = 0;
                    current_indent += 0;
                    this_line_indent += 1;
                }
                continue;
            }
            c => return Err(format!("Unrecognized character whilst tokenizing: '{}'.", c)),
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
        '-' => Token::Sub,
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
        "util" => Token::Util,
        "type" => Token::Type,
        "as" => Token::As,
        "isa" => Token::IsA,
        "constructor" => Token::Constructor,

        "use" => Token::Use,
        "useall" => Token::UseAll,
        "class" => Token::Class,
        "forward" => Token::Forward,
        "self" => Token::_Self,

        "fun" => Token::Fun,
        "def" => Token::Def,
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

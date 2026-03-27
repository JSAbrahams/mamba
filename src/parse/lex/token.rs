use std::cmp::max;
use std::fmt;

use crate::common::position::{CaretPos, Position};

#[derive(PartialEq, Debug, Clone)]
pub struct Lex {
    pub pos: Position,
    pub token: Token,
}

impl Lex {
    pub fn new(start: CaretPos, token: Token) -> Self {
        let end = if let Token::Str(_str, _) = &token {
            start.offset_line(max((_str.lines().count() as i32 - 1) as usize, 0))
        } else if let Token::DocStr(_str) = &token {
            start.offset_line(max((_str.lines().count() as i32 - 1) as usize, 0))
        } else {
            start
        };

        let end = end.offset_pos(token.clone().width());
        let pos = Position { start, end };
        Lex { pos, token }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Token {
    From,
    Type,
    Class,
    Pure,

    As,
    Import,
    Forward,

    Point,
    Comma,
    DoublePoint,
    BSlash,

    Id(String),
    Fin,
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    PowAssign,
    Def,

    Real(String),
    Int(String),
    ENum(String, String),
    Str(String, Vec<Vec<Lex>>),
    DocStr(String),

    Range,
    RangeIncl,
    Slice,
    SliceIncl,

    Add,
    Sub,
    Mul,
    Div,
    FDiv,
    Pow,
    Mod,
    Sqrt,

    Ge,
    Geq,
    Le,
    Leq,

    Eq,
    Neq,
    And,
    Or,
    Not,

    LRBrack,
    RRBrack,
    LSBrack,
    RSBrack,
    LCBrack,
    RCBrack,
    Ver,
    To,
    BTo,

    NL,
    Underscore,

    Raise,
    When,

    While,
    For,
    In,
    If,
    Then,
    Match,
    Else,
    Do,
    Continue,
    Break,
    Ret,
    With,

    Question,
    Pass,
}

/// Name structure, used to give a more descriptive name of a token.
/// Useful in debug or error messages, where we don't only want to print the token itself, but instead a description.
pub struct TokenName<'a>(&'a Token);

impl Token {
    pub fn width(&self) -> usize {
        self.to_string().len()
    }

    pub fn same_type(left: &Token, right: &Token) -> bool {
        match (left.clone(), right.clone()) {
            (Token::Id(_), Token::Id(_)) => true,
            (Token::Real(_), Token::Real(_)) => true,
            (Token::Int(_), Token::Int(_)) => true,
            (Token::Str(..), Token::Str(..)) => true,
            (Token::DocStr(_), Token::DocStr(_)) => true,
            (Token::ENum(..), Token::ENum(..)) => true,
            _ => left == right,
        }
    }

    /// Similar to `Display`, except that it writes a more descriptive name.
    /// Useful in say error or debug messages, where you don't want to literally print the token but what it is.
    /// In many cases, similar to display.
    pub fn name<'a>(&'a self) -> TokenName<'a> {
        TokenName(self)
    }

    /// Quick check to see if token name is not equal to self.
    /// Used in context of error message generation.
    pub fn equals_name(&self) -> bool {
        format!("{self}") == format!("{}", self.name())
    }
}

impl fmt::Display for TokenName<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            Token::Assign => write!(f, "assign"),
            Token::AddAssign => write!(f, "add assign"),
            Token::SubAssign => write!(f, "sub assign"),
            Token::MulAssign => write!(f, "mul assign"),
            Token::PowAssign => write!(f, "pow assign"),
            Token::DivAssign => write!(f, "div assign"),

            Token::Id(_) => write!(f, "identifier"),
            Token::Real(_) => write!(f, "real literal"),
            Token::Int(_) => write!(f, "integer literal"),
            Token::Str(..) => write!(f, "string literal"),
            Token::DocStr(_) => write!(f, "doc string"),
            Token::ENum(..) => write!(f, "enum variant"),

            Token::Range => write!(f, "range"),
            Token::RangeIncl => write!(f, "range inclusive"),
            Token::Slice => write!(f, "slice"),
            Token::SliceIncl => write!(f, "slice inclusive"),

            Token::Add => write!(f, "add"),
            Token::Sub => write!(f, "sub"),
            Token::Mul => write!(f, "mul"),
            Token::Div => write!(f, "div"),
            Token::FDiv => write!(f, "floor div"),
            Token::Pow => write!(f, "pow"),

            Token::Ge => write!(f, "greater than"),
            Token::Geq => write!(f, "greater than or equal to"),
            Token::Le => write!(f, "less than"),
            Token::Leq => write!(f, "less than or equal to"),

            Token::Eq => write!(f, "equal"),
            Token::Neq => write!(f, "not equal"),

            Token::LRBrack => write!(f, "left roung bracket"),
            Token::RRBrack => write!(f, "right rount bracket"),
            Token::LSBrack => write!(f, "left square bracket"),
            Token::RSBrack => write!(f, "right square bracket"),
            Token::LCBrack => write!(f, "left curly bracket"),
            Token::RCBrack => write!(f, "right curly bracket"),
            Token::Ver => write!(f, "vertial"),
            Token::To => write!(f, "to"),
            Token::BTo => write!(f, "broad to"),

            Token::NL => write!(f, "newline"),
            Token::Underscore => write!(f, "underscore"),

            Token::Question => write!(f, "question"),
            Token::Raise => write!(f, "raise"),

            _ => write!(f, "{}", self.0),
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.clone() {
            Token::From => write!(f, "from"),
            Token::Pure => write!(f, "pure"),
            Token::Type => write!(f, "type"),
            Token::Class => write!(f, "class"),

            Token::As => write!(f, "as"),
            Token::Import => write!(f, "import"),
            Token::Forward => write!(f, "forward"),

            Token::Point => write!(f, "."),
            Token::Comma => write!(f, ","),
            Token::DoublePoint => write!(f, ":"),
            Token::BSlash => write!(f, "\\"),

            Token::Fin => write!(f, "fin"),
            Token::Assign => write!(f, ":="),
            Token::AddAssign => write!(f, "+="),
            Token::SubAssign => write!(f, "-="),
            Token::MulAssign => write!(f, "*="),
            Token::PowAssign => write!(f, "^="),
            Token::DivAssign => write!(f, "/="),
            Token::Def => write!(f, "def"),

            Token::Id(id) => write!(f, "{id}"),
            Token::Real(real) => write!(f, "{real}"),
            Token::Int(int) => write!(f, "{int}"),
            Token::ENum(base, exp) => write!(f, "{base}E{exp}"),
            Token::Str(string, _) => write!(f, "\"{string}\""),
            Token::DocStr(docstr) => write!(f, "##{docstr}"),

            Token::Range => write!(f, ".."),
            Token::RangeIncl => write!(f, "..="),
            Token::Slice => write!(f, "::"),
            Token::SliceIncl => write!(f, "::="),

            Token::Add => write!(f, "+"),
            Token::Sub => write!(f, "-"),
            Token::Mul => write!(f, "*"),
            Token::Div => write!(f, "/"),
            Token::FDiv => write!(f, "//"),
            Token::Pow => write!(f, "^"),
            Token::Mod => write!(f, "mod"),
            Token::Sqrt => write!(f, "sqrt"),

            Token::Ge => write!(f, ">"),
            Token::Geq => write!(f, ">="),
            Token::Le => write!(f, "<"),
            Token::Leq => write!(f, "<="),

            Token::Eq => write!(f, "="),
            Token::Neq => write!(f, "!="),
            Token::And => write!(f, "and"),
            Token::Or => write!(f, "or"),
            Token::Not => write!(f, "not"),

            Token::LRBrack => write!(f, "("),
            Token::RRBrack => write!(f, ")"),
            Token::LSBrack => write!(f, "["),
            Token::RSBrack => write!(f, "]"),
            Token::LCBrack => write!(f, "{{"),
            Token::RCBrack => write!(f, "}}"),
            Token::Ver => write!(f, "|"),
            Token::To => write!(f, "->"),
            Token::BTo => write!(f, "=>"),

            Token::NL => write!(f, ""),
            Token::Underscore => write!(f, "_"),

            Token::While => write!(f, "while"),
            Token::For => write!(f, "for"),
            Token::In => write!(f, "in"),
            Token::If => write!(f, "if"),
            Token::Then => write!(f, "then"),
            Token::Match => write!(f, "match"),
            Token::Else => write!(f, "else"),
            Token::Continue => write!(f, "continue"),
            Token::Break => write!(f, "break"),
            Token::Ret => write!(f, "return"),
            Token::Do => write!(f, "do"),
            Token::With => write!(f, "with"),

            Token::Question => write!(f, "?"),

            Token::Raise => write!(f, "!"),
            Token::When => write!(f, "when"),

            Token::Pass => write!(f, "pass"),
        }
    }
}

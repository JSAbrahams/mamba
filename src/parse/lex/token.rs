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
    IsA,
    IsNA,

    As,
    Import,
    Forward,
    _Self,

    Point,
    Comma,
    DoublePoint,
    Vararg,
    BSlash,

    Id(String),
    Fin,
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    PowAssign,
    BLShiftAssign,
    BRShiftAssign,
    Def,

    Real(String),
    Int(String),
    ENum(String, String),
    Str(String, Vec<Vec<Lex>>),
    DocStr(String),
    Bool(bool),

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

    BAnd,
    BOr,
    BXOr,
    BOneCmpl,
    BLShift,
    BRShift,

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
    Indent,
    Dedent,
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
    Handle,

    Pass,
    Undefined,
    Comment(String),

    Eof,
}

impl Token {
    pub fn width(&self) -> usize {
        self.to_string().len()
    }

    pub fn same_type(left: &Token, right: &Token) -> bool {
        match (left.clone(), right.clone()) {
            (Token::Id(_), Token::Id(_)) => true,
            (Token::Real(_), Token::Real(_)) => true,
            (Token::Int(_), Token::Int(_)) => true,
            (Token::Bool(_), Token::Bool(_)) => true,
            (Token::Str(..), Token::Str(..)) => true,
            (Token::DocStr(_), Token::DocStr(_)) => true,
            (Token::ENum(..), Token::ENum(..)) => true,
            _ => left == right,
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
            Token::IsA => write!(f, "is_a"),
            Token::IsNA => write!(f, "isn_t_a"),

            Token::As => write!(f, "as"),
            Token::Import => write!(f, "import"),
            Token::Forward => write!(f, "forward"),
            Token::_Self => write!(f, "self"),

            Token::Point => write!(f, "."),
            Token::Comma => write!(f, ","),
            Token::DoublePoint => write!(f, ":"),
            Token::Vararg => write!(f, "vararg"),
            Token::BSlash => write!(f, "\\"),

            Token::Fin => write!(f, "fin"),
            Token::Assign => write!(f, "="),
            Token::AddAssign => write!(f, "+="),
            Token::SubAssign => write!(f, "-="),
            Token::MulAssign => write!(f, "*="),
            Token::PowAssign => write!(f, "^="),
            Token::DivAssign => write!(f, "/="),
            Token::BLShiftAssign => write!(f, "<<="),
            Token::BRShiftAssign => write!(f, ">>="),
            Token::Def => write!(f, "def"),

            Token::Id(id) => write!(f, "{id}"),
            Token::Real(real) => write!(f, "{real}"),
            Token::Int(int) => write!(f, "{int}"),
            Token::ENum(base, exp) => write!(f, "{base}E{exp}"),
            Token::Str(string, _) => write!(f, "{string}"),
            Token::DocStr(docstr) => write!(f, "{docstr}"),
            Token::Bool(b) => write!(f, "{b}"),

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

            Token::BAnd => write!(f, "_and_"),
            Token::BOr => write!(f, "_or_"),
            Token::BXOr => write!(f, "_xor_"),
            Token::BOneCmpl => write!(f, "_not_"),
            Token::BLShift => write!(f, "<<"),
            Token::BRShift => write!(f, ">>"),

            Token::Ge => write!(f, ">"),
            Token::Geq => write!(f, ">="),
            Token::Le => write!(f, "<"),
            Token::Leq => write!(f, "<="),

            Token::Eq => write!(f, "eq"),
            Token::Is => write!(f, "is"),
            Token::IsN => write!(f, "isn_t"),
            Token::Neq => write!(f, "neq"),
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
            Token::Indent => write!(f, "    "),
            Token::Dedent => write!(f, ""),
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

            Token::Handle => write!(f, "handle"),
            Token::Raise => write!(f, "raise"),
            Token::When => write!(f, "when"),

            Token::Pass => write!(f, "pass"),
            Token::Undefined => write!(f, "undefined"),
            Token::Comment(comment) => write!(f, "# {comment}"),

            Token::Eof => write!(f, ""),
        }
    }
}

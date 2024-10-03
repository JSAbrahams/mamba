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
    pub fn width(self) -> usize {
        match self {
            Token::Id(id) => id.len(),
            Token::Real(real) => real.len(),
            Token::Int(int) => int.len(),
            Token::Bool(true) => 4,
            Token::Bool(false) => 5,
            Token::Str(string, _) => string.len() + 2,
            Token::DocStr(string) => string.len() + 6,
            Token::ENum(num, exp) => num.len() + 1 + exp.len(),
            Token::From => 4,
            Token::Type => 4,
            Token::Class => 5,
            Token::Pure => 4,
            Token::IsA => 3,
            Token::IsNA => 5,
            Token::As => 2,
            Token::Import => 6,
            Token::Forward => 7,
            Token::_Self => 4,
            Token::Point => 1,
            Token::Comma => 1,
            Token::DoublePoint => 1,
            Token::Vararg => 6,
            Token::BSlash => 1,
            Token::Fin => 3,
            Token::Assign => 2,
            Token::AddAssign | Token::SubAssign => 2,
            Token::MulAssign | Token::DivAssign | Token::PowAssign => 2,
            Token::BLShiftAssign | Token::BRShiftAssign => 3,
            Token::Def => 3,
            Token::Range | Token::Slice => 2,
            Token::RangeIncl | Token::SliceIncl => 3,
            Token::Add | Token::Sub => 1,
            Token::Mul | Token::Div | Token::Pow => 1,
            Token::FDiv => 2,
            Token::Mod => 3,
            Token::Sqrt => 4,
            Token::BAnd => 5,
            Token::BOr => 4,
            Token::BXOr => 5,
            Token::BOneCmpl => 5,
            Token::BLShift | Token::BRShift => 2,
            Token::Ge | Token::Le => 1,
            Token::Geq | Token::Leq => 2,
            Token::Eq => 1,
            Token::Is => 2,
            Token::IsN => 4,
            Token::Neq => 2,
            Token::And => 3,
            Token::Or => 2,
            Token::Not => 3,
            Token::LRBrack | Token::RRBrack => 1,
            Token::LSBrack | Token::RSBrack => 1,
            Token::LCBrack | Token::RCBrack => 1,
            Token::Ver => 1,
            Token::To => 2,
            Token::BTo => 2,
            Token::NL | Token::Indent | Token::Dedent => 0,
            Token::Underscore => 1,
            Token::Raise => 5,
            Token::When => 4,
            Token::While => 5,
            Token::For => 3,
            Token::In => 2,
            Token::If => 2,
            Token::Then => 4,
            Token::Match => 5,
            Token::Else => 4,
            Token::Do => 2,
            Token::Continue => 8,
            Token::Break => 5,
            Token::Ret => 6,
            Token::With => 4,
            Token::Question => 1,
            Token::Handle => 5,
            Token::Pass => 4,
            Token::Undefined => 4,
            Token::Comment(comment) => comment.len(),
            Token::Eof => 0
        }
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
            _ => left == right
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self.clone() {
            Token::From => String::from("from"),
            Token::Pure => String::from("pure"),
            Token::Type => String::from("type"),
            Token::Class => String::from("class"),
            Token::IsA => String::from("is_a"),
            Token::IsNA => String::from("isn_t_a"),

            Token::As => String::from("as"),
            Token::Import => String::from("import"),
            Token::Forward => String::from("forward"),
            Token::_Self => String::from("self"),

            Token::Point => String::from("point"),
            Token::Comma => String::from("comma"),
            Token::DoublePoint => String::from("d_point"),
            Token::Vararg => String::from("vararg"),
            Token::BSlash => String::from("b_slash"),

            Token::Fin => String::from("final"),
            Token::Assign => String::from("assign"),
            Token::AddAssign => String::from("add_assign"),
            Token::SubAssign => String::from("sub_assign"),
            Token::MulAssign => String::from("mul_assign"),
            Token::PowAssign => String::from("pow_assign"),
            Token::DivAssign => String::from("div_assign"),
            Token::BLShiftAssign => String::from("blshift_assign"),
            Token::BRShiftAssign => String::from("brshift_assign"),
            Token::Def => String::from("define"),

            Token::Id(_) => String::from("identifier"),
            Token::Real(_) => String::from("real_lit"),
            Token::Int(_) => String::from("int_lit"),
            Token::ENum(_, _) => String::from("enum_lit"),
            Token::Str(_, _) => String::from("string_lit"),
            Token::DocStr(_) => String::from("doc_string"),
            Token::Bool(_) => String::from("bool"),

            Token::Range => String::from("range"),
            Token::RangeIncl => String::from("range_incl"),
            Token::Slice => String::from("slice"),
            Token::SliceIncl => String::from("slice_incl"),

            Token::Add => String::from("add"),
            Token::Sub => String::from("sub"),
            Token::Mul => String::from("mul"),
            Token::Div => String::from("div"),
            Token::FDiv => String::from("f_div"),
            Token::Pow => String::from("pow"),
            Token::Mod => String::from("mod"),
            Token::Sqrt => String::from("sqrt"),

            Token::BAnd => String::from("bin_add"),
            Token::BOr => String::from("bin_or"),
            Token::BXOr => String::from("bin_xor"),
            Token::BOneCmpl => String::from("bin_not"),
            Token::BLShift => String::from("bin_lshift"),
            Token::BRShift => String::from("bin_rshift"),

            Token::Ge => String::from("ge"),
            Token::Geq => String::from("geq"),
            Token::Le => String::from("le"),
            Token::Leq => String::from("leq"),

            Token::Eq => String::from("eq"),
            Token::Is => String::from("is"),
            Token::IsN => String::from("isn_t"),
            Token::Neq => String::from("neq"),
            Token::And => String::from("and"),
            Token::Or => String::from("or"),
            Token::Not => String::from("not"),

            Token::LRBrack => String::from("l_r_brack"),
            Token::RRBrack => String::from("r_r_brack"),
            Token::LSBrack => String::from("l_s_brack"),
            Token::RSBrack => String::from("r_s_brack"),
            Token::LCBrack => String::from("l_c_brack"),
            Token::RCBrack => String::from("r_c_brack"),
            Token::Ver => String::from("vertical"),
            Token::To => String::from("to"),
            Token::BTo => String::from("fun_to"),

            Token::NL => String::from("newline"),
            Token::Indent => String::from("indent"),
            Token::Dedent => String::from("dedent"),
            Token::Underscore => String::from("underscore"),

            Token::While => String::from("while"),
            Token::For => String::from("for"),
            Token::In => String::from("in"),
            Token::If => String::from("if"),
            Token::Then => String::from("then"),
            Token::Match => String::from("match"),
            Token::Else => String::from("else"),
            Token::Continue => String::from("continue"),
            Token::Break => String::from("break"),
            Token::Ret => String::from("return"),
            Token::Do => String::from("do"),
            Token::With => String::from("with"),

            Token::Question => String::from("question"),

            Token::Handle => String::from("handle"),
            Token::Raise => String::from("raise"),
            Token::When => String::from("when"),

            Token::Pass => String::from("pass"),
            Token::Undefined => String::from("undefined"),
            Token::Comment(_) => String::from("comment"),

            Token::Eof => String::from("end_of_file")
        })
    }
}

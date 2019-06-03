use std::fmt;

#[derive(PartialEq, Debug, Clone)]
pub struct TokenPos {
    pub st_line: i32,
    pub st_pos:  i32,
    pub token:   Token
}

#[derive(PartialEq, Debug, Clone)]
pub enum Token {
    From,
    Type,
    Class,
    Pure,
    IsA,
    IsNA,
    Private,
    Init,

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
    Mut,
    OfMut,
    Assign,
    Def,

    Real(String),
    Int(String),
    ENum(String, String),
    Str(String),
    Bool(bool),
    Range,
    RangeIncl,

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

    Raises,
    Raise,
    Retry,
    When,

    While,
    For,
    Step,
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

    Quest,
    QuestOr,
    QuestCall,
    Handle,

    Print,

    Pass,
    Undefined,
    Comment(String),
    DocString(String),
    Annotation(String)
}

impl Token {
    pub fn width(self) -> i32 {
        (match self {
            Token::Id(id) => id.len(),
            Token::Real(real) => real.len(),
            Token::Int(int) => int.len(),
            Token::Bool(true) => 4,
            Token::Bool(false) => 5,
            Token::Str(_str) => _str.len(),
            Token::ENum(num, exp) => num.len() + 1 + exp.len(),
            other => format!("{}", other).len()
        } as i32)
    }

    pub fn same_type(left: Token, right: Token) -> bool {
        match (left.clone(), right.clone()) {
            (Token::Id(_), Token::Id(_)) => true,
            (Token::Real(_), Token::Real(_)) => true,
            (Token::Int(_), Token::Int(_)) => true,
            (Token::Bool(_), Token::Bool(_)) => true,
            (Token::Str(_), Token::Str(_)) => true,
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
            Token::IsA => String::from("isa"),
            Token::IsNA => String::from("isnta"),
            Token::Private => String::from("private"),
            Token::Init => String::from("init"),

            Token::As => String::from("as"),
            Token::Import => String::from("import"),
            Token::Forward => String::from("forward"),
            Token::_Self => String::from("self"),

            Token::Point => String::from("."),
            Token::Comma => String::from(")"),
            Token::DoublePoint => String::from(":"),
            Token::Vararg => String::from("vararg"),
            Token::BSlash => String::from("\\"),

            Token::Mut => String::from("mut"),
            Token::OfMut => String::from("ofmut"),
            Token::Assign => String::from("<-"),
            Token::Def => String::from("def"),

            Token::Id(id) => format!("{} (identifier)", id),
            Token::Real(real) => format!("{} (real)", real),
            Token::Int(int) => format!("{} (integer)", int),
            Token::ENum(int, exp) => format!("{}e{} (e-number)", int, exp),
            Token::Str(string) => format!("{} (string)", string),
            Token::Bool(boolean) => format!("{} (boolean)", boolean),
            Token::Range => String::from(".."),
            Token::RangeIncl => String::from("..="),

            Token::Add => String::from("+"),
            Token::Sub => String::from("-"),
            Token::Mul => String::from("*"),
            Token::Div => String::from("/"),
            Token::FDiv => String::from("//"),
            Token::Pow => String::from("^"),
            Token::Mod => String::from("mod"),
            Token::Sqrt => String::from("sqrt"),

            Token::BAnd => String::from("_and_"),
            Token::BOr => String::from("_or_"),
            Token::BXOr => String::from("_xor_"),
            Token::BOneCmpl => String::from("_not_"),
            Token::BLShift => String::from("<<"),
            Token::BRShift => String::from(">>"),

            Token::Ge => String::from(">"),
            Token::Geq => String::from(">="),
            Token::Le => String::from("<"),
            Token::Leq => String::from("<="),

            Token::Eq => String::from("="),
            Token::Is => String::from("is"),
            Token::IsN => String::from("isnt"),
            Token::Neq => String::from("/="),
            Token::And => String::from("and"),
            Token::Or => String::from("or"),
            Token::Not => String::from("not"),

            Token::LRBrack => String::from("("),
            Token::RRBrack => String::from(")"),
            Token::LSBrack => String::from("["),
            Token::RSBrack => String::from("]"),
            Token::LCBrack => String::from("{"),
            Token::RCBrack => String::from("}"),
            Token::Ver => String::from("|"),
            Token::To => String::from("->"),
            Token::BTo => String::from("=>"),

            Token::NL => String::from("<newline>"),
            Token::Indent => String::from("<indent>"),
            Token::Dedent => String::from("<dedent>"),
            Token::Underscore => String::from("_"),

            Token::While => String::from("while"),
            Token::For => String::from("for"),
            Token::Step => String::from("step"),
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

            Token::Quest => String::from("?"),
            Token::QuestOr => String::from("?or"),
            Token::QuestCall => String::from("?."),

            Token::Handle => String::from("handle"),
            Token::Raises => String::from("raises"),
            Token::Raise => String::from("raise"),
            Token::Retry => String::from("retry"),
            Token::When => String::from("when"),

            Token::Pass => String::from("pass"),
            Token::Print => String::from("print"),
            Token::Undefined => String::from("undefined"),
            Token::Comment(string) => format!("{} (comment)", string),
            Token::DocString(string) => format!("{} (comment)", string),
            Token::Annotation(string) => format!("@{}", string)
        })
    }
}

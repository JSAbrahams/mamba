use std::fmt;

#[derive(PartialEq, Debug, Clone)]
pub struct TokenPos {
    pub line:  i32,
    pub pos:   i32,
    pub token: Token
}

#[derive(PartialEq, Debug, Clone)]
pub enum Token {
    From,
    Type,
    Class,
    Stateless,
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
    Retry,
    When,

    While,
    For,
    Map,
    In,
    If,
    Then,
    Match,
    With,
    Else,
    Do,
    Continue,
    Break,
    Ret,

    Quest,
    QuestOr,
    QuestCall,
    Handle,

    Print,

    Pass,
    Undefined,
    Comment(String)
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self.clone() {
            Token::From => String::from("from"),
            Token::Stateless => String::from("stateless"),
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
            Token::Comma => String::from("),"),
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
            Token::Pow => String::from("^"),
            Token::Mod => String::from("mod"),
            Token::Sqrt => String::from("sqrt"),

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
            Token::Map => String::from("map"),
            Token::In => String::from("in"),
            Token::If => String::from("if"),
            Token::Then => String::from("then"),
            Token::Match => String::from("match"),
            Token::With => String::from("with"),
            Token::Else => String::from("else"),
            Token::Continue => String::from("continue"),
            Token::Break => String::from("break"),
            Token::Ret => String::from("return"),
            Token::Do => String::from("do"),

            Token::Quest => String::from("?"),
            Token::QuestOr => String::from("?or"),
            Token::QuestCall => String::from("?."),

            Token::Handle => String::from("handle"),
            Token::Raises => String::from("raises"),
            Token::Retry => String::from("retry"),
            Token::When => String::from("when"),

            Token::Pass => String::from("pass"),
            Token::Print => String::from("print"),
            Token::Undefined => String::from("undefined"),
            Token::Comment(string) => format!("{} (comment)", string)
        })
    }
}

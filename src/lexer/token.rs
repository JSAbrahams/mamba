use std::fmt;

#[derive(PartialEq, Debug, Clone)]
pub struct TokenPos {
    pub line: i32,
    pub pos: i32,
    pub token: Token,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Token {
    From,
    Type,
    Class,
    Util,
    IsA,
    Constructor,
    Private,

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
    OfMut,
    Assign,
    Def,

    Real(String),
    Int(String),
    ENum(String, String),
    Str(String),
    Bool(bool),
    Range,
    InRange,

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

    NL,
    Indent,
    Dedent,

    Raises,

    While,
    For,
    Where,
    Map,
    In,
    If,
    When,
    Then,
    Else,
    Do,
    Continue,
    Break,
    Ret,

    Quest,
    QuestOr,
    Handle,

    Print,
    PrintLn,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string_representation = match self.clone() {
            Token::From => "from".to_string(),
            Token::Util => "util".to_string(),
            Token::Type => "type".to_string(),
            Token::Class => "class".to_string(),
            Token::IsA => "isa".to_string(),
            Token::Constructor => "constructor".to_string(),
            Token::Private => "private".to_string(),

            Token::As => "as".to_string(),
            Token::Use => "use".to_string(),
            Token::UseAll => "useall".to_string(),
            Token::Forward => "forward".to_string(),
            Token::_Self => "self".to_string(),

            Token::Fun => "fun".to_string(),
            Token::Point => ".".to_string(),
            Token::Comma => ",".to_string(),
            Token::DoublePoint => ":".to_string(),
            Token::DDoublePoint => "::".to_string(),

            Token::Id(id) => format!("<identifier>: {}", id),
            Token::Mut => "mut".to_string(),
            Token::OfMut => "ofmut".to_string(),
            Token::Assign => "<-".to_string(),
            Token::Def => "def".to_string(),

            Token::Real(real) => format!("<real>: {}", real),
            Token::Int(int) => format!("<int>: {}", int),
            Token::ENum(int, exp) => format!("<e-number>: {}e{}", int, exp),
            Token::Str(string) => format!("<string>: \"{}\"", string),
            Token::Bool(boolean) => format!("<bool>: {}", boolean),
            Token::Range => "..".to_string(),
            Token::InRange => "inrange".to_string(),

            Token::Add => "+".to_string(),
            Token::Sub => "-".to_string(),
            Token::Mul => "*".to_string(),
            Token::Div => "/".to_string(),
            Token::Pow => "^".to_string(),
            Token::Mod => "mod".to_string(),
            Token::Sqrt => "sqrt".to_string(),

            Token::Ge => ">".to_string(),
            Token::Geq => ">=".to_string(),
            Token::Le => "<".to_string(),
            Token::Leq => "<=".to_string(),

            Token::Eq => "eq".to_string(),
            Token::Is => "is".to_string(),
            Token::IsN => "isnt".to_string(),
            Token::Neq => "neq".to_string(),
            Token::And => "and".to_string(),
            Token::Or => "or".to_string(),
            Token::Not => "not".to_string(),

            Token::LRBrack => "(".to_string(),
            Token::RRBrack => ")".to_string(),
            Token::LSBrack => "[".to_string(),
            Token::RSBrack => "]".to_string(),
            Token::LCBrack => "{".to_string(),
            Token::RCBrack => "}".to_string(),
            Token::Ver => "|".to_string(),
            Token::To => "->".to_string(),

            Token::NL => "<newline>".to_string(),
            Token::Indent => "<indent>".to_string(),
            Token::Dedent => "<dedent>".to_string(),

            Token::Raises => "raises".to_string(),

            Token::While => "while".to_string(),
            Token::For => "foreach".to_string(),
            Token::Where => "where".to_string(),
            Token::Map => "map".to_string(),
            Token::In => "in".to_string(),
            Token::If => "if".to_string(),
            Token::When => "when".to_string(),
            Token::Then => "then".to_string(),
            Token::Else => "else".to_string(),
            Token::Do => "do".to_string(),
            Token::Continue => "continue".to_string(),
            Token::Break => "break".to_string(),
            Token::Ret => "return".to_string(),

            Token::Quest => "?".to_string(),
            Token::QuestOr => "?or".to_string(),
            Token::Handle => "handle".to_string(),

            Token::Print => "print".to_string(),
            Token::PrintLn => "println".to_string()
        };

        write!(f, "{}", string_representation)
    }
}

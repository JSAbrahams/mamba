pub enum Core {
    ImportModUse(Box<Core>, Box<Core>),
    ImportModUseAs(Box<Core>, Box<Core>, Box<Core>),

    FunDef(Box<Core>, Vec<Core>, Box<Core>, Box<Core>),
    FunCall(Box<Core>, String, Box<Core>),
    FunArg(String, Box<Core>),
    FunType(Box<Core>, Box<Core>),
    FunTuple(Vec<Core>),

    Module(Box<Core>),
    Script(Vec<Core>, Vec<Core>, Box<Core>),
    Class(Box<Core>, Vec<Core>, Vec<Core>),
    Util(Box<Core>, Vec<Core>, Vec<Core>),

    Id(String),
    Let(String, Box<Core>),
    Assign(Box<Core>, Box<Core>),
    SetBuilder(Box<Core>, Vec<Core>),

    Block(Vec<Core>),

    Real(Vec<f64>),
    Int(Vec<i64>),
    ENum(Vec<i64>, Vec<i64>),
    Str(String),
    Bool(bool),
    Tuple(Vec<Core>),

    Add(Box<Core>, Box<Core>),
    AddU(Box<Core>),
    Sub(Box<Core>, Box<Core>),
    SubU(Box<Core>),
    Mul(Box<Core>, Box<Core>),
    Div(Box<Core>, Box<Core>),
    Mod(Box<Core>, Box<Core>),
    Pow(Box<Core>, Box<Core>),

    Le(Box<Core>, Box<Core>),
    Ge(Box<Core>, Box<Core>),
    Leq(Box<Core>, Box<Core>),
    Geq(Box<Core>, Box<Core>),

    Is(Box<Core>, Box<Core>),
    Eq(Box<Core>, Box<Core>),
    Not(Box<Core>),
    And(Box<Core>, Box<Core>),
    Or(Box<Core>, Box<Core>),

    IfElse(Box<Core>, Box<Core>, Box<Core>),
    When(Box<Core>, Vec<Core>),
    For(Box<Core>, Box<Core>, Box<Core>),
    While(Box<Core>, Box<Core>),
    Break,
    Continue,

    Return(Box<Core>),
    Print(Box<Core>),

    Empty,
    All
}

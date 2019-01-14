pub enum Core {
    ImportModUse(Box<Core>, Box<Core>),
    ImportModUseAs(Box<Core>, Box<Core>, Box<Core>),

    FunDef(Box<Core>, Vec<ASTNode>, Box<Core>, Box<Core>),
    FunDefNoRetType(Box<Core>, Vec<ASTNode>, Box<Core>),
    FunAnon(Box<Core>, Box<Core>),
    FunCall(Box<Core>, Box<Core>, Box<Core>),
    FunCallDirect(Box<Core>, Box<Core>),
    FunArg(Box<Core>, Box<Core>),
    FunType(Box<Core>, Box<Core>),
    FunTuple(Vec<ASTNode>),

    Module(Box<Core>),
    Script(Vec<ASTNode>, Vec<ASTNode>, Box<Core>),
    Class(Box<Core>, Vec<ASTNode>, Vec<ASTNode>),
    Util(Box<Core>, Vec<ASTNode>, Vec<ASTNode>),

    Id(Box<Core>, String),
    Assign(Box<Core>, Box<Core>),
    Defer(Box<Core>, Vec<ASTNode>),
    LetType(Box<Core>, Box<Core>, Box<Core>),
    SetBuilder(Box<Core>, Vec<ASTNode>),

    Block(Vec<ASTNode>),

    Real(String),
    Int(String),
    ENum(String, String),
    Str(String),
    Bool(bool),
    Tuple(Vec<ASTNode>),

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
    When(Box<Core>, Vec<ASTNode>),
    For(Box<Core>, Box<Core>, Box<Core>),
    While(Box<Core>, Box<Core>),
    Break,
    Continue,

    Return(Box<Core>),
    Print(Box<Core>),

    Empty,
    All
}

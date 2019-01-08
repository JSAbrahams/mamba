use crate::lexer::Token;

#[macro_use]
macro_rules! next_and { ($it:expr, $stmt:stmt) => {{ $it.next(); $stmt }} }
macro_rules! wrap { ($node:expr) => {{ Box::new($node) }} }

mod expression_or_statement;
mod module;
mod expression;
mod statement;
mod util;

#[derive(PartialEq)]
#[derive(Debug)]
pub enum ASTNode {
    ImportModUse(Box<ASTNode>, Box<ASTNode>),
    ImportModUseAs(Box<ASTNode>, Box<ASTNode>, Box<ASTNode>),
    ImportModUseAll(Box<ASTNode>),

    FunDef(Box<ASTNode>, Vec<ASTNode>, Box<ASTNode>, Box<ASTNode>),
    FunDefNoRetType(Box<ASTNode>, Vec<ASTNode>, Box<ASTNode>),
    FunCall(Box<ASTNode>, Box<ASTNode>, Box<ASTNode>),
    FunCallDirect(Box<ASTNode>, Box<ASTNode>),
    FunArg(Box<ASTNode>, Box<ASTNode>),
    FunType(Box<ASTNode>, Box<ASTNode>),
    FunTuple(Vec<ASTNode>),

    ModProgram(Vec<ASTNode>, Vec<ASTNode>, Box<ASTNode>),
    ModClass(Vec<ASTNode>, Box<ASTNode>, Vec<ASTNode>),

    Id(String),
    Assign(Box<ASTNode>, Box<ASTNode>),
    Mut(Box<ASTNode>),

    Do(Vec<ASTNode>),

    Real(String),
    Int(String),
    ENum(String, String),
    Str(String),
    Bool(bool),
    Tuple(Vec<ASTNode>),

    Add(Box<ASTNode>, Box<ASTNode>),
    AddU(Box<ASTNode>),
    Sub(Box<ASTNode>, Box<ASTNode>),
    SubU(Box<ASTNode>),
    Mul(Box<ASTNode>, Box<ASTNode>),
    Div(Box<ASTNode>, Box<ASTNode>),
    Mod(Box<ASTNode>, Box<ASTNode>),
    Pow(Box<ASTNode>, Box<ASTNode>),

    Le(Box<ASTNode>, Box<ASTNode>),
    Ge(Box<ASTNode>, Box<ASTNode>),
    Leq(Box<ASTNode>, Box<ASTNode>),
    Geq(Box<ASTNode>, Box<ASTNode>),

    Is(Box<ASTNode>, Box<ASTNode>),
    IsN(Box<ASTNode>, Box<ASTNode>),
    Eq(Box<ASTNode>, Box<ASTNode>),
    Neq(Box<ASTNode>, Box<ASTNode>),
    Not(Box<ASTNode>),
    And(Box<ASTNode>, Box<ASTNode>),
    Or(Box<ASTNode>, Box<ASTNode>),

    If(Box<ASTNode>, Box<ASTNode>),
    IfElse(Box<ASTNode>, Box<ASTNode>, Box<ASTNode>),
    Unless(Box<ASTNode>, Box<ASTNode>),
    UnlessElse(Box<ASTNode>, Box<ASTNode>, Box<ASTNode>),
    When(Box<ASTNode>, Vec<ASTNode>),
    For(Box<ASTNode>, Box<ASTNode>, Box<ASTNode>),
    While(Box<ASTNode>, Box<ASTNode>),
    Loop(Box<ASTNode>),
    Break,
    Continue,

    Return(Box<ASTNode>),
    Print(Box<ASTNode>),
    DoNothing,
}

// module ::= class | program
pub fn parse(input: Vec<Token>) -> Result<ASTNode, String> {
    return module::parse(&mut input.iter().peekable());
}

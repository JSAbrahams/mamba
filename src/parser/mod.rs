use crate::lexer::TokenPos;
use crate::parser::parse_result::ParseResult;

#[macro_use]
macro_rules! next_and { ($it:expr, $stmt:stmt) => {{ $it.next(); $stmt }} }
macro_rules! get_or_err { ($res:expr, $msg:expr) => {
    match $res {
        (Ok(node), ind) => (Box::new(node), ind),
        (Err(err), _) => return Err(ParseErr { parsing: $msg.to_string(), cause: Box::new(err) })
    }
}}
macro_rules! check_next_is { ($it: expr, $ind:expr, $tok:path) => {
    if let Some(next) = $it.next() {
        if next.token != $tok { return Err(TokenErr { expected: $tok, actual: next.clone() }); }
    } else { return Err(EOFErr { expected: $tok }); }
}}

mod parse_result;

mod assignment;
mod control_flow_stmt;
mod control_flow_expr;
mod do_block;
mod expr_or_stmt;
mod function;
mod maybe_expr;
mod module;
mod operation;
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
    FunAnon(Box<ASTNode>, Box<ASTNode>),
    FunCall(Box<ASTNode>, Box<ASTNode>, Box<ASTNode>),
    FunCallDirect(Box<ASTNode>, Box<ASTNode>),
    FunArg(Box<ASTNode>, Box<ASTNode>),
    FunType(Box<ASTNode>, Box<ASTNode>),
    FunTuple(Vec<ASTNode>),

    ModScript(Vec<ASTNode>, Vec<ASTNode>, Box<ASTNode>),
    ModClass(Box<ASTNode>, Vec<ASTNode>, Vec<ASTNode>),
    ModUtil(Box<ASTNode>, Vec<ASTNode>, Vec<ASTNode>),

    Id(String),
    Self_(Box<ASTNode>),
    Assign(Box<ASTNode>, Box<ASTNode>),
    Defer(Box<ASTNode>, Vec<ASTNode>),
    Mut(Box<ASTNode>),
    Let(Box<ASTNode>),
    LetType(Box<ASTNode>, Box<ASTNode>),

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
    From(Box<ASTNode>, Box<ASTNode>),
    FromMap(Box<ASTNode>, Box<ASTNode>, Box<ASTNode>),
    For(Box<ASTNode>, Box<ASTNode>, Box<ASTNode>),
    While(Box<ASTNode>, Box<ASTNode>),
    Loop(Box<ASTNode>),
    Break,
    Continue,

    Return(Box<ASTNode>),
    ReturnEmpty,
    Print(Box<ASTNode>),
}

// module ::= interface | util | class | script

pub fn parse(input: Vec<TokenPos>) -> ParseResult<ASTNode> {
    return module::parse_module(&mut input.iter().peekable());
}

use crate::lexer::TokenPos;
use crate::parser::parse_result::ParseResult;

#[macro_use]
/// Call next on the iterator and execute the statement.
/// This ignores the value of the next value of the iterator.
macro_rules! next_and { ($it:expr, $stmt:stmt) => {{ $it.next(); $stmt }} }

macro_rules! print_parse { ($it:expr, $ind:expr, $msg:expr) => {{
    if env::var("PRINT_PARSE").is_ok() {
        let mut ind_string = String::new();
        for _ in 0..$ind { ind_string.push('-') };

        match $it.peek() {
            Some(tp) => println!("{}{:?} ({})", ind_string, tp, $msg),
            None => println!("{}({})", ind_string, $msg)
        }
    }
}}}

/// Evaluates the result.
/// 
/// If it is an Ok tuple, return Boxed [`ASTNode`] and indent in tuple.
/// If it is error, return [`ParseErr`] with the [`Err`] wrapped.
macro_rules! get_or_err { ($it:expr, $ind:expr, $fun:path, $msg:expr) => {{
    let current = $it.peek().cloned();
    match $fun($it, $ind) {
        Ok((node, ind)) => (Box::new(node), ind),
        Err(err) => return match current {
            Some(tp) => Err(ParseErr { parsing: $msg.to_string(), cause: Box::new(err),
                                       position: Some(tp.clone()) }),
            None =>
                Err(ParseErr { parsing: $msg.to_string(), cause: Box::new(err), position: None })
        }
    }
}}}

/// Evaluates the expression and check result.
///
/// If it is an [`Ok`] tuple, return [`ASTNode`] and indent tuple.
/// If it is error, return [`ParseErr`] with [`Err`] wrapped.
macro_rules! get_or_err_direct { ($it:expr, $ind:expr, $fun:path, $msg:expr) => {{
    let current = $it.peek().cloned();
    match $fun($it, $ind) {
        Ok((node, ind)) => (node, ind),
        Err(e) => return match current {
            Some(tp) => Err(ParseErr { parsing: $msg.to_string(), cause: Box::new(e),
                                       position: Some(tp.clone()) }),
            None => Err(ParseErr { parsing: $msg.to_string(), cause: Box::new(e), position: None })
        }
}}}}

/// Check that the next is of expected token type.
///
/// If it is not of the expected token type, returns [`TokenErr`].
/// If there is no token ([`iterator::next()`] returns [`None`]), returns [`EOFErr`].
macro_rules! check_next_is { ($it: expr, $tok:path) => {
    if let Some(next) = $it.next() {
        if next.token != $tok { return Err(TokenErr { expected: $tok, actual: next.clone() }); }
    } else { return Err(EOFErr { expected: $tok }); }
}}

mod parse_result;

mod control_flow_stmt;
mod control_flow_expr;
mod declaration;
mod block;
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

    Module(Box<ASTNode>),
    Script(Vec<ASTNode>, Vec<ASTNode>, Box<ASTNode>),
    Class(Box<ASTNode>, Vec<ASTNode>, Vec<ASTNode>),
    Util(Box<ASTNode>, Vec<ASTNode>, Vec<ASTNode>),

    Id(String),
    Self_(Box<ASTNode>),
    Assign(Box<ASTNode>, Box<ASTNode>),
    Defer(Box<ASTNode>, Vec<ASTNode>),
    Mut(Box<ASTNode>),
    Let(Box<ASTNode>),
    LetType(Box<ASTNode>, Box<ASTNode>),
    SetBuilder(Box<ASTNode>, Vec<ASTNode>),

    Block(Vec<ASTNode>),

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
    Sqrt(Box<ASTNode>),

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

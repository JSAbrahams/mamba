use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::parse_result::ParseResult;
use std::iter::Peekable;
use std::slice::Iter;

#[macro_use]
/// Evaluates the result.
/// 
/// If it is an Ok tuple, return Boxed [`ASTNodePos`].
/// If it is error, return [`ParseErr`] with the [`Err`] wrapped.
macro_rules! get_or_err { ($it:expr, $fun:path, $msg:expr) => {{
    let current = $it.peek().cloned();
    match $fun($it) {
        Ok(node) => Box::new(node),
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
/// If it is an [`Ok`] tuple, return [`ASTNodePos`]..
/// If it is error, return [`ParseErr`] with [`Err`] wrapped.
macro_rules! get_or_err_direct { ($it:expr, $fun:path, $msg:expr) => {{
    let current = $it.peek().cloned();
    match $fun($it) {
        Ok(node) => node,
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
        next.clone()
    } else { return Err(EOFErr { expected: $tok }); }
}}

type TPIterator<'a> = Peekable<Iter<'a, TokenPos>>;

/// Gets the starting line and position of the current token using [`TPIterator.peek()`].
fn start_pos(it: &mut TPIterator) -> (i32, i32) {
    match it.peek() {
        Some(TokenPos { line, pos, token: _ }) => (*line, *pos),
        None => (0, 0)
    }
}

/// Gets the end line and position of the current token using [`TPIterator.peek()`].
///
/// The end position is calculated by offsetting the starting position by the offset of the current
/// token, by calling its [`fmt::Display`] method.
fn end_pos(it: &mut TPIterator) -> (i32, i32) {
    match it.peek() {
        Some(TokenPos { line, pos, token }) => {
            let tok_width = match token {
                Token::Id(id) => id.len(),
                Token::Real(real) => real.len(),
                Token::Int(int) => int.len(),
                Token::Bool(true) => 4,
                Token::Bool(false) => 5,
                Token::Str(_str) => _str.len(),
                Token::ENum(num, exp) => num.len() + 1 + exp.len(),
                other => format!("{}", other).len() - 2
            } as i32;

            (*line, *pos + tok_width)
        }
        None => (-1, -1)
    }
}

mod parse_result;

mod control_flow_stmt;
mod control_flow_expr;
mod definition;
mod block;
mod collection;
mod expr_or_stmt;
mod function;
mod expression;
mod module;
mod operation;
mod statement;
mod _type;


#[derive(PartialEq, Eq, Hash)]
#[derive(Debug)]
/// Wrapper of ASTNode, and its start end end position in the source code.
/// The start and end positions can be used to generate useful error messages.
pub struct ASTNodePos {
    pub st_line: i32,
    pub st_pos: i32,
    pub en_line: i32,
    pub en_pos: i32,
    pub node: ASTNode,
}

#[derive(PartialEq, Eq, Hash)]
#[derive(Debug)]
pub enum ASTNode {
    ImportModUse { _mod: Box<ASTNodePos>, _use: Box<ASTNodePos> },
    ImportModUseAs { _mod: Box<ASTNodePos>, _use: Box<ASTNodePos>, _as: Box<ASTNodePos> },
    ImportModUseAll { _mod: Box<ASTNodePos> },

    Script {
        imports: Vec<ASTNodePos>,
        decl: Box<ASTNodePos>,
        funcs: Vec<ASTNodePos>,
        body: Box<ASTNodePos>,
    },
    Class {
        imports: Vec<ASTNodePos>,
        name: Box<ASTNodePos>,
        decls: Box<ASTNodePos>,
        funcs: Vec<ASTNodePos>,
    },
    Util {
        imports: Vec<ASTNodePos>,
        name: Box<ASTNodePos>,
        decls: Box<ASTNodePos>,
        funcs: Vec<ASTNodePos>,
    },

    ModName { name: String },
    ModNameIsA { name: String, isa: Vec<String> },

    EmptyDef { _mut: bool, of_mut: bool, id_maybe_type: Box<ASTNodePos> },
    Def { _mut: bool, of_mut: bool, id_maybe_type: Box<ASTNodePos>, expr: Box<ASTNodePos> },

    Id { lit: String },
    TypeId { id: String },
    TypeFun { left: Box<ASTNodePos>, right: Box<ASTNodePos> },
    TypeTup { types: Vec<ASTNodePos> },
    TypeDef { id: Box<ASTNodePos>, _type: Box<ASTNodePos> },
    IdAndType { id: Box<ASTNodePos>, _type: Box<ASTNodePos> },

    Defer { definition: Box<ASTNodePos>, forwarded: Vec<ASTNodePos> },
    _Self { expr: Box<ASTNodePos> },

    Assign { left: Box<ASTNodePos>, right: Box<ASTNodePos> },

    SetBuilder { set: Box<ASTNodePos>, conditions: Vec<ASTNodePos> },
    Set { elements: Vec<ASTNodePos> },
    List { elements: Vec<ASTNodePos> },
    Tuple { elements: Vec<ASTNodePos> },

    Block { stmts: Vec<ASTNodePos> },

    Real { lit: String },
    Int { lit: String },
    ENum { num: String, exp: String },
    Str { lit: String },
    Bool { lit: bool },

    Add { left: Box<ASTNodePos>, right: Box<ASTNodePos> },
    AddU { expr: Box<ASTNodePos> },
    Sub { left: Box<ASTNodePos>, right: Box<ASTNodePos> },
    SubU { expr: Box<ASTNodePos> },
    Mul { left: Box<ASTNodePos>, right: Box<ASTNodePos> },
    Div { left: Box<ASTNodePos>, right: Box<ASTNodePos> },
    Mod { left: Box<ASTNodePos>, right: Box<ASTNodePos> },
    Pow { left: Box<ASTNodePos>, right: Box<ASTNodePos> },
    Sqrt { expr: Box<ASTNodePos> },

    Le { left: Box<ASTNodePos>, right: Box<ASTNodePos> },
    Ge { left: Box<ASTNodePos>, right: Box<ASTNodePos> },
    Leq { left: Box<ASTNodePos>, right: Box<ASTNodePos> },
    Geq { left: Box<ASTNodePos>, right: Box<ASTNodePos> },
    Is { left: Box<ASTNodePos>, right: Box<ASTNodePos> },
    IsN { left: Box<ASTNodePos>, right: Box<ASTNodePos> },
    Eq { left: Box<ASTNodePos>, right: Box<ASTNodePos> },
    Neq { left: Box<ASTNodePos>, right: Box<ASTNodePos> },
    IsA { left: Box<ASTNodePos>, right: Box<ASTNodePos> },
    Not { expr: Box<ASTNodePos> },
    And { left: Box<ASTNodePos>, right: Box<ASTNodePos> },
    Or { left: Box<ASTNodePos>, right: Box<ASTNodePos> },

    If { cond: Box<ASTNodePos>, then: Box<ASTNodePos> },
    IfElse { cond: Box<ASTNodePos>, then: Box<ASTNodePos>, _else: Box<ASTNodePos> },
    When { cond: Box<ASTNodePos>, cases: Vec<ASTNodePos> },
    For { expr: Box<ASTNodePos>, collection: Box<ASTNodePos>, body: Box<ASTNodePos> },
    While { cond: Box<ASTNodePos>, body: Box<ASTNodePos> },
    Break,
    Continue,

    Return { expr: Box<ASTNodePos> },
    ReturnEmpty,

    Print { expr: Box<ASTNodePos> },
    PrintLn { expr: Box<ASTNodePos> },
}

pub fn parse(input: Vec<TokenPos>) -> ParseResult {
    return module::parse_module(&mut input.iter().peekable());
}

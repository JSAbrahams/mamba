use crate::lexer::TokenPos;
use crate::parser::parse_result::ParseResult;
use std::iter::Peekable;
use std::slice::Iter;

#[macro_use]
/// Call next on the iterator and execute the statement.
/// This ignores the value of the next value of the iterator.
macro_rules! next_and { ($it:expr, $stmt:stmt) => {{ $it.next(); $stmt }} }

macro_rules! print_parse { ($it:expr, $msg:expr) => {{
    if env::var("PRINT_PARSE").is_ok() {
        match $it.peek() {
            Some(tp) => println!("{:?} ({})", tp, $msg),
            None => println!("({})", $msg)
        }
    }
}}}

/// Evaluates the result.
/// 
/// If it is an Ok tuple, return Boxed [`ASTNode`].
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
/// If it is an [`Ok`] tuple, return [`ASTNode`]..
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

pub struct ASTNodePos {
    pub line: i32,
    pub pos: i32,
    pub node: ASTNode,
}

#[derive(PartialEq)]
#[derive(Debug)]
pub enum ASTNode {
    ImportModUse { _mod: Box<ASTNode>, _use: Box<ASTNode> },
    ImportModUseAs { _mod: Box<ASTNode>, _use: Box<ASTNode>, _as: Box<ASTNode> },
    ImportModUseAll { _mod: Box<ASTNode> },

    Script { imports: Vec<ASTNode>, decl: Box<ASTNode>, funcs: Vec<ASTNode>, body: Box<ASTNode> },
    Class { imports: Vec<ASTNode>, name: Box<ASTNode>, decls: Box<ASTNode>, funcs: Vec<ASTNode> },
    Util { imports: Vec<ASTNode>, name: Box<ASTNode>, decls: Box<ASTNode>, funcs: Vec<ASTNode> },
    ModName { name: String },
    ModNameIsA { name: String, isa: Vec<String> },

    _Self { expr: Box<ASTNode> },
    Assign { left: Box<ASTNode>, right: Box<ASTNode> },
    Defer { declaration: Box<ASTNode>, properties: Vec<ASTNode> },
    Mut { decl: Box<ASTNode> },
    Let { id: String },
    LetType { id: String, _type: Box<ASTNode> },

    SetBuilder { set: Box<ASTNode>, conditions: Vec<ASTNode> },
    Set { elements: Vec<ASTNode> },
    List { elements: Vec<ASTNode> },
    Tuple { elements: Vec<ASTNode> },

    Block { stmts: Vec<ASTNode> },

    Id { id: String },
    Real { real: String },
    Int { int: String },
    ENum { int_digits: String, frac_digits: String },
    Str { string: String },
    Bool { _bool: bool },

    Add { left: Box<ASTNode>, right: Box<ASTNode> },
    AddU { expr: Box<ASTNode> },
    Sub { left: Box<ASTNode>, right: Box<ASTNode> },
    SubU { expr: Box<ASTNode> },
    Mul { left: Box<ASTNode>, right: Box<ASTNode> },
    Div { left: Box<ASTNode>, right: Box<ASTNode> },
    Mod { left: Box<ASTNode>, right: Box<ASTNode> },
    Pow { left: Box<ASTNode>, right: Box<ASTNode> },
    Sqrt { expr: Box<ASTNode> },

    Le { left: Box<ASTNode>, right: Box<ASTNode> },
    Ge { left: Box<ASTNode>, right: Box<ASTNode> },
    Leq { left: Box<ASTNode>, right: Box<ASTNode> },
    Geq { left: Box<ASTNode>, right: Box<ASTNode> },

    Is { left: Box<ASTNode>, right: Box<ASTNode> },
    IsN { left: Box<ASTNode>, right: Box<ASTNode> },
    Eq { left: Box<ASTNode>, right: Box<ASTNode> },
    Neq { left: Box<ASTNode>, right: Box<ASTNode> },
    IsA { left: Box<ASTNode>, right: Box<ASTNode> },
    Not { expr: Box<ASTNode> },
    And { left: Box<ASTNode>, right: Box<ASTNode> },
    Or { left: Box<ASTNode>, right: Box<ASTNode> },

    If { cond: Box<ASTNode>, then: Box<ASTNode> },
    IfElse { cond: Box<ASTNode>, then: Box<ASTNode>, _else: Box<ASTNode> },
    Unless { cond: Box<ASTNode>, then: Box<ASTNode> },
    UnlessElse { cond: Box<ASTNode>, then: Box<ASTNode>, _else: Box<ASTNode> },
    When { cond: Box<ASTNode>, cases: Vec<ASTNode> },
    For { expr: Box<ASTNode>, collection: Box<ASTNode>, body: Box<ASTNode> },
    While { cond: Box<ASTNode>, body: Box<ASTNode> },
    Break,
    Continue,

    Return { expr: Box<ASTNode> },
    ReturnEmpty,
    Print { expr: Box<ASTNode> },
}

type TPIterator<'a> = Peekable<Iter<'a, TokenPos>>;

pub fn parse(input: Vec<TokenPos>) -> ParseResult {
    return module::parse_module(&mut input.iter().peekable());
}

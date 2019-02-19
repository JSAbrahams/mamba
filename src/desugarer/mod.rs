use crate::desugarer::expression::desugar_expression;
use crate::parser::ASTNodePos;
use std::collections::HashMap;
use std::collections::HashSet;

#[macro_use]
/// Desugar and box.
macro_rules! desugar { ($ast:expr ) => {{ Box::new(desugar($ast)) }} }

mod expression;

pub enum Core {
    Import { file: String, _use: Vec<Core> },
    Module { id: String, imports: Vec<String>, functions: Vec<Core>, body: Box<Core> },
    Function { function: String, args: Vec<String>, body: Box<Core> },
    FunctionCall { namespace: String, function: String, args: Vec<Core> },
    MethodCall { object: Box<Core>, method: String, args: Vec<Core> },

    Id { lit: String },
    Assign { left: Box<Core>, right: Box<Core> },
    VarDef { id: Box<Core>, right: Box<Core> },
    FunDef { id: String, args: Vec<Core>, raises: Vec<Core>, right: Box<Core> },
    FunArg { vararg: bool, id: String },

    Block { statements: Vec<Core> },

    BigFloat { int_digits: Vec<i64>, frac_digits: Vec<i64> },
    BigInt { integers: Vec<i64> },
    Str { str: String },
    Bool { _bool: bool },

    Tuple { elements: Vec<Core> },
    Set { elements: HashSet<Core> },
    List { elements: Vec<Core> },

    Le { left: Box<Core>, right: Box<Core> },
    Leq { left: Box<Core>, right: Box<Core> },

    Not { expr: Box<Core> },
    Is { left: Box<Core>, right: Box<Core> },
    Eq { left: Box<Core>, right: Box<Core> },
    IsA { left: Box<Core>, right: Box<Core> },
    And { left: Box<Core>, right: Box<Core> },
    Or { left: Box<Core>, right: Box<Core> },

    IfElse { cond: Box<Core>, then: Box<Core>, _else: Box<Core> },
    When { cond: Box<Core>, cases: Vec<Core> },
    Case { cond: Box<Core>, then: Box<Core> },
    While { cond: Box<Core>, body: Box<Core> },
    Break,
    Continue,

    Return { expr: Box<Core> },
    Print { expr: Box<Core> },
    UnderScore,

    Undefined,
    Empty,
}

pub fn desugar(input: ASTNodePos) -> Core {
    desugar_expression(input)
}

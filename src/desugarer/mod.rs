use crate::parser::ASTNode;

#[macro_use]
/// Desugar and box.
macro_rules! des { ($ast:expr ) => {{ Box::new(desugar(*$ast)) }} }

macro_rules! des_direct { ($ast:expr ) => {{ desugar(*$ast) }} }

macro_rules! des_vec { ($ast:expr ) => {{ panic!("not implemented") }} }

// mod expression;
// mod function;
// mod module;
// mod statement;

pub enum Core {
    Module { id: String, imports: Vec<String>, functions: Vec<Core>, body: Box<Core> },
    Function { function: String, args: Vec<String>, body: Box<Core> },
    FunctionCall { namespace: String, function: String, args: Vec<Core> },
    Import { file: String, _use: Vec<Core> },

    Id { id: String },
    Let { id: String, right: Box<Core> },
    Assign { left: Box<Core>, right: Box<Core> },

    Block { stmts: Vec<Core> },

    Real { int_digits: f64 },
    BigReal { int_digits: Vec<i64>, frac_digits: Vec<i64> },
    Int { integer_digits: i64 },
    BigInt { integer_digits: Vec<i64> },
    ENum { base: f64, exp: i64 },
    Str(String),
    Bool(bool),
    Tuple(Vec<Core>),

    Add { left: Box<Core>, right: Box<Core> },
    AddU { expr: Box<Core> },
    Sub { left: Box<Core>, right: Box<Core> },
    SubU { expr: Box<Core> },
    Mul { left: Box<Core>, right: Box<Core> },
    Div { left: Box<Core>, right: Box<Core> },
    Mod { left: Box<Core>, right: Box<Core> },
    Pow { left: Box<Core>, right: Box<Core> },

    Le { left: Box<Core>, right: Box<Core> },
    Ge { left: Box<Core>, right: Box<Core> },
    Leq { left: Box<Core>, right: Box<Core> },
    Geq { left: Box<Core>, right: Box<Core> },

    Is { left: Box<Core>, right: Box<Core> },
    Eq { left: Box<Core>, right: Box<Core> },
    Not { expr: Box<Core> },
    And { left: Box<Core>, right: Box<Core> },
    Or { left: Box<Core>, right: Box<Core> },
    IsA { expr: Box<Core>, _type: Box<Core> },

    IfElse { cond: Box<Core>, then: Box<Core>, _else: Box<Core> },
    When { expr: Box<Core>, cases: Vec<Core> },
    While { cond: Box<Core>, body: Box<Core> },
    Break,
    Continue,

    Return { expr: Box<Core> },
    Print { expr: Box<Core> },

    Empty,
    All,
}

pub fn desugar(input: ASTNode) -> Core {
    panic!("not implemented")
}

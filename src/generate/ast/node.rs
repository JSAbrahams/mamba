use std::fmt::{Display, Formatter};

use crate::check::context::function;
use crate::check::context::function::python;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Core {
    FromImport { from: Box<Core>, import: Box<Core> },
    Import { imports: Vec<Core> },
    ImportAs { imports: Vec<Core>, aliases: Vec<Core> },
    ClassDef { name: Box<Core>, parent_names: Vec<Core>, body: Box<Core> },
    FunctionCall { function: Box<Core>, args: Vec<Core> },
    PropertyCall { object: Box<Core>, property: Box<Core> },
    Id { lit: String },
    Type { lit: String, generics: Vec<Core> },
    ExpressionType { expr: Box<Core>, ty: Box<Core> },
    Assign { left: Box<Core>, right: Box<Core>, op: CoreOp },
    VarDef { var: Box<Core>, ty: Option<Box<Core>>, expr: Option<Box<Core>> },
    FunDefOp { op: CoreOp, arg: Vec<Core>, ty: Option<Box<Core>>, body: Box<Core> },
    FunDef { id: Box<Core>, arg: Vec<Core>, ty: Option<Box<Core>>, body: Box<Core> },
    FunArg { vararg: bool, var: Box<Core>, ty: Option<Box<Core>>, default: Option<Box<Core>> },
    AnonFun { args: Vec<Core>, body: Box<Core> },
    Block { statements: Vec<Core> },
    Float { float: String },
    Int { int: String },
    ENum { num: String, exp: String },
    DocStr { string: String },
    Str { string: String },
    FStr { string: String },
    Bool { boolean: bool },
    Tuple { elements: Vec<Core> },
    TupleLiteral { elements: Vec<Core> },
    Set { elements: Vec<Core> },
    List { elements: Vec<Core> },
    Index { item: Box<Core>, range: Box<Core> },
    Ge { left: Box<Core>, right: Box<Core> },
    Geq { left: Box<Core>, right: Box<Core> },
    Le { left: Box<Core>, right: Box<Core> },
    Leq { left: Box<Core>, right: Box<Core> },
    Not { expr: Box<Core> },
    Is { left: Box<Core>, right: Box<Core> },
    IsN { left: Box<Core>, right: Box<Core> },
    Eq { left: Box<Core>, right: Box<Core> },
    Neq { left: Box<Core>, right: Box<Core> },
    IsA { left: Box<Core>, right: Box<Core> },
    And { left: Box<Core>, right: Box<Core> },
    Or { left: Box<Core>, right: Box<Core> },
    Add { left: Box<Core>, right: Box<Core> },
    AddU { expr: Box<Core> },
    Sub { left: Box<Core>, right: Box<Core> },
    SubU { expr: Box<Core> },
    Mul { left: Box<Core>, right: Box<Core> },
    Mod { left: Box<Core>, right: Box<Core> },
    Pow { left: Box<Core>, right: Box<Core> },
    Div { left: Box<Core>, right: Box<Core> },
    FDiv { left: Box<Core>, right: Box<Core> },
    Sqrt { expr: Box<Core> },
    BAnd { left: Box<Core>, right: Box<Core> },
    BOr { left: Box<Core>, right: Box<Core> },
    BXOr { left: Box<Core>, right: Box<Core> },
    BOneCmpl { expr: Box<Core> },
    BLShift { left: Box<Core>, right: Box<Core> },
    BRShift { left: Box<Core>, right: Box<Core> },
    For { expr: Box<Core>, col: Box<Core>, body: Box<Core> },
    If { cond: Box<Core>, then: Box<Core> },
    IfElse { cond: Box<Core>, then: Box<Core>, el: Box<Core> },
    Ternary { cond: Box<Core>, then: Box<Core>, el: Box<Core> },
    KeyValue { key: Box<Core>, value: Box<Core> },
    While { cond: Box<Core>, body: Box<Core> },
    In { left: Box<Core>, right: Box<Core> },
    Break,
    Continue,
    Return { expr: Box<Core> },
    UnderScore,
    Pass,
    None,
    Empty,
    Comment { comment: String },
    TryExcept { setup: Option<Box<Core>>, attempt: Box<Core>, except: Vec<Core> },
    Except { id: Box<Core>, class: Option<Box<Core>>, body: Box<Core> },
    Raise { error: Box<Core> },
    With { resource: Box<Core>, expr: Box<Core> },
    WithAs { resource: Box<Core>, alias: Box<Core>, expr: Box<Core> },
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum CoreOp {
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    PowAssign,
    BLShiftAssign,
    BRShiftAssign,

    Ge,
    Geq,
    Le,
    Leq,
    Eq,
    Neq,
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Mod,
    FDiv,
}

impl CoreOp {
    pub fn from(lit: &str) -> Option<CoreOp> {
        Some(match lit {
            function::GE => CoreOp::Ge,
            function::GEQ => CoreOp::Geq,
            function::LE => CoreOp::Le,
            function::LEQ => CoreOp::Leq,

            function::EQ => CoreOp::Eq,
            function::NEQ => CoreOp::Neq,

            function::ADD => CoreOp::Add,
            function::SUB => CoreOp::Sub,
            function::POW => CoreOp::Pow,
            function::MUL => CoreOp::Mul,
            function::MOD => CoreOp::Mod,
            function::DIV => CoreOp::Div,
            function::FDIV => CoreOp::FDiv,
            _ => return None
        })
    }
}

impl Display for CoreOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                CoreOp::Assign => "=",
                CoreOp::AddAssign => "+=",
                CoreOp::SubAssign => "-=",
                CoreOp::MulAssign => "*=",
                CoreOp::DivAssign => "/=",
                CoreOp::PowAssign => "**=",
                CoreOp::BLShiftAssign => "<<=",
                CoreOp::BRShiftAssign => ">>=",

                CoreOp::Ge => python::GE,
                CoreOp::Geq => python::GEQ,
                CoreOp::Le => python::LE,
                CoreOp::Leq => python::LEQ,

                CoreOp::Eq => python::EQ,
                CoreOp::Neq => python::NEQ,

                CoreOp::Add => python::ADD,
                CoreOp::Sub => python::SUB,
                CoreOp::Pow => python::POW,
                CoreOp::Mul => python::MUL,
                CoreOp::Mod => python::MOD,
                CoreOp::Div => python::DIV,
                CoreOp::FDiv => python::FDIV,
            }
        )
    }
}

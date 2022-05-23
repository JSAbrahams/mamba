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
    FunDefOp { op: CoreFunOp, arg: Vec<Core>, ty: Option<Box<Core>>, body: Box<Core> },
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
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum CoreFunOp {
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

impl CoreFunOp {
    pub fn from(lit: &str) -> Option<CoreFunOp> {
        Some(match lit {
            function::GE => CoreFunOp::Ge,
            function::GEQ => CoreFunOp::Geq,
            function::LE => CoreFunOp::Le,
            function::LEQ => CoreFunOp::Leq,
            function::EQ => CoreFunOp::Eq,
            function::NEQ => CoreFunOp::Neq,
            function::ADD => CoreFunOp::Add,
            function::SUB => CoreFunOp::Sub,
            function::POW => CoreFunOp::Pow,
            function::MUL => CoreFunOp::Mul,
            function::MOD => CoreFunOp::Mod,
            function::DIV => CoreFunOp::Div,
            function::FDIV => CoreFunOp::FDiv,
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
            }
        )
    }
}

impl Display for CoreFunOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                CoreFunOp::Ge => python::GE,
                CoreFunOp::Geq => python::GEQ,
                CoreFunOp::Le => python::LE,
                CoreFunOp::Leq => python::LEQ,
                CoreFunOp::Eq => python::EQ,
                CoreFunOp::Neq => python::NEQ,
                CoreFunOp::Add => python::ADD,
                CoreFunOp::Sub => python::SUB,
                CoreFunOp::Pow => python::POW,
                CoreFunOp::Mul => python::MUL,
                CoreFunOp::Mod => python::MOD,
                CoreFunOp::Div => python::DIV,
                CoreFunOp::FDiv => python::FDIV,
            }
        )
    }
}

use std::convert::TryFrom;
use std::fmt::{Display, Formatter};

use crate::ASTTy;
use crate::check::context::function;
use crate::generate::result::UnimplementedErr;
use crate::parse::ast::node_op::NodeOp;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Core {
    Import { from: Option<Box<Core>>, import: Vec<Core>, alias: Vec<Core> },
    ClassDef { name: Box<Core>, parent_names: Vec<Core>, body: Box<Core> },
    FunctionCall { function: Box<Core>, args: Vec<Core> },
    PropertyCall { object: Box<Core>, property: Box<Core> },
    Id { lit: String },
    Type { lit: String, generics: Vec<Core> },
    ExpressionType { expr: Box<Core>, ty: Box<Core> },
    Assign { left: Box<Core>, right: Box<Core>, op: CoreOp },
    VarDef { var: Box<Core>, ty: Option<Box<Core>>, expr: Option<Box<Core>> },
    FunDefOp { op: CoreFunOp, arg: Vec<Core>, ty: Option<Box<Core>>, body: Box<Core> },
    FunDef { id: String, arg: Vec<Core>, ty: Option<Box<Core>>, body: Box<Core> },
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
    Match { expr: Box<Core>, cases: Vec<Core> },
    Case { expr: Box<Core>, body: Box<Core> },
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

impl TryFrom<(&ASTTy, &NodeOp)> for CoreOp {
    type Error = UnimplementedErr;

    fn try_from((ast, op): (&ASTTy, &NodeOp)) -> Result<Self, Self::Error> {
        match &op {
            NodeOp::Add => Ok(CoreOp::AddAssign),
            NodeOp::Sub => Ok(CoreOp::SubAssign),
            NodeOp::Mul => Ok(CoreOp::MulAssign),
            NodeOp::Div => Ok(CoreOp::DivAssign),
            NodeOp::Pow => Ok(CoreOp::PowAssign),
            NodeOp::BLShift => Ok(CoreOp::BLShiftAssign),
            NodeOp::BRShift => Ok(CoreOp::BRShiftAssign),
            NodeOp::Assign => Ok(CoreOp::Assign),
            op => Err(UnimplementedErr::new(ast, &format!("Reassign with {}", op)))
        }
    }
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
                CoreFunOp::Ge => function::python::GE,
                CoreFunOp::Geq => function::python::GEQ,
                CoreFunOp::Le => function::python::LE,
                CoreFunOp::Leq => function::python::LEQ,
                CoreFunOp::Eq => function::python::EQ,
                CoreFunOp::Neq => function::python::NEQ,
                CoreFunOp::Add => function::python::ADD,
                CoreFunOp::Sub => function::python::SUB,
                CoreFunOp::Pow => function::python::POW,
                CoreFunOp::Mul => function::python::MUL,
                CoreFunOp::Mod => function::python::MOD,
                CoreFunOp::Div => function::python::DIV,
                CoreFunOp::FDiv => function::python::FDIV,
            }
        )
    }
}

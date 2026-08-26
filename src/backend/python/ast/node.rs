use std::convert::TryFrom;
use std::fmt::{Display, Formatter};

use crate::backend::python::result::UnimplementedErr;
use crate::check::context::function;
use crate::parse::ast::node_op::NodeOp;
use crate::ASTTy;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum PythonCore {
    Import {
        from: Option<Box<PythonCore>>,
        import: Vec<PythonCore>,
        alias: Vec<PythonCore>,
    },
    ClassDef {
        name: Box<PythonCore>,
        parent_names: Vec<PythonCore>,
        body: Box<PythonCore>,
    },
    FunctionCall {
        function: Box<PythonCore>,
        args: Vec<PythonCore>,
    },
    PropertyCall {
        object: Box<PythonCore>,
        property: Box<PythonCore>,
    },
    Id {
        lit: String,
    },
    Type {
        lit: String,
        generics: Vec<PythonCore>,
    },
    ExpressionType {
        expr: Box<PythonCore>,
        ty: Box<PythonCore>,
    },
    Assign {
        left: Box<PythonCore>,
        right: Box<PythonCore>,
        op: CoreOp,
    },
    VarDef {
        var: Box<PythonCore>,
        ty: Option<Box<PythonCore>>,
        expr: Option<Box<PythonCore>>,
    },
    FunDefOp {
        op: CoreFunOp,
        arg: Vec<PythonCore>,
        ty: Option<Box<PythonCore>>,
        body: Box<PythonCore>,
    },
    FunDef {
        dec: Vec<String>,
        id: String,
        arg: Vec<PythonCore>,
        ty: Option<Box<PythonCore>>,
        body: Box<PythonCore>,
    },
    FunArg {
        vararg: bool,
        var: Box<PythonCore>,
        ty: Option<Box<PythonCore>>,
        default: Option<Box<PythonCore>>,
    },
    AnonFun {
        args: Vec<PythonCore>,
        body: Box<PythonCore>,
    },
    Block {
        statements: Vec<PythonCore>,
    },
    Float {
        float: String,
    },
    Int {
        int: String,
    },
    ENum {
        num: String,
        exp: String,
    },
    DocStr {
        string: String,
    },
    Str {
        string: String,
    },
    FStr {
        string: String,
    },
    Bool {
        boolean: bool,
    },
    Tuple {
        elements: Vec<PythonCore>,
    },
    TupleLiteral {
        elements: Vec<PythonCore>,
    },
    DictComprehension {
        from: Box<PythonCore>,
        to: Box<PythonCore>,
        col: Box<PythonCore>,
        conds: Vec<PythonCore>,
    },
    Comprehension {
        expr: Box<PythonCore>,
        col: Box<PythonCore>,
        conds: Vec<PythonCore>,
    },
    Dictionary {
        elements: Vec<(PythonCore, PythonCore)>,
    },
    Set {
        elements: Vec<PythonCore>,
    },
    List {
        elements: Vec<PythonCore>,
    },
    Index {
        item: Box<PythonCore>,
        range: Box<PythonCore>,
    },
    Ge {
        left: Box<PythonCore>,
        right: Box<PythonCore>,
    },
    Geq {
        left: Box<PythonCore>,
        right: Box<PythonCore>,
    },
    Le {
        left: Box<PythonCore>,
        right: Box<PythonCore>,
    },
    Leq {
        left: Box<PythonCore>,
        right: Box<PythonCore>,
    },
    Not {
        expr: Box<PythonCore>,
    },
    Eq {
        left: Box<PythonCore>,
        right: Box<PythonCore>,
    },
    Neq {
        left: Box<PythonCore>,
        right: Box<PythonCore>,
    },
    And {
        left: Box<PythonCore>,
        right: Box<PythonCore>,
    },
    Or {
        left: Box<PythonCore>,
        right: Box<PythonCore>,
    },
    Add {
        left: Box<PythonCore>,
        right: Box<PythonCore>,
    },
    AddU {
        expr: Box<PythonCore>,
    },
    Sub {
        left: Box<PythonCore>,
        right: Box<PythonCore>,
    },
    SubU {
        expr: Box<PythonCore>,
    },
    Mul {
        left: Box<PythonCore>,
        right: Box<PythonCore>,
    },
    Mod {
        left: Box<PythonCore>,
        right: Box<PythonCore>,
    },
    Pow {
        left: Box<PythonCore>,
        right: Box<PythonCore>,
    },
    Div {
        left: Box<PythonCore>,
        right: Box<PythonCore>,
    },
    FDiv {
        left: Box<PythonCore>,
        right: Box<PythonCore>,
    },
    Sqrt {
        expr: Box<PythonCore>,
    },
    For {
        expr: Box<PythonCore>,
        col: Box<PythonCore>,
        body: Box<PythonCore>,
    },
    If {
        cond: Box<PythonCore>,
        then: Box<PythonCore>,
    },
    IfElse {
        cond: Box<PythonCore>,
        then: Box<PythonCore>,
        el: Box<PythonCore>,
    },
    Match {
        expr: Box<PythonCore>,
        cases: Vec<PythonCore>,
    },
    Case {
        expr: Box<PythonCore>,
        body: Box<PythonCore>,
    },
    Ternary {
        cond: Box<PythonCore>,
        then: Box<PythonCore>,
        el: Box<PythonCore>,
    },
    KeyValue {
        key: Box<PythonCore>,
        value: Box<PythonCore>,
    },
    While {
        cond: Box<PythonCore>,
        body: Box<PythonCore>,
    },
    In {
        left: Box<PythonCore>,
        right: Box<PythonCore>,
    },
    Break,
    Continue,
    Return {
        expr: Box<PythonCore>,
    },
    /// Python's `del <name>`, removing a name binding entirely.
    /// Used to keep a generated `for` loop's variable from leaking into the enclosing scope when Mamba's own scoping says it shouldn't.
    /// Generally, Mamba's scoping rules are stricter than Python's.
    Del {
        name: String,
    },
    UnderScore,
    Pass,
    None,
    Empty,
    TryExcept {
        setup: Option<Box<PythonCore>>,
        attempt: Box<PythonCore>,
        except: Vec<PythonCore>,
    },
    ExceptId {
        id: Box<PythonCore>,
        class: Box<PythonCore>,
        body: Box<PythonCore>,
    },
    Except {
        class: Box<PythonCore>,
        body: Box<PythonCore>,
    },
    Raise {
        error: Box<PythonCore>,
    },
    With {
        resource: Box<PythonCore>,
        expr: Box<PythonCore>,
    },
    WithAs {
        resource: Box<PythonCore>,
        alias: Box<PythonCore>,
        expr: Box<PythonCore>,
    },
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum CoreOp {
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    PowAssign,
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
            NodeOp::Assign => Ok(CoreOp::Assign),
            op => Err(UnimplementedErr::new(ast, &format!("Reassign with {op}"))),
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
            function::python::GE => CoreFunOp::Ge,
            function::python::GEQ => CoreFunOp::Geq,
            function::python::LE => CoreFunOp::Le,
            function::python::LEQ => CoreFunOp::Leq,
            function::python::EQ => CoreFunOp::Eq,
            function::python::NEQ => CoreFunOp::Neq,
            function::python::ADD => CoreFunOp::Add,
            function::python::SUB => CoreFunOp::Sub,
            function::python::POW => CoreFunOp::Pow,
            function::python::MUL => CoreFunOp::Mul,
            function::python::MOD => CoreFunOp::Mod,
            function::python::DIV => CoreFunOp::Div,
            function::python::FDIV => CoreFunOp::FDiv,
            _ => return None,
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

use std::fmt::{Display, Error, Formatter};

use crate::check::context::function;

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum NodeOp {
    Assign,
    Add,
    Sub,
    Sqrt,
    Mul,
    FDiv,
    Div,
    Pow,
    Mod,
    Eq,
    Le,
    Ge,
    BLShift,
    BRShift,
}

impl Display for NodeOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match &self {
            NodeOp::Assign => write!(f, ":="),
            NodeOp::Add => write!(f, "{}", function::ADD),
            NodeOp::Sub => write!(f, "{}", function::SUB),
            NodeOp::Sqrt => write!(f, "{}", function::SQRT),
            NodeOp::Mul => write!(f, "{}", function::MUL),
            NodeOp::FDiv => write!(f, "{}", function::FDIV),
            NodeOp::Div => write!(f, "{}", function::DIV),
            NodeOp::Pow => write!(f, "{}", function::POW),
            NodeOp::Mod => write!(f, "{}", function::MOD),
            NodeOp::Eq => write!(f, "{}", function::EQ),
            NodeOp::Le => write!(f, "{}", function::LE),
            NodeOp::Ge => write!(f, "{}", function::GE),
            NodeOp::BLShift => write!(f, "<<"),
            NodeOp::BRShift => write!(f, ">>"),
        }
    }
}

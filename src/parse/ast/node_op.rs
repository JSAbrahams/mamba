use std::fmt::{Display, Error, Formatter};

use crate::check::context::function::python::{ADD, DIV, EQ, FDIV, GE, LE, MOD, MUL, POW, SUB};
use crate::check::context::function::SQRT;

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
            NodeOp::Add => write!(f, "{ADD}"),
            NodeOp::Sub => write!(f, "{SUB}"),
            NodeOp::Sqrt => write!(f, "{SQRT}"),
            NodeOp::Mul => write!(f, "{MUL}"),
            NodeOp::FDiv => write!(f, "{FDIV}"),
            NodeOp::Div => write!(f, "{DIV}"),
            NodeOp::Pow => write!(f, "{POW}"),
            NodeOp::Mod => write!(f, "{MOD}"),
            NodeOp::Eq => write!(f, "{EQ}"),
            NodeOp::Le => write!(f, "{LE}"),
            NodeOp::Ge => write!(f, "{GE}"),
            NodeOp::BLShift => write!(f, "<<"),
            NodeOp::BRShift => write!(f, ">>"),
        }
    }
}

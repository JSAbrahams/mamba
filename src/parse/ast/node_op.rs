use std::fmt::{Display, Error, Formatter};

use crate::parse::lex::token::Token;

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
            NodeOp::Assign => write!(f, "{}", Token::Assign),
            NodeOp::Add => write!(f, "{}", Token::Add),
            NodeOp::Sub => write!(f, "{}", Token::Sub),
            NodeOp::Sqrt => write!(f, "{}", Token::Sqrt),
            NodeOp::Mul => write!(f, "{}", Token::Mul),
            NodeOp::FDiv => write!(f, "{}", Token::FDiv),
            NodeOp::Div => write!(f, "{}", Token::Div),
            NodeOp::Pow => write!(f, "{}", Token::Pow),
            NodeOp::Mod => write!(f, "{}", Token::Mod),
            NodeOp::Eq => write!(f, "{}", Token::Eq),
            NodeOp::Le => write!(f, "{}", Token::Le),
            NodeOp::Ge => write!(f, "{}", Token::Ge),
            NodeOp::BLShift => write!(f, "{}", Token::BLShift),
            NodeOp::BRShift => write!(f, "{}", Token::BRShift),
        }
    }
}

use crate::lexer::token::Token;
use crate::parser::ast::{Node, AST};
use crate::type_checker::type_result::TypeErr;

pub fn try_from_id(node_pos: &AST) -> Result<String, TypeErr> {
    match &node_pos.node {
        Node::Id { lit } => Ok(lit.clone()),
        Node::_Self => Ok(Token::_Self.to_string()),
        Node::Init => Ok(Token::Init.to_string()),

        Node::GeOp => Ok(Token::Ge.to_string()),
        Node::LeOp => Ok(Token::Le.to_string()),
        Node::EqOp => Ok(Token::Eq.to_string()),
        Node::AddOp => Ok(Token::Add.to_string()),
        Node::SubOp => Ok(Token::Sub.to_string()),
        Node::PowOp => Ok(Token::Pow.to_string()),
        Node::MulOp => Ok(Token::Mul.to_string()),
        Node::ModOp => Ok(Token::Mod.to_string()),
        Node::DivOp => Ok(Token::Div.to_string()),
        Node::FDivOp => Ok(Token::FDiv.to_string()),

        _ => Err(TypeErr::new(&node_pos.pos, "Expected identifier"))
    }
}

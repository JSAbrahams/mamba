use crate::lexer::token::Token;
use crate::parser::ast::{ASTNode, ASTNodePos};
use crate::type_checker::type_result::TypeErr;

pub fn try_from_id(node_pos: &ASTNodePos) -> Result<String, TypeErr> {
    match &node_pos.node {
        ASTNode::Id { lit } => Ok(lit.clone()),
        ASTNode::_Self => Ok(Token::_Self.to_string()),
        ASTNode::Init => Ok(Token::Init.to_string()),

        ASTNode::GeOp => Ok(Token::Ge.to_string()),
        ASTNode::LeOp => Ok(Token::Le.to_string()),
        ASTNode::EqOp => Ok(Token::Eq.to_string()),
        ASTNode::AddOp => Ok(Token::Add.to_string()),
        ASTNode::SubOp => Ok(Token::Sub.to_string()),
        ASTNode::PowOp => Ok(Token::Pow.to_string()),
        ASTNode::MulOp => Ok(Token::Mul.to_string()),
        ASTNode::ModOp => Ok(Token::Mod.to_string()),
        ASTNode::DivOp => Ok(Token::Div.to_string()),
        ASTNode::FDivOp => Ok(Token::FDiv.to_string()),

        _ => Err(TypeErr::new(&node_pos.position, "Expected identifier"))
    }
}

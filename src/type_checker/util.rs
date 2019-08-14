use crate::parser::ast::{ASTNode, ASTNodePos};

pub trait ExtractStmtExt {
    fn statements(&self) -> Result<Vec<ASTNodePos>, String>;
}

impl ExtractStmtExt for ASTNodePos {
    fn statements(&self) -> Result<Vec<ASTNodePos>, String> {
        match &self.node {
            ASTNode::Block { statements } => Ok(statements.clone()),
            other => Err(format!("Expected block {:?}", other))
        }
    }
}

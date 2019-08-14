use crate::parser::ast::{ASTNode, ASTNodePos};

pub trait ExtractStmtExt {
    fn statements(&self) -> Result<Vec<ASTNodePos>, String>;
}

impl ExtractStmtExt for ASTNodePos {
    /// Extract statements from a block, excluding comments
    fn statements(&self) -> Result<Vec<ASTNodePos>, String> {
        match &self.node {
            ASTNode::Block { statements } => Ok(statements
                .iter()
                .filter(|node_pos| match &node_pos.node {
                    ASTNode::Comment { .. } => false,
                    _ => true
                })
                .cloned()
                .collect()),
            other => Err(format!("Expected block {:?}", other))
        }
    }
}

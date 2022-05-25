use std::ops::Deref;

use crate::check::name::Name;
use crate::common::position::Position;
use crate::parse::ast::{AST, Node};

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct ASTTy {
    pub pos: Position,
    pub node: Node,
    pub ty: Option<Name>,
}

impl From<&AST> for ASTTy {
    fn from(ast: &AST) -> Self {
        ASTTy { pos: ast.pos.clone(), node: ast.node.clone(), ty: None }
    }
}

impl From<&Box<AST>> for ASTTy {
    fn from(ast: &Box<AST>) -> Self {
        ASTTy::from(ast.deref())
    }
}

impl ASTTy {
    pub fn to_ty(self, ty: &Name) -> ASTTy {
        ASTTy { ty: Some(ty.clone()), ..self }
    }
}

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

#[cfg(test)]
mod test {
    use crate::{AST, ASTTy};
    use crate::check::name::Name;
    use crate::common::position::{CaretPos, Position};
    use crate::parse::ast::Node;

    #[test]
    fn from_ast() {
        let node = Node::Pass;
        let pos = Position::from(&CaretPos::new(4, 8));
        let ast = AST::new(&pos, node.clone());

        let ast_ty = ASTTy::from(&ast);
        let ast_ty2 = ASTTy::from(&Box::from(ast));

        assert_eq!(ast_ty, ast_ty2);
        assert_eq!(ast_ty.node, node);
        assert_eq!(ast_ty.pos, pos);
    }

    #[test]
    fn to_ty() {
        let node = Node::Pass;
        let ast = AST::new(&Position::default(), node.clone());
        let ast_ty = ASTTy::from(&ast).to_ty(&Name::from("Dummy"));

        assert_eq!(ast_ty.ty, Some(Name::from("Dummy")));
    }
}

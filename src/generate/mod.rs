use crate::check::ast::ASTTy;
use crate::generate::convert::convert_node;
use crate::generate::convert::state::{Imports, State};
use crate::generate::result::GenResult;

mod convert;

pub mod ast;

pub mod result;

/// Consumes the given [AST](mamba::parser::ast::AST) and produces
/// a [Core](mamba::generate.ast::construct::Core) node.
///
/// Note that the given [AST](mamba::parser::ast::AST) must be
/// correctly formed. Therefore, malformed
/// [AST](mamba::parser::ast::AST)'s should be caught by either
/// the parser or the type checker.
///
/// # Examples
///
/// ```
/// # use mamba::check::ast::ASTTy;
/// # use mamba::parse::ast::Node;
/// # use mamba::parse::ast::AST;
/// # use mamba::generate::ast::node::Core;
/// # use mamba::common::position::{CaretPos, Position};
/// # use mamba::generate::gen;
/// let node = Node::ReturnEmpty;
/// let ast = AST::new(&Position::new(&CaretPos::new(1, 1), &CaretPos::new(1, 5)), node);
/// let ast_ty = ASTTy::from(&ast);
/// let core_result = gen(&ast_ty).unwrap();
///
/// assert_eq!(core_result, Core::Return { expr: Box::from(Core::None) });
/// ```
///
/// # Failures
///
/// Fails if converting a construct which has not been implemented yet.
///
/// ```rust
/// # use mamba::check::ast::ASTTy;
/// # use mamba::parse::ast::Node;
/// # use mamba::parse::ast::AST;
/// # use mamba::generate::ast::node::Core;
/// # use mamba::common::position::{CaretPos, Position};
/// # use mamba::generate::gen;
/// let cond_node = Node::Int { lit: String::from("56") };
/// let cond_pos = AST::new(&Position::new(&CaretPos::new(0, 0), &CaretPos::new(0, 5)), cond_node);
/// let node = Node::Condition { cond: Box::from(cond_pos), el: None };
/// let ast = AST::new(&Position::new(&CaretPos::new(0, 0), &CaretPos::new(0, 5)), node);
/// let ast_ty = ASTTy::from(&ast);
/// let core_result = gen(&ast_ty);
///
/// assert!(core_result.is_err());
/// ```
///
/// # Panics
///
/// A malformed [AST](crate::parser::ast::AST) causes this stage
/// to panic.
pub fn gen(ast_ty: &ASTTy) -> GenResult {
    convert_node(ast_ty, &mut Imports::new(), &State::new())
}

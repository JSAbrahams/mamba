use crate::core::construct::Core;
use crate::desugar::context::Context;
use crate::desugar::context::State;
use crate::desugar::node::desugar_node;
use crate::parser::ast::ASTNodePos;

mod call;
mod common;
mod context;
mod control_flow;
mod definition;
mod node;

/// Consumes the given [ASTNodePos](crate::parser::ast::ASTNodePos) and produces
/// a [Core](crate::core::construct::Core) node.
///
/// Note that the given [ASTNodePos](crate::parser::ast::ASTNodePos) must be
/// correctly formed. Therefore, malformed
/// [ASTNodePos](crate::parser::ast::ASTNodePos)'s should be caught by either
/// the parser or the type checker.
///
/// The desugar stage itself should never throw an error.
///
/// # Examples
///
/// ```
/// # use mamba::parser::ast::ASTNode;
/// # use mamba::parser::ast::ASTNodePos;
/// # use mamba::desugar::desugar;
/// # use mamba::core::construct::Core;
/// let node = ASTNode::ReturnEmpty;
/// let ast_node_pos = ASTNodePos { st_line: 0, st_pos: 0, en_line: 0, en_pos: 5, node };
/// let core_node = desugar(&ast_node_pos);
///
/// assert_eq!(core_node, Core::Return { expr: Box::from(Core::Empty) });
/// ```
///
/// # Panics
///
/// A malformed [ASTNodePos](crate::parser::ast::ASTNodePos) would for instance
/// be a definition which does not contain a definition.
///
/// ```rust,should_panic
/// # use mamba::parser::ast::ASTNode;
/// # use mamba::parser::ast::ASTNodePos;
/// # use mamba::desugar::desugar;
/// let ast_literal_node_pos = Box::from(ASTNodePos {
///     st_line: 0, st_pos: 4, en_line: 0, en_pos: 6, node: ASTNode::Int { lit: String::from("10") }
/// });
/// let ast_definition_node_pos = Box::from(ASTNodePos {
///     st_line: 0, st_pos: 0, en_line: 0, en_pos: 6,
///     node: ASTNode::Def { private: false, definition: ast_literal_node_pos }
/// });
///
/// // should panic since definition is a literal
/// let core_node = desugar(&ast_definition_node_pos);
/// ```
///
pub fn desugar(input: &ASTNodePos) -> Core { desugar_node(&input, &Context::new(), &State::new()) }

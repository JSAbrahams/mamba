use crate::desugar::context::Imports;
use crate::desugar::context::State;
use crate::desugar::desugar_result::DesugarResult;
use crate::desugar::node::desugar_node;
use crate::parser::ast::ASTNodePos;
use std::path::PathBuf;

mod call;
mod class;
mod common;
mod context;
mod control_flow;
mod definition;
mod node;

pub mod desugar_result;

/// Consumes the given [ASTNodePos](crate::parser::ast::ASTNodePos) and produces
/// a [Core](crate::core::construct::Core) node.
///
/// Note that the given [ASTNodePos](crate::parser::ast::ASTNodePos) must be
/// correctly formed. Therefore, malformed
/// [ASTNodePos](crate::parser::ast::ASTNodePos)'s should be caught by either
/// the parser or the type checker.
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
/// let core_result = desugar(&ast_node_pos).unwrap();
///
/// assert_eq!(core_result, Core::Return { expr: Box::from(Core::None) });
/// ```
///
/// # Failures
///
/// Fails if desugaring a construct which has not been implemented yet.
///
/// ```rust
/// # use mamba::parser::ast::ASTNode;
/// # use mamba::parser::ast::ASTNodePos;
/// # use mamba::desugar::desugar;
/// # use mamba::core::construct::Core;
/// let cond_node = ASTNode::Int { lit: String::from("56") };
/// let cond_pos =
///     ASTNodePos { st_line: 0, st_pos: 0, en_line: 0, en_pos: 5, node: cond_node };
/// let node = ASTNode::Condition { cond: Box::from(cond_pos), _else: None };
/// let ast_node_pos = ASTNodePos { st_line: 0, st_pos: 0, en_line: 0, en_pos: 5, node };
/// let core_result = desugar(&ast_node_pos);
///
/// assert!(core_result.is_err());
/// ```
///
/// # Panics
///
/// A malformed [ASTNodePos](crate::parser::ast::ASTNodePos) causes this stage
/// to panic. A malformed [ASTNodePos](crate::parser::ast::ASTNodePos) would for
/// instance be a definition which does not contain a definition.
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
pub fn desugar(input: &ASTNodePos) -> DesugarResult {
    desugar_node(&input, &mut Imports::new(), &State::new())
}

pub fn desugar_all(inputs: &[(ASTNodePos, Option<String>, Option<PathBuf>)]) -> Vec<DesugarResult> {
    inputs
        .iter()
        .map(|(node_pos, source, path)| (desugar(node_pos), source, path))
        .map(|(result, source, path)| result.map_err(|err| err.into_with_source(source, path)))
        .collect()
}

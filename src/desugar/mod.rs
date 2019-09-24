use std::path::PathBuf;

use crate::desugar::context::Imports;
use crate::desugar::context::State;
use crate::desugar::desugar_result::{DesugarResult, DesugarResults};
use crate::desugar::node::desugar_node;
use crate::parser::ast::AST;

mod call;
mod class;
mod common;
mod context;
mod control_flow;
mod definition;
mod node;

pub mod desugar_result;

pub type DesugarInput = (AST, Option<String>, Option<PathBuf>);

/// Consumes the given [AST](crate::parser::ast::AST) and produces
/// a [Core](crate::core::construct::Core) node.
///
/// Note that the given [AST](crate::parser::ast::AST) must be
/// correctly formed. Therefore, malformed
/// [AST](crate::parser::ast::AST)'s should be caught by either
/// the parser or the type checker.
///
/// # Examples
///
/// ```
/// # use mamba::parser::ast::Node;
/// # use mamba::parser::ast::AST;
/// # use mamba::desugar::desugar;
/// # use mamba::core::construct::Core;
/// # use mamba::common::position::{EndPoint, Position};
/// let node = Node::ReturnEmpty;
/// let ast_node_pos = AST::new(&Position::new(&EndPoint::new(1, 1), &EndPoint::new(1, 5)), node);
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
/// # use mamba::parser::ast::Node;
/// # use mamba::parser::ast::AST;
/// # use mamba::desugar::desugar;
/// # use mamba::core::construct::Core;
/// use mamba::common::position::{EndPoint, Position};
/// let cond_node = Node::Int { lit: String::from("56") };
/// let cond_pos = AST::new(&Position::new(&EndPoint::new(0, 0), &EndPoint::new(0, 5)), cond_node);
/// let node = Node::Condition { cond: Box::from(cond_pos), _else: None };
/// let ast_node_pos = AST::new(&Position::new(&EndPoint::new(0, 0), &EndPoint::new(0, 5)), node);
/// let core_result = desugar(&ast_node_pos);
///
/// assert!(core_result.is_err());
/// ```
///
/// # Panics
///
/// A malformed [AST](crate::parser::ast::AST) causes this stage
/// to panic.
pub fn desugar(input: &AST) -> DesugarResult {
    desugar_node(&input, &mut Imports::new(), &State::new())
}

pub fn desugar_all(inputs: &[DesugarInput]) -> DesugarResults {
    let inputs: Vec<_> = inputs
        .iter()
        .map(|(node_pos, source, path)| (desugar(node_pos), source, path))
        .map(|(result, source, path)| {
            (result.map_err(|err| err.into_with_source(source, path)), source.clone(), path.clone())
        })
        .collect();

    let (oks, errs): (Vec<_>, Vec<_>) = inputs.iter().partition(|(res, ..)| res.is_ok());
    if errs.is_empty() {
        Ok(oks
            .iter()
            .map(|(res, src, path)| (res.as_ref().unwrap().clone(), src.clone(), path.clone()))
            .collect())
    } else {
        Err(errs.iter().map(|(res, ..)| res.as_ref().unwrap_err().clone()).collect())
    }
}

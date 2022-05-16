use std::path::PathBuf;

use crate::generate::convert::convert_node;
use crate::generate::convert::state::{Imports, State};
use crate::generate::result::{GenResult, GenResults};
use crate::parse::ast::AST;

mod convert;

pub mod ast;

pub mod result;

pub type DesugarInput = (AST, Option<String>, Option<PathBuf>);

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
/// # use mamba::parse::ast::Node;
/// # use mamba::parse::ast::AST;
/// # use mamba::generate::ast::node::Core;
/// # use mamba::common::position::{CaretPos, Position};
/// # use mamba::generate::gen;
/// let node = Node::ReturnEmpty;
/// let ast = AST::new(&Position::new(&CaretPos::new(1, 1), &CaretPos::new(1, 5)), node);
/// let core_result = gen(&ast).unwrap();
///
/// assert_eq!(core_result, Core::Return { expr: Box::from(Core::None) });
/// ```
///
/// # Failures
///
/// Fails if converting a construct which has not been implemented yet.
///
/// ```rust
/// # use mamba::parse::ast::Node;
/// # use mamba::parse::ast::AST;
/// # use mamba::generate::ast::node::Core;
/// # use mamba::common::position::{CaretPos, Position};
/// # use mamba::generate::gen;
/// let cond_node = Node::Int { lit: String::from("56") };
/// let cond_pos = AST::new(&Position::new(&CaretPos::new(0, 0), &CaretPos::new(0, 5)), cond_node);
/// let node = Node::Condition { cond: Box::from(cond_pos), el: None };
/// let ast = AST::new(&Position::new(&CaretPos::new(0, 0), &CaretPos::new(0, 5)), node);
/// let core_result = gen(&ast);
///
/// assert!(core_result.is_err());
/// ```
///
/// # Panics
///
/// A malformed [AST](crate::parser::ast::AST) causes this stage
/// to panic.
pub fn gen(input: &AST) -> GenResult {
    convert_node(input, &mut Imports::new(), &State::new())
}

pub fn gen_all(inputs: &[DesugarInput]) -> GenResults {
    let inputs: Vec<_> = inputs
        .iter()
        .map(|(ast, source, path)| (gen(ast), source, path))
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

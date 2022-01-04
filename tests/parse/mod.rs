use mamba::lex::tokenize;
use mamba::parse::parse;

use crate::common::*;

pub mod invalid;
pub mod valid;
mod util;

#[test]
fn parse_empty_file() {
    let source = resource_content(true, &[], "empty_file.mamba");
    parse(&tokenize(&source).unwrap()).unwrap();
}

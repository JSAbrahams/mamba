use crate::util::*;
use mamba::lexer::tokenize;
use mamba::parser::ast::ASTNodePos;
use mamba::parser::parse;
use mamba::parser::parse_result::ParseErr;

pub mod invalid;
pub mod valid;

#[test]
fn parse_empty_file() -> Result<ASTNodePos, ParseErr> {
    let source = valid_resource_content(&[], "empty_file.mamba");
    parse(&tokenize(&source).unwrap())
}

use crate::common::resource_content;
use mamba::lexer::tokenize;
use mamba::parser::parse;
use mamba::type_checker::stage_1::Context;

#[test]
fn assign_type_to_self() -> Result<(), Vec<String>> {
    let source = resource_content(false, &["function"], "assign_type_to_self.mamba");
    let ast_node = *parse(&tokenize(&source).unwrap()).unwrap();

    assert!(Context::new(&[ast_node]).is_err());
    Ok(())
}

#[test]
fn top_level_with_self() -> Result<(), Vec<String>> {
    let source = resource_content(false, &["function"], "top_level_with_self.mamba");
    let ast_node = *parse(&tokenize(&source).unwrap()).unwrap();

    assert!(Context::new(&[ast_node]).is_err());
    Ok(())
}

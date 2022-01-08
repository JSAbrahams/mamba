use mamba::lex::tokenize;
use mamba::parse::parse;

use crate::common::resource_content;

#[test]
fn type_annotation_in_tuple() {
    let source = resource_content(false, &["syntax"], "type_annotation_in_tuple.mamba");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}


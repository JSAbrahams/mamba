use mamba::check::check_all;
use mamba::lex::tokenize;
use mamba::parse::parse;

use crate::common::resource_content;

#[test]
fn top_level_tuple() {
    let source = resource_content(true, &["class"], "top_level_tuple.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap();
}

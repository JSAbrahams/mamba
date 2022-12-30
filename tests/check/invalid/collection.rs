use mamba::check::check_all;
use mamba::parse::parse;

use crate::common::resource_content;

#[test]
fn conflicting_collection_types() {
    let source = resource_content(false, &["type", "collection"], "conflicting_collection_types.mamba");
    check_all(&[*parse(&source).unwrap()]).unwrap_err();
}

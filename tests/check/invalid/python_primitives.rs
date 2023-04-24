use std::convert::TryFrom;

use mamba::check::check_all;
use mamba::check::context::{Context, LookupClass};
use mamba::check::name::string_name::StringName;
use mamba::common::position::Position;
use mamba::parse::ast::AST;

use crate::common::resource_content;

#[test]
fn float_and() {
    let source = resource_content(false, &["type", "control_flow"], "float_and.mamba");
    check_all(&[source.parse::<AST>().unwrap()]).unwrap_err();
}

#[test]
pub fn non_existent_primitive() {
    let files = vec![];
    let context = Context::try_from(files.as_slice()).unwrap();
    let context = context.into_with_primitives().unwrap();

    context.class(&StringName::from("nothing"), Position::invisible()).unwrap_err();
}

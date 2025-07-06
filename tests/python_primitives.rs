use std::convert::TryFrom;

use mamba::check::context::{Context, LookupClass};
use mamba::check::name::string_name::StringName;
use mamba::common::position::Position;

#[test]
pub fn non_existent_primitive() {
    let files = vec![];
    let context = Context::try_from(files.as_slice()).unwrap();
    let context = context.into_with_primitives().unwrap();

    context
        .class(&StringName::from("nothing"), Position::invisible())
        .unwrap_err();
}

use std::convert::TryFrom;

use mamba::check::CheckInput;
use mamba::check::context::{Context, LookupClass};
use mamba::check::name::stringname::StringName;
use mamba::common::position::Position;

#[test]
pub fn primitives_present() {
    let files: Vec<CheckInput> = vec![];
    let context = Context::try_from(files.as_slice()).unwrap();
    let context = context.into_with_primitives().unwrap();

    context.class(&StringName::from("String"), &Position::default()).unwrap();
    context.class(&StringName::from("Bool"), &Position::default()).unwrap();
    context.class(&StringName::from("Float"), &Position::default()).unwrap();
    context.class(&StringName::from("Int"), &Position::default()).unwrap();
    context.class(&StringName::from("Complex"), &Position::default()).unwrap();
}

#[test]
pub fn std_lib_present() {
    let files: Vec<CheckInput> = vec![];
    let context = Context::try_from(files.as_slice()).unwrap();
    let context = context.into_with_std_lib().unwrap();

    // TODO Test for Set and List
    context.class(&StringName::from("Range"), &Position::default()).unwrap();
    context.class(&StringName::from("None"), &Position::default()).unwrap();
    context.class(&StringName::from("Exception"), &Position::default()).unwrap();
}

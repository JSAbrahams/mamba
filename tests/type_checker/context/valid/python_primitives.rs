use std::convert::TryFrom;

use mamba::common::position::Position;
use mamba::type_checker::context::generic::type_name::GenericType;
use mamba::type_checker::context::Context;
use mamba::type_checker::CheckInput;

#[test]
pub fn non_existent_primitive() {
    let files: Vec<CheckInput> = vec![];
    let context = Context::try_from(files.as_slice()).unwrap();
    let context = context.into_with_primitives().unwrap();

    context.lookup(&GenericType::new("nothing"), &Position::default()).unwrap_err();
}

#[test]
pub fn primitives_present() {
    let files: Vec<CheckInput> = vec![];
    let context = Context::try_from(files.as_slice()).unwrap();
    let context = context.into_with_primitives().unwrap();

    context.lookup(&GenericType::new("str"), &Position::default()).unwrap();
    context.lookup(&GenericType::new("bool"), &Position::default()).unwrap();
    context.lookup(&GenericType::new("float"), &Position::default()).unwrap();
    context.lookup(&GenericType::new("int"), &Position::default()).unwrap();
    context.lookup(&GenericType::new("complex"), &Position::default()).unwrap();
}

use std::convert::TryFrom;

use mamba::common::position::Position;
use mamba::type_checker::context::generic::type_name::GenericActualTypeName;
use mamba::type_checker::context::Context;
use mamba::type_checker::CheckInput;

#[test]
pub fn non_existent_primitive() {
    let files: Vec<CheckInput> = vec![];
    let context = Context::try_from(files.as_slice()).unwrap();
    let context = context.into_with_primitives().unwrap();

    context.lookup(&GenericActualTypeName::new("nothing"), &Position::default()).unwrap_err();
}

#[test]
pub fn primitives_present() {
    let files: Vec<CheckInput> = vec![];
    let context = Context::try_from(files.as_slice()).unwrap();
    let context = context.into_with_primitives().unwrap();

    context.lookup(&GenericActualTypeName::new("str"), &Position::default()).unwrap();
    context.lookup(&GenericActualTypeName::new("bool"), &Position::default()).unwrap();
    context.lookup(&GenericActualTypeName::new("float"), &Position::default()).unwrap();
    context.lookup(&GenericActualTypeName::new("int"), &Position::default()).unwrap();
    context.lookup(&GenericActualTypeName::new("complex"), &Position::default()).unwrap();
}

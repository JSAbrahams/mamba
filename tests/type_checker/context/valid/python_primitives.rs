use std::convert::TryFrom;

use mamba::common::position::Position;
use mamba::type_checker::context::concrete::type_name::TypeName;
use mamba::type_checker::context::generic::type_name::GenericTypeName;
use mamba::type_checker::context::Context;
use mamba::type_checker::CheckInput;

#[test]
pub fn non_existent_primitive() {
    let files: Vec<CheckInput> = vec![];
    let context = Context::try_from(files.as_slice()).unwrap();
    let context = context.into_with_primitives().unwrap();

    context.lookup(&GenericTypeName::new("nothing"), &Position::default()).unwrap_err();
}

#[test]
pub fn primitives_present() {
    let files: Vec<CheckInput> = vec![];
    let context = Context::try_from(files.as_slice()).unwrap();
    let context = context.into_with_primitives().unwrap();

    context.lookup(&GenericTypeName::new("str"), &Position::default()).unwrap();
    context.lookup(&GenericTypeName::new("bool"), &Position::default()).unwrap();
    context.lookup(&GenericTypeName::new("float"), &Position::default()).unwrap();
    context.lookup(&GenericTypeName::new("int"), &Position::default()).unwrap();
    context.lookup(&GenericTypeName::new("complex"), &Position::default()).unwrap();
}

use std::convert::TryFrom;

use mamba::common::position::Position;
use mamba::type_checker::context::type_name::TypeName;
use mamba::type_checker::context::Context;
use mamba::type_checker::CheckInput;

#[test]
pub fn non_existent_primitive() {
    let files: Vec<CheckInput> = vec![];
    let context = Context::try_from(files.as_slice()).unwrap();
    let context = context.into_with_primitives().unwrap();

    context.lookup(&TypeName::new("nothing", &vec![]), &Position::default()).unwrap_err();
}

#[test]
pub fn primitives_present() {
    let files: Vec<CheckInput> = vec![];
    let context = Context::try_from(files.as_slice()).unwrap();
    let context = context.into_with_primitives().unwrap();

    context.lookup(&TypeName::new("str", &vec![]), &Position::default()).unwrap();
    context.lookup(&TypeName::new("bool", &vec![]), &Position::default()).unwrap();
    context.lookup(&TypeName::new("float", &vec![]), &Position::default()).unwrap();
    context.lookup(&TypeName::new("int", &vec![]), &Position::default()).unwrap();
    context.lookup(&TypeName::new("complex", &vec![]), &Position::default()).unwrap();
}

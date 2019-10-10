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

    context.lookup(&TypeName::new("String", &vec![]), &Position::default()).unwrap();
    context.lookup(&TypeName::new("Bool", &vec![]), &Position::default()).unwrap();
    context.lookup(&TypeName::new("Float", &vec![]), &Position::default()).unwrap();
    context.lookup(&TypeName::new("Int", &vec![]), &Position::default()).unwrap();
    context.lookup(&TypeName::new("Complex", &vec![]), &Position::default()).unwrap();
}

#[test]
pub fn std_lib_present() {
    let files: Vec<CheckInput> = vec![];
    let context = Context::try_from(files.as_slice()).unwrap();
    let context = context.into_with_std_lib().unwrap();

    context.lookup(&TypeName::new("Set", &vec![]), &Position::default()).unwrap();
    context.lookup(&TypeName::new("List", &vec![]), &Position::default()).unwrap();
    context.lookup(&TypeName::new("Range", &vec![]), &Position::default()).unwrap();
}

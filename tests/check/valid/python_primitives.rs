use std::convert::TryFrom;

use mamba::check::context::Context;
use mamba::check::ty::name::TypeName;
use mamba::check::CheckInput;
use mamba::common::position::Position;

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

    context
        .lookup(&TypeName::new("Set", &vec![TypeName::from("Int")]), &Position::default())
        .unwrap();
    context
        .lookup(&TypeName::new("List", &vec![TypeName::from("Something")]), &Position::default())
        .unwrap();
    context.lookup(&TypeName::new("Range", &vec![]), &Position::default()).unwrap();
    context.lookup(&TypeName::new("undefined", &vec![]), &Position::default()).unwrap();
    context.lookup(&TypeName::new("Exception", &vec![]), &Position::default()).unwrap();
}

use std::convert::TryFrom;

use mamba::check::context::Context;
use mamba::check::ty::Type;
use mamba::check::CheckInput;
use mamba::common::position::Position;

#[test]
pub fn primitives_present() {
    let files: Vec<CheckInput> = vec![];
    let context = Context::try_from(files.as_slice()).unwrap();
    let context = context.into_with_primitives().unwrap();

    context.class(&Type::new("String", &vec![]), &Position::default()).unwrap();
    context.class(&Type::new("Bool", &vec![]), &Position::default()).unwrap();
    context.class(&Type::new("Float", &vec![]), &Position::default()).unwrap();
    context.class(&Type::new("Int", &vec![]), &Position::default()).unwrap();
    context.class(&Type::new("Complex", &vec![]), &Position::default()).unwrap();
}

#[test]
pub fn std_lib_present() {
    let files: Vec<CheckInput> = vec![];
    let context = Context::try_from(files.as_slice()).unwrap();
    let context = context.into_with_std_lib().unwrap();

    context.class(&Type::new("Set", &vec![Type::from("Int")]), &Position::default()).unwrap();
    context
        .class(&Type::new("List", &vec![Type::from("Something")]), &Position::default())
        .unwrap();
    context.class(&Type::new("Range", &vec![]), &Position::default()).unwrap();
    context.class(&Type::new("undefined", &vec![]), &Position::default()).unwrap();
    context.class(&Type::new("Exception", &vec![]), &Position::default()).unwrap();
}

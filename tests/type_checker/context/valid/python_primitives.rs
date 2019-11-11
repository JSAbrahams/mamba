use std::convert::TryFrom;

use crate::common::resource_content;
use mamba::common::position::Position;
use mamba::lexer::tokenize;
use mamba::parser::parse;
use mamba::type_checker::check_all;
use mamba::type_checker::context::type_name::TypeName;
use mamba::type_checker::context::Context;
use mamba::type_checker::CheckInput;

#[test]
fn float_and() {
    let source = resource_content(false, &["type", "control_flow"], "float_and.mamba");
    check_all(&[(*parse(&tokenize(&source).unwrap()).unwrap(), None, None)]).unwrap_err();
}

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

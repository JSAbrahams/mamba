use crate::common::resource_content;
use mamba::common::position::Position;
use mamba::lexer::tokenize;
use mamba::parser::parse;
use mamba::type_checker::context::Context;
use mamba::type_checker::ty_name::TypeName;
use mamba::type_checker::{check_all, CheckInput};
use std::convert::TryFrom;

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

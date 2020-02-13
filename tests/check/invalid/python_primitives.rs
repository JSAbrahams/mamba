use crate::common::resource_content;
use mamba::check::context::Context;
use mamba::check::ty::concrete::ExpressionType;
use mamba::check::ty::name::TypeName;
use mamba::check::{check_all, CheckInput};
use mamba::common::position::Position;
use mamba::lex::tokenize;
use mamba::parse::parse;
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

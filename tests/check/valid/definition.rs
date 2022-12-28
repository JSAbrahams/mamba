use crate::check::{check_test, CheckTestRet};
use crate::common::resource_content;

#[test]
#[ignore]
fn all_mutable_in_call_chain() -> CheckTestRet {
    let source = resource_content(true, &["definition"], "all_mutable_in_call_chain.mamba");
    check_test(&source)
}

#[test]
fn nested_mut_field() -> CheckTestRet {
    let source = resource_content(true, &["definition"], "nested_mut_field.mamba");
    check_test(&source)
}

#[test]
#[ignore]
fn assign_to_inner_mut() -> CheckTestRet {
    let source = resource_content(true, &["definition"], "assign_to_inner_mut.mamba");
    check_test(&source)
}

#[test]
#[ignore]
fn nested_function() -> CheckTestRet {
    let source = resource_content(true, &["definition"], "nested_function.mamba");
    check_test(&source)
}

#[test]
fn tuple_modify_mut() -> CheckTestRet {
    let source = resource_content(true, &["definition"], "tuple_modify_mut.mamba");
    check_test(&source)
}

#[test]
fn tuple_modify_outer_mut() -> CheckTestRet {
    let source = resource_content(true, &["definition"], "tuple_modify_outer_mut.mamba");
    check_test(&source)
}

#[test]
fn f_strings() -> CheckTestRet {
    let source = resource_content(true, &["definition"], "f_strings.mamba");
    check_test(&source)
}

#[test]
#[ignore]
fn collection_in_f_strings() -> CheckTestRet {
    let source = resource_content(true, &["definition"], "collection_in_f_strings.mamba");
    check_test(&source)
}

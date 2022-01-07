use crate::check::{check_test, CheckTestRet};
use crate::common::resource_content;

#[test]
fn top_level_tuple() -> CheckTestRet {
    let source = resource_content(true, &["class"], "top_level_tuple.mamba");
    check_test(&source)
}

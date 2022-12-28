use crate::system::{OutTestRet, test_directory};

#[test]
fn factorial() -> OutTestRet {
    test_directory(true, &["readme_example"], &["readme_example", "target"], "factorial")
}

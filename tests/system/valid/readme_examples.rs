use crate::system::{OutTestRet, test_directory};

#[test]
fn error_handling() -> OutTestRet {
    test_directory(true, &["readme_example"], &["readme_example", "target"], "error_handling")
}

#[test]
fn factorial() -> OutTestRet {
    test_directory(true, &["readme_example"], &["readme_example", "target"], "factorial")
}

#[test]
fn factorial_dynamic() -> OutTestRet {
    test_directory(true, &["readme_example"], &["readme_example", "target"], "factorial_dynamic")
}

#[test]
fn handle() -> OutTestRet {
    test_directory(true, &["readme_example"], &["readme_example", "target"], "handle")
}

#[test]
#[ignore] // milestone 0.6 (type refinement)
fn pos_int() -> OutTestRet {
    test_directory(true, &["readme_example"], &["readme_example", "target"], "pos_int")
}

#[test]
#[ignore] // milestone 0.4.1
fn pure_functions() -> OutTestRet {
    test_directory(true, &["readme_example"], &["readme_example", "target"], "pure_functions")
}

#[test]
fn server_class() -> OutTestRet {
    test_directory(true, &["readme_example"], &["readme_example", "target"], "server_class")
}

#[test]
#[ignore] // milestone 0.6
fn type_refinement() -> OutTestRet {
    test_directory(true, &["readme_example"], &["readme_example", "target"], "type_refinement")
}

#[test]
#[ignore] // milestone 0.6
fn type_refinement_use() -> OutTestRet {
    test_directory(true, &["readme_example"], &["readme_example", "target"], "type_refinement_use")
}

#[test]
fn use_server() -> OutTestRet {
    test_directory(true, &["readme_example"], &["readme_example", "target"], "use_server")
}

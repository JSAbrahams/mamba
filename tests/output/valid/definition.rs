use crate::output::test_directory;

#[test]
fn long_f_string() -> Result<(), Vec<String>> {
    test_directory(true, &["definition"], &["definition", "target"], "long_f_string")
}

#[test]
#[ignore]  // re-evaluate parsing rules nameunion
fn assign_tuples() -> Result<(), Vec<String>> {
    test_directory(true, &["definition"], &["definition", "target"], "assign_tuples")
}

use crate::output::test_directory;

#[test]
fn long_f_string() -> Result<(), Vec<String>> {
    test_directory(true, &["definition"], &["definition", "target"], "long_f_string")
}

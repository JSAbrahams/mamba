use crate::util::valid_resource_path;
use mamba::command::quick_transpile;
use std::path::Path;

mod util;

#[test]
fn test_output_class() {
    let source = valid_resource_path(&String::from("class.txt"));
    let output = quick_transpile(Path::new(&source));
}

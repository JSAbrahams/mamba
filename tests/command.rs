use crate::util::valid_resource_path;
use mamba::command::mamba_to_python_direct;
use std::path::Path;

mod util;

#[test]
fn test_output_class() {
    let source = valid_resource_path(&String::from("class.txt"));
    let path = &mut Path::new(&source);

    mamba_to_python_direct(path);
}

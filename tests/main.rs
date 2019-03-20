use crate::util::check_valid_resource_exists_and_delete;
use crate::util::valid_resource_path;
use assert_cmd::prelude::*;
use std::process::Command;

mod util;

#[test]
fn command_line_class() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::main_binary()?;
    cmd.arg("-i").arg(valid_resource_path(&["class"],"class"));

    cmd.output().unwrap();

    if check_valid_resource_exists_and_delete(&["class"],"class.py") {
        Ok(())
    } else {
        let output = valid_resource_path(&["class"],"class.py");
        panic!("no output file found. {}", output)
    }
}

#[test]
fn command_line_class_with_output() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::main_binary()?;
    let output = valid_resource_path(&["class"],"class.py");
    cmd.arg("-i").arg(valid_resource_path(&["class"],"class.txt")).arg("-o").arg(output.clone());

    cmd.output().unwrap();

    if check_valid_resource_exists_and_delete(&["class"],"class.py") {
        Ok(())
    } else {
        panic!("no output file found: {}", output)
    }
}

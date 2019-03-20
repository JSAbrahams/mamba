use crate::util::check_valid_resource_exists_and_delete;
use assert_cmd::prelude::*;
// Run programs
use crate::util::valid_resource_path;
use std::process::Command;
// Add methods on commands

mod util;

#[test]
fn command_line_class() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::main_binary()?;
    cmd.arg("-i").arg(valid_resource_path("class"));

    cmd.output().unwrap();
    if check_valid_resource_exists_and_delete("class.py") {
        Ok(())
    } else {
        let output = format!("{}.py", valid_resource_path("class"));
        panic!("no output file found. {}", output)
    }
}

#[test]
fn command_line_class_with_output() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::main_binary()?;
    println!("{:?}", cmd);

    let output = format!("{}.py", valid_resource_path("class"));
    cmd.arg("-i").arg(valid_resource_path("class.txt")).arg("-o").arg(output.clone());

    cmd.output().unwrap();
    if check_valid_resource_exists_and_delete("class.py") {
        Ok(())
    } else {
        panic!("no output file found: {}", output)
    }
}

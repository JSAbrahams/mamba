use assert_cmd::prelude::*;
use crate::util::valid_resource_exists;
// Run programs
use crate::util::valid_resource_path;
use std::process::Command;
// Add methods on commands

mod util;

#[test]
fn command_line_class() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::main_binary()?;
    cmd.arg("mambac")
        .arg("-i")
        .arg(valid_resource_path("class"));

    cmd.output().unwrap();
    if valid_resource_exists("class") {
        Ok(())
    } else {
        panic!("no output file found.")
    }
}

#[test]
fn command_line_class_with_output() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::main_binary()?;
    cmd.arg("mambac")
        .arg("-i")
        .arg(valid_resource_path("class"))
        .arg("-o")
        .arg(format!("{}.py", valid_resource_path("class")));

    cmd.output().unwrap();
    if valid_resource_exists("class") {
        Ok(())
    } else {
        panic!("no output file found.")
    }
}

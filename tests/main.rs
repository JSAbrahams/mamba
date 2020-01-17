use std::prelude::v1::Result::Ok;
use std::process::Command;
use std::process::Stdio;

use assert_cmd::prelude::*;

use crate::common::exists_and_delete;
use crate::common::resource_path;

#[macro_use]
mod common;

mod core;
mod desugar;
mod lexer;
mod output;
mod parser;
mod type_checker;

#[test]
fn command_line_class_no_output() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::main_binary()?;
    cmd.current_dir(resource_path(true, &["operation"], ""));

    let input = resource_path(true, &["operation"], "boolean.mamba");
    cmd.arg("-i").arg(input).stderr(Stdio::inherit()).stdout(Stdio::inherit()).output()?;

    assert!(exists_and_delete(true, &["operation", "target"], "boolean.py"));
    Ok(())
}

#[test]
fn command_line_class_with_output() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::main_binary()?;
    cmd.current_dir(resource_path(true, &["operation"], ""));

    let input = resource_path(true, &["operation"], "boolean.mamba");
    let output = resource_path(true, &["operation"], "my_target");
    cmd.arg("-i")
        .arg(input)
        .arg("-o")
        .arg(output)
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .output()?;

    assert!(exists_and_delete(true, &["operation", "my_target"], "boolean.py"));
    Ok(())
}

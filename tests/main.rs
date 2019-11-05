use crate::common::exists_and_delete;
use crate::common::resource_path;
use assert_cmd::prelude::*;
use std::prelude::v1::Result::Ok;
use std::process::Command;
use std::process::Stdio;

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
    cmd.current_dir(resource_path(true, &["class"], ""));

    let input = resource_path(true, &["class"], "types.mamba");
    cmd.arg("-i").arg(input).stderr(Stdio::inherit()).output()?;

    assert!(exists_and_delete(true, &["class", "target"], "types.py"));
    Ok(())
}

#[test]
fn command_line_class_with_output() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::main_binary()?;
    cmd.current_dir(resource_path(true, &["class"], ""));

    let input = resource_path(true, &["class"], "types.mamba");
    let output = resource_path(true, &["class"], "my_target");
    cmd.arg("-i").arg(input).arg("-o").arg(output).stderr(Stdio::inherit()).output()?;

    assert!(exists_and_delete(true, &["class", "my_target"], "types.py"));
    Ok(())
}

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
mod pipeline;

#[test]
fn command_line_class_no_output() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::main_binary()?;
    let input = resource_path(true, &["class"], "types.mamba");
    cmd.arg("-i").arg(input).stdout(Stdio::inherit()).output()?;

    Ok(assert!(exists_and_delete(true, &["class"], "types.py")))
}

#[test]
fn command_line_class_with_output() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::main_binary()?;
    let input = resource_path(true, &["class"], "types.mamba");
    let output = resource_path(true, &["class"], "types.py");
    cmd.arg("-i").arg(input).arg("-o").arg(output).stdout(Stdio::inherit()).output()?;

    Ok(assert!(exists_and_delete(true, &["class"], "types.py")))
}

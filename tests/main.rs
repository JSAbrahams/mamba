use std::process::Command;
use std::process::Stdio;

use assert_cmd::prelude::*;

use crate::common::exists_and_delete;
use crate::common::resource_path;

#[macro_use]
mod common;

mod check;
mod core;
mod desugar;
mod lex;
mod output;
mod parse;

#[test]
fn command_line_class_no_output() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::main_binary()?;
    cmd.current_dir(resource_path(true, &["class"], ""));

    let input = resource_path(true, &["class"], "types.mamba");
    cmd.arg("-i").arg(input).stderr(Stdio::inherit()).stdout(Stdio::inherit()).output()?;

    exists_and_delete(true, &["class", "target"], "types.py")
}

#[test]
fn command_line_class_with_output() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::main_binary()?;
    cmd.current_dir(resource_path(true, &["class"], ""));

    let input = resource_path(true, &["class"], "types.mamba");
    let output = resource_path(true, &["class"], "my_target");
    cmd.arg("-i").arg(input).arg("-o").arg(output);
    cmd.stderr(Stdio::inherit()).stdout(Stdio::inherit()).output()?;

    exists_and_delete(true, &["class", "my_target"], "types.py")
}

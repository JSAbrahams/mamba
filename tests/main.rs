use std::process::Command;
use std::process::Stdio;

use assert_cmd::prelude::*;

use crate::common::resource_path;
use crate::common::{delete_dir, resource_content_randomize};

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

    let output = resource_path(true, &["class", "target"], "");
    delete_dir(&output)
}

#[test]
fn command_line_class_with_output() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::main_binary()?;
    cmd.current_dir(resource_path(true, &["class"], ""));

    let input = resource_path(true, &["class"], "types.mamba");
    let output = resource_content_randomize(true, &["class"], "");
    cmd.arg("-i").arg(input).arg("-o").arg(&output.0);
    cmd.stderr(Stdio::inherit()).stdout(Stdio::inherit()).output()?;

    delete_dir(&output.0)
}

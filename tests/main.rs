use std::process::Command;
use std::process::Stdio;

use assert_cmd::prelude::*;

use crate::common::{delete_dir, resource_content_randomize};
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

    let output = resource_path(true, &["class", "target"], "");
    delete_dir(&output)
}

// TODO investigate why this test fails on Appveyor
#[test]
#[ignore]
fn command_line_class_with_output() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::main_binary()?;
    cmd.current_dir(resource_path(true, &["class"], ""));

    let input = resource_path(true, &["class"], "types.mamba");
    let (output_path, _) = resource_content_randomize(true, &["class"], "");
    cmd.arg("-v").arg("-i").arg(input).arg("-o").arg(&output_path);
    cmd.stderr(Stdio::inherit()).stdout(Stdio::inherit()).output()?;

    delete_dir(&output_path)
}

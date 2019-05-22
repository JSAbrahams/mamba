use crate::common::check_exists_and_delete;
use crate::common::resource_path;
use assert_cmd::prelude::*;
use std::prelude::v1::Result::Ok;
use std::process::Command;

#[macro_use]
mod common;

mod core;
mod desugar;
mod lexer;
mod output;
mod parser;

#[test]
fn command_line_class() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::main_binary()?;
    cmd.arg("-i").arg(resource_path(true, &["class"], "class"));

    cmd.output().unwrap();
    assert_eq!(check_exists_and_delete(true, &["class"], "class.py"), true);
    Ok(())
}

#[test]
fn command_line_class_with_output() -> Result<(), Box<std::error::Error>> {
    let mut cmd = Command::main_binary()?;
    let output = resource_path(true, &["class"], "class.py");
    cmd.arg("-i").arg(resource_path(true, &["class"], "class.mamba")).arg("-o").arg(output.clone());

    cmd.output().unwrap();
    assert_eq!(check_exists_and_delete(true, &["class"], "class.py"), true);
    Ok(())
}

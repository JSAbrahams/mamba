use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::Stdio;

use assert_cmd::prelude::*;

use crate::common::{delete_dir, resource_content_randomize};
use crate::common::resource_path;

#[macro_use]
mod common;

mod check;
mod generate;
mod system;

#[test]
fn command_line_class_no_output() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::main_binary()?;
    cmd.current_dir(resource_path(true, &["class"], ""));

    let input = resource_path(true, &["class"], "types.mamba");
    let res = cmd.arg("-i").arg(input).stderr(Stdio::inherit()).stdout(Stdio::inherit()).output();

    let output = resource_path(true, &["class", "target"], "");
    let del_res = delete_dir(&output);
    assert!(res.is_ok(), "{:?}", res);
    del_res
}

#[test]
fn command_line_class_with_output() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::main_binary()?;
    cmd.current_dir(resource_path(true, &["class"], ""));

    let input = resource_path(true, &["class"], "types.mamba");
    let (output_path, _) = resource_content_randomize(true, &["class"], "");
    cmd.arg("-v").arg("-i").arg(input).arg("-o").arg(&output_path);
    let res = cmd.stderr(Stdio::inherit()).stdout(Stdio::inherit()).output();

    let del_res = delete_dir(&output_path);
    assert!(res.is_ok(), "{:?}", res);
    del_res
}

#[test]
fn transpile_src_in_dir() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::main_binary()?;
    cmd.current_dir(resource_path(true, &["dummy", "proj1"], ""));

    cmd.arg("-v");
    assert!(cmd.stderr(Stdio::inherit()).stdout(Stdio::inherit()).output()?.ok().is_ok());
    let output_path = resource_path(true, &["dummy", "proj1", "target"], "");

    delete_dir(&output_path)
}

#[test]
fn transpile_custom_src_in_dir() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::main_binary()?;
    cmd.current_dir(resource_path(true, &["dummy", "proj1"], ""));

    cmd.arg("-v").arg("--input").arg("custom_src");
    let res = cmd.stderr(Stdio::inherit()).stdout(Stdio::inherit()).output()?.ok();
    assert!(res.is_ok());
    let output_path = resource_path(true, &["dummy", "proj1", "target"], "");

    let del_res = delete_dir(&output_path);
    del_res
}

#[test]
fn transpile_src_in_dir_custom_target() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::main_binary()?;
    cmd.current_dir(resource_path(true, &["dummy", "proj1"], ""));

    cmd.arg("-v").arg("--output").arg("custom_target");
    assert!(cmd.stderr(Stdio::inherit()).stdout(Stdio::inherit()).output()?.ok().is_ok());
    let output_path = resource_path(true, &["dummy", "proj1", "custom_target"], "");

    let del_res = delete_dir(&output_path);
    del_res
}

#[test]
fn transpile_file_not_src() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::main_binary()?;
    cmd.current_dir(resource_path(true, &["dummy", "proj1"], ""));

    let input_path = PathBuf::from("src").join(PathBuf::from("hello_world.mamba"));
    cmd.arg("-v").arg("--input").arg(input_path.as_os_str().to_str().unwrap());
    assert!(cmd.stderr(Stdio::inherit()).stdout(Stdio::inherit()).output()?.ok().is_ok());
    let output_path = resource_path(true, &["dummy", "proj1", "custom_target"], "");

    let del_res = delete_dir(&output_path);
    del_res
}

#[test]
fn transpile_non_existent_custom_src_in_dir() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::main_binary()?;
    cmd.current_dir(resource_path(true, &["dummy", "proj1"], ""));

    cmd.arg("-v").arg("-input non_existent");
    assert!(!cmd.output().err().is_some());
    Ok(())
}

#[test]
fn transpile_non_existent_file() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::main_binary()?;
    cmd.current_dir(resource_path(true, &["dummy", "proj1"], ""));

    let input_path = PathBuf::from("src").join(PathBuf::from("hello_world_does_not_exist.mamba"));
    cmd.arg("-v").arg("-input").arg(input_path.as_os_str().to_str().unwrap());
    assert!(!cmd.output().err().is_some());
    Ok(())
}

#[test]
fn err_output_relative_path_from_src() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::main_binary()?;
    cmd.current_dir(resource_path(false, &["dummy", "proj1"], ""));

    cmd.arg("-v");
    let res = String::from_utf8(cmd.output()?.stderr)?;

    let path_res_str = format!("─→ {}", Path::new("src").join(Path::new("hello_world.mamba")).to_str().unwrap());
    assert!(res.contains(&path_res_str), "err msg did not contain: \"{}\"\nerr:\n{}\n\n", path_res_str, res);
    Ok(())
}

#[test]
fn err_output_relative_path_from_custom_src() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::main_binary()?;
    cmd.current_dir(resource_path(false, &["dummy", "proj1"], ""));

    cmd.arg("-v").arg("--input").arg("custom_src");
    let res = String::from_utf8(cmd.output()?.stderr)?;

    let path_res_str = format!("─→ {}", Path::new("custom_src").join(Path::new("hello_world.mamba")).to_str().unwrap());
    assert!(res.contains(&path_res_str), "err msg did not contain: \"{}\"\nerr:\n{}\n\n", path_res_str, res);
    Ok(())
}

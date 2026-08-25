//! Runtime execution tests: unlike the AST-diff tests in `tests/check/valid.rs`
//! (`tests_util::test_directory`), which only check that generated Python is *structurally*
//! equivalent to a reference file, these actually run the compiled output and assert on what it
//! prints -- for both backends.

use std::path::Path;
use std::process::Command;

use mamba::backend::Backend;
use mamba::{transpile_dir, Arguments};
use tests_util::{resource_path, run_python};

#[test]
fn python_backend_prints_expected_output() -> Result<(), Box<dyn std::error::Error>> {
    let src_dir = resource_path(true, &["function"], "");
    let out_dir = tempfile::tempdir()?;

    let arguments = Arguments::default(); // backend defaults to `Backend::Python`
    let output_dir = transpile_dir(
        Path::new(&src_dir),
        Some("hello_world.mamba"),
        Some(out_dir.path().join("out").to_str().unwrap()),
        &arguments,
    )
    .map_err(|errs| format!("{errs:?}"))?;

    let stdout = run_python(&output_dir.join("hello_world.py"))?;
    assert_eq!(stdout, "hello world\n");
    Ok(())
}

#[test]
fn bin_backend_prints_expected_output() -> Result<(), Box<dyn std::error::Error>> {
    let src_dir = resource_path(true, &["function"], "");
    let out_dir = tempfile::tempdir()?;
    let bin_path = out_dir.path().join("hello_world_bin");

    let arguments = Arguments {
        annotate: false,
        backend: Backend::Bin { target: None },
    };
    let produced = transpile_dir(
        Path::new(&src_dir),
        Some("hello_world.mamba"),
        Some(bin_path.to_str().unwrap()),
        &arguments,
    )
    .map_err(|errs| format!("{errs:?}"))?;

    let output = Command::new(&produced).output()?;
    assert!(
        output.status.success(),
        "executable exited with an error:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8(output.stdout)?, "hello world\n");
    Ok(())
}

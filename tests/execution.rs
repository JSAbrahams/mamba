//! Runtime execution tests: unlike the AST-diff tests in `tests/check/valid.rs`
//! (`tests_util::test_directory`), which only check that generated Python is *structurally*
//! equivalent to a reference file, these actually run the compiled output and assert on what it
//! prints -- for both backends.

use std::error::Error;
use std::path::Path;

use test_case::test_case;
use tests_util::{resource_path, run_cli, run_via_bin, run_via_python, Runner};

#[test_case(run_via_python, &["function"], "hello_world.mamba" => "hello world\n")]
#[test_case(run_via_bin, &["function"], "hello_world.mamba" => "hello world\n")]
#[test_case(run_via_python, &["function"], "arithmetic_sum.mamba" => "14\n")]
#[test_case(run_via_bin, &["function"], "arithmetic_sum.mamba" => "14\n")]
#[test_case(run_via_python, &["function"], "for_loop_sum.mamba" => "10\n")]
#[test_case(run_via_bin, &["function"], "for_loop_sum.mamba" => "10\n")]
fn execution(run: Runner, dirs: &[&str], file: &str) -> String {
    run(dirs, file).unwrap()
}

#[test]
fn asm_prints_disassembly_to_stdout() -> Result<(), Box<dyn Error>> {
    let src_dir = resource_path(true, &["function"], "");

    let stdout = run_cli(Path::new(&src_dir), &["--asm", "-i", "hello_world.mamba"])?;
    assert!(
        stdout.contains("; -- main --") && stdout.contains("ret"),
        "expected disassembly on stdout, got:\n{stdout}"
    );

    // No output directory/file should have been written -- `--asm` only prints.
    assert!(!Path::new(&src_dir).join("target").exists());
    assert!(!Path::new(&src_dir).join("a.out").exists());
    Ok(())
}

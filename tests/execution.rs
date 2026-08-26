//! Runtime execution tests: unlike the AST-diff tests in `tests/check/valid.rs`
//! (`tests_util::test_directory`), which only check that generated Python is *structurally*
//! equivalent to a reference file, these actually run the compiled output and assert on what it
//! prints -- for both backends.

use std::error::Error;
use std::path::Path;

use test_case::test_case;
use tests_util::{resource_path, run_cli, run_via_asm, run_via_bin, run_via_python, Runner};

/// Fixtures whose output is identical across both backends -- the bulk of correctness coverage.
#[test_case(run_via_python, &["function"], "hello_world.mamba" => "hello world\n")]
#[test_case(run_via_bin, &["function"], "hello_world.mamba" => "hello world\n")]
#[test_case(run_via_python, &["function"], "arithmetic_sum.mamba" => "14\n")]
#[test_case(run_via_bin, &["function"], "arithmetic_sum.mamba" => "14\n")]
#[test_case(run_via_python, &["function"], "arithmetic_ops.mamba" => "6\n40\n1\n")]
#[test_case(run_via_bin, &["function"], "arithmetic_ops.mamba" => "6\n40\n1\n")]
#[test_case(run_via_python, &["function"], "float_arithmetic.mamba" => "1\n")]
#[test_case(run_via_bin, &["function"], "float_arithmetic.mamba" => "1\n")]
#[test_case(run_via_python, &["function"], "comparison_int.mamba" => "0\n0\n1\n1\n0\n")]
#[test_case(run_via_bin, &["function"], "comparison_int.mamba" => "0\n0\n1\n1\n0\n")]
#[test_case(run_via_python, &["function"], "comparison_bool.mamba" => "0\n1\n")]
#[test_case(run_via_bin, &["function"], "comparison_bool.mamba" => "0\n1\n")]
#[test_case(run_via_python, &["function"], "if_no_else_stmt.mamba" => "5\n")]
#[test_case(run_via_bin, &["function"], "if_no_else_stmt.mamba" => "5\n")]
#[test_case(run_via_python, &["function"], "void_function_call_stmt.mamba" => "42\n")]
#[test_case(run_via_bin, &["function"], "void_function_call_stmt.mamba" => "42\n")]
#[test_case(run_via_python, &["function"], "for_loop_sum.mamba" => "10\n")]
#[test_case(run_via_bin, &["function"], "for_loop_sum.mamba" => "10\n")]
#[test_case(run_via_python, &["function"], "for_loop_exclusive_range.mamba" => "10\n")]
#[test_case(run_via_bin, &["function"], "for_loop_exclusive_range.mamba" => "10\n")]
#[test_case(run_via_python, &["function"], "for_loop_shadow.mamba" => "6\n3\n")]
#[test_case(run_via_bin, &["function"], "for_loop_shadow.mamba" => "6\n3\n")]
fn execution(run: Runner, dirs: &[&str], file: &str) -> String {
    run(dirs, file).unwrap()
}

/// Fixtures that only run through one specific backend: either the two backends' output
/// legitimately diverges (e.g. the Cranelift backend prints a `Bool` as `1`/`0` via `printf`,
/// where the Python backend prints `True`/`False`), or -- for `if_else_tail.mamba` and
/// `implicit_last_expr_return.mamba` -- the Python backend has a real, pre-existing bug
/// unrelated to the Cranelift backend under test here: without `--annotate` (off by default),
/// it fails to emit a `return` for a function whose body is an implicit last-expression (no
/// `return` keyword), so the function silently returns `None` instead. `run_via_python` uses
/// `Arguments::default()` (`annotate: false`), so pairing these against it would just be
/// asserting on that separate, known-bad behavior.
#[test_case(run_via_bin, &["function"], "print_bool.mamba" => "1\n0\n")]
#[test_case(run_via_bin, &["function"], "if_else_tail.mamba" => "1\n-1\n")]
#[test_case(run_via_bin, &["function"], "implicit_last_expr_return.mamba" => "36\n")]
#[test_case(run_via_bin, &["function"], "float_var_decl.mamba" => "1\n")]
fn bin_only_execution(run: Runner, dirs: &[&str], file: &str) -> String {
    run(dirs, file).unwrap()
}

/// Mamba constructs that are valid (the Python backend handles all of these) but fall outside
/// this backend's supported subset -- each should fail with a clear, specific error rather than
/// panicking or producing silently wrong output.
#[test_case("float_print_unsupported.mamba", "print of a Float value")]
#[test_case("floordiv_unsupported.mamba", "FDiv")]
#[test_case("compound_reassign_unsupported.mamba", "compound reassignment")]
#[test_case("for_over_list_unsupported.mamba", "for-loop collection")]
#[test_case(
    "nullable_arg_unsupported.mamba",
    "not supported by the machine-code backend"
)]
#[test_case(
    "generic_arg_unsupported.mamba",
    "not supported by the machine-code backend"
)]
#[test_case(
    "str_arg_unsupported.mamba",
    "not supported by the machine-code backend"
)]
#[test_case("print_zero_args_unsupported.mamba", "!= 1 argument")]
#[test_case("print_two_args_unsupported.mamba", "!= 1 argument")]
#[test_case("print_interpolated_unsupported.mamba", "interpolated string")]
#[test_case("no_initializer_unsupported.mamba", "variable definition")]
fn bin_backend_rejects_gracefully(file: &str, expected_substring: &str) {
    let err = run_via_bin(&["function"], file)
        .expect_err("this fixture is deliberately outside the Cranelift backend's support");
    assert!(
        err.to_string().contains(expected_substring),
        "expected an error containing {expected_substring:?}, got: {err}"
    );
}

/// Drives `--asm`'s whole pipeline (`print_asm` -> `mamba_to_asm` -> `disassemble` ->
/// `build_isa`) in-process, for both the default (host) and an explicit target triple -- see
/// [run_via_asm]'s doc comment for why this needs to be in-process rather than via [run_cli].
#[test_case(None)]
#[test_case(Some("x86_64-unknown-linux-gnu"))]
fn asm_backend_disassembles_in_process(triple: Option<&str>) {
    run_via_asm(&["function"], "hello_world.mamba", triple)
        .unwrap_or_else(|err| panic!("expected disassembly to succeed, got: {err}"));
}

/// `not-a-real-triple` isn't parseable as a target triple at all; `sparc-unknown-none-elf`
/// parses fine but isn't an ISA Cranelift implements -- two different rejection points in
/// `build_isa`, both exercised here.
#[test_case("not-a-real-triple", "Invalid target")]
#[test_case("sparc-unknown-none-elf", "Unsupported target")]
fn asm_backend_rejects_bad_target_triple(triple: &str, expected_substring: &str) {
    let err = run_via_asm(&["function"], "hello_world.mamba", Some(triple))
        .expect_err("not a target Cranelift can compile for");
    assert!(
        err.to_string().contains(expected_substring),
        "expected an error containing {expected_substring:?}, got: {err}"
    );
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

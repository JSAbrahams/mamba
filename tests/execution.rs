//! Runtime execution tests: unlike the AST-diff tests in `tests/check/valid.rs`
//! (`tests_util::test_directory`), which only check that generated Python is *structurally*
//! equivalent to a reference file, these actually run the compiled output and assert on what it
//! prints -- for both backends.
//!
//! What each fixture should print is in the `<name>.stdout` file beside it, see `tests_util::expected_stdout`.

use mamba::{compile, Options, Output, Source, Stage};
use test_case::{test_case, test_matrix};
use tests_util::{
    assert_prints_expected, resource_path, run_via_asm, run_via_bin, run_via_python, Runner,
};

/// Fixtures whose output is identical across both backends -- the bulk of correctness coverage.
/// We use test matrix to emphaize that output should be the same for python and binary.
#[test_matrix([run_via_python, run_via_bin], &["function"], "hello_world.mamba")]
#[test_matrix([run_via_python, run_via_bin], &["function"], "arithmetic_sum.mamba")]
#[test_matrix([run_via_python, run_via_bin], &["function"], "arithmetic_ops.mamba")]
#[test_matrix([run_via_python, run_via_bin], &["function"], "float_arithmetic.mamba")]
#[test_matrix([run_via_python, run_via_bin], &["function"], "comparison_int.mamba")]
#[test_matrix([run_via_python, run_via_bin], &["function"], "comparison_bool.mamba")]
#[test_matrix([run_via_python, run_via_bin], &["function"], "if_no_else_stmt.mamba")]
#[test_matrix([run_via_python, run_via_bin], &["function"], "void_function_call_stmt.mamba")]
#[test_matrix([run_via_python, run_via_bin], &["function"], "for_loop_sum.mamba")]
#[test_matrix([run_via_python, run_via_bin], &["function"], "for_loop_exclusive_range.mamba")]
#[test_matrix([run_via_python, run_via_bin], &["function"], "for_loop_shadow.mamba")]
#[test_matrix([run_via_python, run_via_bin], &["function"], "let_in_if_shadow.mamba")]
#[test_matrix([run_via_python, run_via_bin], &["function"], "let_in_loop_shadow.mamba")]
#[test_matrix([run_via_python, run_via_bin], &["function"], "neq_int.mamba")]
#[test_matrix([run_via_python, run_via_bin], &["function"], "neq_float.mamba")]
#[test_matrix([run_via_python, run_via_bin], &["function"], "float_comparison.mamba")]
#[test_matrix([run_via_python, run_via_bin], &["function"], "simple_reassign.mamba")]
#[test_matrix([run_via_python, run_via_bin], &["function"], "forward_ref.mamba")]
#[test_matrix([run_via_python, run_via_bin], &["function"], "void_implicit_body.mamba")]
#[test_matrix([run_via_python, run_via_bin], &["function"], "recursion_factorial.mamba")]
#[test_matrix([run_via_python, run_via_bin], &["function"], "early_return.mamba")]
#[test_matrix([run_via_python, run_via_bin], &["function"], "both_branches_return.mamba")]
#[test_matrix([run_via_python, run_via_bin], &["function"], "nested_for_sum.mamba")]
#[test_matrix([run_via_python, run_via_bin], &["function"], "if_else_tail.mamba")]
#[test_matrix([run_via_python, run_via_bin], &["function"], "implicit_last_expr_return.mamba")]
fn execution(run: Runner, dirs: &[&str], file: &str) {
    assert_prints_expected(run, dirs, file);
}

/// `Int` is unbounded, so both backends must print the same digits once a value outgrows a machine word.
/// The operands in the `bigint_add*` fixtures each exceed 2^64.
/// `bigint_boundary` crosses 2^62 in both directions, which is where the Cranelift backend moves a value to the heap and back.
#[test_matrix([run_via_python, run_via_bin], &["function"], "bigint_add.mamba")]
#[test_matrix([run_via_python, run_via_bin], &["function"], "bigint_add_carry.mamba")]
#[test_matrix([run_via_python, run_via_bin], &["function"], "bigint_add_negative.mamba")]
#[test_matrix([run_via_python, run_via_bin], &["function"], "bigint_sub.mamba")]
#[test_matrix([run_via_python, run_via_bin], &["function"], "bigint_mul.mamba")]
#[test_matrix([run_via_python, run_via_bin], &["function"], "bigint_boundary.mamba")]
#[test_matrix([run_via_python, run_via_bin], &["function"], "bigint_compare.mamba")]
#[test_matrix([run_via_python, run_via_bin], &["function"], "bigint_factorial.mamba")]
#[test_matrix([run_via_python, run_via_bin], &["function"], "bigint_for_loop.mamba")]
#[test_matrix([run_via_python, run_via_bin], &["function"], "bigint_function.mamba")]
#[test_matrix([run_via_python, run_via_bin], &["function"], "bigint_literal.mamba")]
#[test_matrix([run_via_python, run_via_bin], &["function"], "bigint_to_float.mamba")]
#[test_matrix([run_via_python, run_via_bin], &["function"], "bigint_float_compare.mamba")]
fn unbounded_int_execution(run: Runner, dirs: &[&str], file: &str) {
    assert_prints_expected(run, dirs, file);
}

/// A `Float` cannot hold every `Int`, and 2^1100 is past its range.
/// Both backends stop with an error, rather than carry on with infinity.
#[test_case(run_via_python)]
#[test_case(run_via_bin)]
fn int_too_large_for_float(run: Runner) {
    run(&["function"], "bigint_float_overflow.mamba").expect_err("2^1100 does not fit a Float");
}

/// Fixtures that only run through one specific backend, because the two backends' output
/// legitimately diverges.
/// The Cranelift backend prints a `Bool` as `1`/`0`, where the Python backend prints
/// `True`/`False`.
#[test_case(run_via_bin, &["function"], "print_bool.mamba")]
#[test_case(run_via_bin, &["function"], "float_var_decl.mamba")]
fn bin_only_execution(run: Runner, dirs: &[&str], file: &str) {
    assert_prints_expected(run, dirs, file);
}

/// Fixtures that only run through the Python backend, because they use language features
/// outside the Cranelift backend's supported subset. Tuples are not lowered to machine code
/// at all, so there is no second backend to compare against.
#[test_case(run_via_python, &["definition"], "tuple_nested.mamba")]
#[test_case(run_via_python, &["definition"], "identifier_sqrt.mamba")]
#[test_case(run_via_python, &["class"], "associated_function.mamba")]
#[test_case(run_via_python, &["class"], "self_type.mamba")]
#[test_case(run_via_python, &["class"], "derived_field.mamba")]
#[test_case(run_via_python, &["class"], "pure_new_with_impure_method.mamba")]
#[test_case(run_via_python, &["class"], "pure_new_written_out.mamba")]
#[test_case(run_via_python, &["class"], "pure_new_no_class_arguments.mamba")]
#[test_case(run_via_python, &["class"], "new_underscore_one_argument.mamba")]
#[test_case(run_via_python, &["class"], "new_rest_one_argument.mamba")]
#[test_case(run_via_python, &["class"], "redundant_new.mamba")]
#[test_case(run_via_python, &["class"], "parent_literal_argument.mamba")]
#[test_case(run_via_python, &["definition"], "tuple_element_mut_annotated.mamba")]
// Match patterns and guards. The AST diff in `tests/check/valid.rs` covers these too, since the
// move to `ruff_python_parser`, but these assert on what the output actually does when run.
#[test_case(run_via_python, &["control_flow"], "match_tuple_underscore.mamba")]
#[test_case(run_via_python, &["control_flow"], "match_exhaustive_bool.mamba")]
#[test_case(run_via_python, &["control_flow"], "match_exhaustive_nullable_bool.mamba")]
#[test_case(run_via_python, &["control_flow"], "match_exhaustive_tuple_of_bools.mamba")]
#[test_case(run_via_python, &["control_flow"], "match_guarded_arm_does_not_hide_next.mamba")]
#[test_case(run_via_python, &["control_flow"], "match_guard.mamba")]
#[test_case(run_via_python, &["control_flow"], "match_first_arm_wins.mamba")]
#[test_case(run_via_python, &["control_flow"], "match_guard_falls_through.mamba")]
#[test_case(run_via_python, &["control_flow"], "match_guard_calls_function.mamba")]
#[test_case(run_via_python, &["control_flow"], "match_guard_on_literal_pattern.mamba")]
#[test_case(run_via_python, &["control_flow"], "match_guard_on_wildcard.mamba")]
#[test_case(run_via_python, &["control_flow"], "match_guard_reads_outer.mamba")]
#[test_case(run_via_python, &["control_flow"], "match_compound_guard.mamba")]
#[test_case(run_via_python, &["control_flow"], "match_literal_str.mamba")]
#[test_case(run_via_python, &["control_flow"], "match_negative_int_literal.mamba")]
#[test_case(run_via_python, &["control_flow"], "match_nested_tuple_literals.mamba")]
#[test_case(run_via_python, &["control_flow"], "match_in_for_body.mamba")]
#[test_case(run_via_python, &["control_flow"], "match_shadow_restores_outer.mamba")]
#[test_case(run_via_python, &["control_flow"], "match_partial_pattern_restores_outer.mamba")]
#[test_case(run_via_python, &["control_flow"], "match_wildcards_on_different_types.mamba")]
#[test_case(run_via_python, &["control_flow"], "shadow_in_sibling_blocks.mamba")]
#[test_case(run_via_python, &["control_flow"], "match_single_irrefutable_arm.mamba")]
#[test_case(run_via_python, &["control_flow"], "match_wildcard_only.mamba")]
#[test_case(run_via_python, &["control_flow"], "match_subject_is_call.mamba")]
#[test_case(run_via_python, &["control_flow"], "match_arm_body_block.mamba")]
#[test_case(run_via_python, &["control_flow"], "match_guard_not_evaluated_when_pattern_fails.mamba")]
#[test_case(run_via_python, &["control_flow"], "match_tuple_pattern.mamba")]
#[test_case(run_via_python, &["control_flow"], "match_nested_tuple_pattern.mamba")]
#[test_case(run_via_python, &["control_flow"], "match_tuple_shadows_argument.mamba")]
fn python_only_execution(run: Runner, dirs: &[&str], file: &str) {
    assert_prints_expected(run, dirs, file);
}

/// Every example in the top-level `README.md`, actually run.
///
/// The AST-diff tests in `tests/check/valid.rs` only compare generated Python structurally.
/// They cannot tell that an example a reader copies out of the README does what the README says.
/// These do, which is the point: the README is the language's promise.
///
/// An example whose README block is only definitions gets a use added here, so there is output to
/// assert on rather than an empty string.
/// Where the block continues an earlier one, the fixture repeats that setup to stand alone.
///
/// An example that is *about* a construct which does not exist yet is ignored, with the reason.
/// Those reasons match the ones on the same fixture in `tests/check/valid.rs`.
/// Where only a line or two is unimplemented, that line is commented out in the fixture instead,
/// so the rest still runs and the intent is still on record.
#[test_case("ackermann")]
#[test_case("builtin_trait" => ignore["@ decorator syntax and meta trait/external impl for a built-in type not implemented"])]
#[test_case("class")]
#[test_case("class_associated_function")]
#[test_case("class_construction")]
#[test_case("class_derived_field")]
#[test_case("class_new_enforces")]
#[test_case("class_with_constants")]
#[test_case("error_handling")]
#[test_case("error_handling_as_expression")]
#[test_case("error_handling_desyntax_sugared" => ignore["a bare `match` on a fallible call is not checked as one, and a case cannot bind an error's constructor arguments yet"])]
#[test_case("error_handling_early_exit")]
#[test_case("error_handling_handle_subset" => ignore["explicit Result[...] type and re-raising via `<var> = <var> !` not implemented"])]
#[test_case("error_handling_result_type" => ignore["explicit Result[...] type not implemented, and a nested generic argument such as Union[...] does not parse"])]
#[test_case("factorial")]
#[test_case("factorial_dynamic")]
#[test_case("impl_trait" => ignore["`def <Trait> for <Class> where ...` external-implementation syntax not implemented"])]
#[test_case("list_shorthand")]
#[test_case("lists")]
#[test_case("mutability")]
#[test_case("pure_construction")]
#[test_case("pure_construction_methods")]
#[test_case("pure_functions")]
#[test_case("sets_maps")]
#[test_case("total_functions" => ignore["`Nat` type not implemented"])]
#[test_case("trait_inheritance" => ignore["composing multiple parent traits not implemented"])]
#[test_case("trait_meta" => ignore["`Measurable` trait not implemented"])]
#[test_case("traits" => ignore["generics on traits and external-implementation syntax not implemented"])]
fn readme_example_execution(file: &str) {
    assert_prints_expected(
        run_via_python,
        &["readme_example"],
        &format!("{file}.mamba"),
    );
}

/// Mamba constructs that are valid (the Python backend handles all of these) but fall outside
/// this backend's supported subset -- each should fail with a clear, specific error rather than
/// panicking or producing silently wrong output.
#[test_case("float_print_unsupported.mamba")]
#[test_case("floordiv_unsupported.mamba")]
#[test_case("compound_reassign_unsupported.mamba")]
#[test_case("for_over_list_unsupported.mamba")]
#[test_case("nullable_arg_unsupported.mamba")]
#[test_case("generic_arg_unsupported.mamba")]
#[test_case("str_arg_unsupported.mamba")]
#[test_case("print_zero_args_unsupported.mamba")]
#[test_case("print_two_args_unsupported.mamba")]
#[test_case("print_interpolated_unsupported.mamba")]
#[test_case("no_initializer_unsupported.mamba")]
#[test_case("and_unsupported.mamba")]
#[test_case("or_unsupported.mamba")]
#[test_case("not_unsupported.mamba")]
#[test_case("pow_unsupported.mamba")]
#[test_case("mod_unsupported.mamba")]
#[test_case("while_unsupported.mamba")]
fn bin_backend_rejects_gracefully(file: &str) {
    let text = std::fs::read_to_string(resource_path(true, &["function"], file)).unwrap();
    let options = Options::new(Output::Object { target: None });
    let errs = compile(&[Source::named(file, text)], &options)
        .expect_err("this fixture is deliberately outside the Cranelift backend's support");
    assert_eq!(errs[0].stage, Stage::Generate, "{errs:?}");
    assert!(errs[0].primary_label().is_some(), "{errs:?}");
}

/// Drives `--asm`'s whole pipeline (`compile` -> `disassemble` -> `build_isa`) in-process, for both the default (host) and an explicit target triple -- see
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

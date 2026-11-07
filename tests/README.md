# Test suite notes

Fixture layout and `test_case` conventions are in the root `CLAUDE.md`.
This file lists what is not obvious from the code: traps, coverage, and known gaps.
Each unimplemented feature also carries an `ignore[...]` reason at its `test_case`, which is the authoritative record.

## Kinds of test

- `tests/{parse,check}/{valid,invalid}.rs`: table-driven over `tests/resource/`.
  Valid check tests diff the transpiler's Python AST against a reference `.py`, via `ruff_python_parser`.
  They never run the output, and always transpile with `annotate: true`.
- `tests/execution.rs`: transpiles with both backends, runs the result, and asserts both print the same.
  What a fixture should print is in a `<name>.stdout` file beside its `<name>.mamba`, read by `tests_util::expected_stdout`.
  A new execution fixture needs that file, even if empty, or its test panics.
  It goes through the CLI's default path, without `--annotate`.
  Fixtures here must stay within the Cranelift subset, see `src/backend/cranelift/README.md`.
- `tests/main.rs`: black-box CLI tests.
  Some write to fixed paths under `tests/resource/valid/dummy/proj1/`, which can race under parallel runs.
  Use `-- --test-threads=1` if they flake.

## Traps when un-ignoring

- An `invalid` case must keep `=> matches Err(_)`.
  Without a matcher, `test_case` passes only on `Ok`, so the test goes green exactly when the checker fails to reject the program.
- An ignored `valid` case, `=> ignore["..."]`, has no expected value.
  `--run-ignored all` therefore passes it even if it returns `Err`.
  Check it by transpiling directly: `./target/debug/mamba -i <fixture>.mamba -o <out> -a`.
  Un-ignoring means deleting the whole `=> ignore[...]`.

## Coverage

Use the same exclusions as CI, from `.github/workflows/coverage.yml`:

```sh
cargo llvm-cov --package mamba --lcov --output-path target/lcov.info \
  --ignore-filename-regex '(^|/)tests?/.*|(^|/)tests_util/.*|.*_tests\.rs$'
```

The goal is 100% outside thin plumbing files: `main.rs`, `io.rs`, and the `result.rs` error-formatting modules.

Some checker branches are reached only through the content of the Python stubs in `src/check/resource/{primitive,std}`.
Cover those by adding a realistic stub signature, plus a fixture that uses it, as `divmod` does.
Stubs must parse as Python.
`std/builtins.py` still declares `__debug__: bool`, which `ruff` accepts but CPython rejects.

## Known gaps

Unfinished features whose code exists but is not wired in:

- `Function::args_compatible`: callable arity is never checked, so the arity arm in `check/constrain/unify/function.rs` is unreachable.
- `Context::field`: top-level fields from stubs, such as `__debug__`, are parsed but never looked up.

Checker:

- Multi-variable builders, `[(x, y) | x in a, y in b]`, only bind the first variable.
- Passing `None` as an argument of a nullable type is rejected, see `definition/none_to_nullable_argument`.
- The type of a tuple built from a nullable value loses its nullability.
  A match on it then accepts a missing `None`, see `match_tuple_variable_missing_none`.
- Two fields whose initializers reference each other check and generate, see `class/cyclic_field_dependency`.
- `total` and `meta` are parsed but not checked.

Generator:

- A top-level function named `size` becomes `__size__`, but its call sites do not.
- With `--annotate`, a method annotated with its own class emits a forward reference Python rejects.

Fixtures:

- `invalid/type/control_flow/different_type_shadow.mamba` is a byte-identical copy of `undefined_var_in_match_arm.mamba`.

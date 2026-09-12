# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project overview

Mamba is a statically-typed, Python-like programming language.
It is implemented here as a **transpiler written in Rust**, which converts `.mamba` source files into `.py` (Python 3) source files.
The crate is both a library (`src/lib.rs`) and a binary (`src/main.rs`, the `mamba` CLI).

## Commands

The dev environment is [Devbox](https://www.jetify.com/devbox), configured by `devbox.json`.
Versions are pinned in `devbox.lock`.
Devbox is a layer over Nix.
It supplies the Rust toolchain, Python 3.10, and the cargo helpers the hooks and CI call.
`devbox shell` enters the environment.
`devbox run -- <cmd>` runs a single command in it.
The cargo commands below assume you are inside that environment.
`devbox.json` also defines named scripts, such as `devbox run lint` and `devbox run precommit`.
See CONTRIBUTING.md for those.

```sh
cargo build                       # build the transpiler
cargo run -- -i <input> -o <out>  # run the CLI directly (see src/cli.rs for all flags)
cargo test --package mamba        # run the full test suite
cargo fmt --all -- --check        # check formatting (CI enforces this)
cargo clippy --all-features -- -D warnings  # lint (CI enforces this, treats warnings as errors)
```

CI runs the suite with [nextest](https://nexte.st/) rather than `cargo test`.
To reproduce a CI test run exactly:

```sh
cargo nextest run --package mamba --config-file .config/nextest.toml --profile ci
```

Note that CI's clippy step does *not* pass `--tests`.
Lints that only fire inside `#[cfg(test)]` code are therefore not enforced.

Running the test suite requires a `python3` on `PATH`.
That is `python3.10` on Linux, `python3` on macOS, and `python` on Windows.
See `tests_util/src/lib.rs` for the exact names.
The requirement exists because generated Python output is validated with `python -m py_compile`.
Devbox pins `python@3.10.18` for this.
If you change the version there, change `tests_util::PYTHON` to match.

To run a single test:

```sh
cargo test --package mamba <test_name>            # e.g. cargo test transpile_src_in_dir
cargo test --package mamba -- --list               # list all test names, including test_case-expanded ones
```

Most `check`/`parse` behavior tests are table-driven via `#[test_case(...)]` macros.
These live in `tests/check/valid.rs`, `tests/check/invalid.rs`, `tests/parse/valid.rs`, and `tests/parse/invalid.rs`.
The generated test name is derived from the macro arguments.
For example, `#[test_case("class", "generics")]` produces a test whose name embeds `class_generics`.
Use `cargo test -- --list` to find the exact generated name before running one directly.

### Git hooks

Hooks live in `.githooks/` (not `.git/hooks/`) and are opt-in:

```sh
git config core.hooksPath .githooks
```

The `pre-commit` hook runs several checks.
These are `cargo fmt --check`, `cargo check --tests --all` in debug and release, `cargo sort --check`, `cargo clippy -D warnings` in debug and release with all features, and `cargo check --benches`.
`cargo sort --check` is there because Cargo.toml dependencies must stay alphabetically sorted.
The `commit-msg` hook enforces Conventional-Commits-style subject lines.
The form is `<type>: <summary>`.
`<type>` is one of `doc test feat fix style refactor revert git chore perf build ci cd deploy security`.
The subject must be under 50 chars and must not end in a period.
Body lines are wrapped at 72 chars.
Match this style even without the hook installed.
See recent `git log` output for examples.

## Architecture

The pipeline for a single file is: `String` → **parse** → `AST` → **check** → typed AST (`ASTTy`) → **backend**.
The default Python backend then goes `ASTTy` → `PythonCore` → `String` (Python).
The alternative Cranelift backend lowers `ASTTy` straight to machine code.
Each stage lives in its own module under `src/`.
Each also has its own README, worth reading before making non-trivial changes there.
Those are `src/parse/README.md`, `src/check/README.md`, `src/backend/python/README.md`, and `src/backend/cranelift/README.md`.

- **`src/parse`**: `lex/` tokenizes source into a `Vec<Token>`.
  The parser walks those tokens via a `TokenIterator` and builds an `AST` (`parse/ast`).
  Each AST node carries a `Position`, used for error reporting.
  The parser is split across `expression.rs`, `statement.rs`, `class.rs`, `definition.rs`, `control_flow_*.rs`, `collection.rs`, `call.rs`, `operation.rs`, `ty.rs` and others.

- **`src/check`**: the type checker, and where most language semantics live.
  It has three phases:
  1. **Context building** (`check/context`): scans all ASTs (including implicit/explicit imports and built-in
     Python primitives/stdlib shims under `check/resource/primitive` and `check/resource/std`, which are `.py`
     files with Mamba-visible signatures) into a `Context` of function/class signatures.
     This does not type check bodies.
     It only validates that signatures are well-formed.
  2. **Constraint generation** (`check/constrain`): walks the AST, plus an environment and the `Context`, to produce a list of type constraints.
     The environment tracks variable definition and scope.
     The `Context` tracks class and function existence and signatures.
     The constraints note expected supertype relationships.
     For example, an assignment's annotation must be a supertype of the RHS expression.
  3. **Substitution and unification**: unifies constrained types by comparing `Name`s and checking supertype relationships.
     It produces a `Position` → `Name` mapping.
     That mapping is then used to annotate the AST into `ASTTy` (`check/ast`).

Type identity is centered on `check/name`.
A `Name` is a set of `TrueName`s, which is a type union.
A `TrueName` wraps nullability and mutability flags around a `NameVariant`.
A `NameVariant` is one of `StringName` (a nominal type), `Tuple`, or `Function` (arguments plus a return `Name`).

- **`src/backend`**: `mod.rs` defines the `Backend` enum selecting which of the two backends the pipeline targets (`Python`, the default, or `Bin`/`Asm` via Cranelift).

  - **`src/backend/python`**: converts `ASTTy` into `PythonCore`, a simplified near-Python IR that tracks blocks and indentation.
    It desugars language constructs that have no 1:1 Python equivalent.
    It also tracks which imports the output needs, such as `from typing import Tuple`.
    `PythonCore` is then rendered directly to a Python string.
    Errors here generally indicate an unimplemented language construct, or a type-checker bug that let through an AST shape the generator was not expecting.
    `mamba_to_python`, which takes source strings to Python strings, lives here in `backend/python/mod.rs`.

  - **`src/backend/cranelift`**: lowers a checked `ASTTy` directly to native machine code, with no
    intermediate tree, for the `--bin` and `--asm` CLI flags. Only a small subset of the language is
    supported (`Int`/`Bool`/`Float`, arithmetic/comparison operators, `if`/`else`, `Int` ranges in `for`,
    top-level functions, and `print`); anything else errors with `BackendErr::unimplemented`. See its README
    for the exact subset.

- **`src/common`**: shared types used across all stages, notably `Position` (source spans, used for error messages) and `WithSource`/error-formatting helpers.

- **`src/lib.rs`**: wires the stages together via `transpile_dir`.
  That walks an input directory of `.mamba` files, and mirrors the structure into an output directory of `.py` files.
  `src/cli.rs` defines the `clap` CLI surface.
  `src/io.rs` holds the source read/write and directory-walking helpers.
  `src/main.rs` is the thin binary entrypoint.

### Tests

- `tests/parse/{valid,invalid}.rs` and `tests/check/{valid,invalid}.rs`: table-driven tests over fixtures in `tests/resource/{valid,invalid}/<category>/<name>.mamba`.
  Valid check tests also take a `<name>.py` reference file.
  See `tests_util::fallable`, which diffs the transpiler's Python AST against the reference file's Python AST, not raw text.
  `valid` and `invalid` fixtures are separate directory trees under `tests/resource/`.
- `tests/main.rs`: black-box CLI tests, which invoke the built binary via `assert_cmd`.
  These cover input/output directory resolution and error paths.
- `tests/execution.rs`: runtime tests that actually *run* the output and assert on what it prints, for both backends.
  The AST-diff tests above only compare generated Python structurally, so this is the stronger check.
  Most fixtures go through a `test_matrix` over `run_via_python` and `run_via_bin`, asserting both backends print the same thing.
  `run_via_asm` covers the `--asm` path.
  This file requires a working `cc` for the `--bin` link step.
- `tests_util` is a separate crate, and a path-dependency of `mamba`'s dev-dependencies.
  It holds shared test helpers: fixture path resolution, randomized temp output dirs, the Python-AST-diff assertion logic used by `test_directory` and `test_directory_args`, and the `run_via_*` backend runners used by `tests/execution.rs`.
- `tests/README.md`: coverage-archaeology notes.
  It covers confirmed dead code, which checker branches are gated by Python-stub-file content rather than `.mamba` fixtures, known type-checker gaps found while chasing coverage, and a flaky test.
  Read it before assuming an uncovered line is just a missing test.

## Block syntax (post indent/dedent removal)

`for`/`while`/`using` bodies always require an explicit `do ... end` block.
There is no single-statement shorthand for these three.
So `for a in b do c` is a parse error, and it must be `for a in b do c end`.
`if`/`then`/`else` branches are the exception.
Each branch is parsed as one `parse_expr_or_stmt`.
That accepts either a bare single statement or expression, or an explicit `do ... end` block.
So `if a then do ... end else c` is valid.
A leading newline before a statement or expression is insignificant whitespace, and is skipped.
See the `eat_while(&Token::NL)` calls in `parse_expression` and `parse_expr_or_stmt`.
A *trailing* newline is still usually required as a statement separator inside a block.
Some constructs need to look past that newline for an optional following keyword.
An example is `parse_if` scanning past newlines for a possible `else`.
These must use a lookahead-with-rollback helper, `LexIterator::peek_if_skipping`, rather than unconditionally consuming the newline.
Otherwise they break the case of no `else` followed by more statements in the same block.

The call-site "handle" construct for a call that may raise is `<expr> ! where <case> ... end`.
An example is `f(10) ! where err: MyErr => do ... end end`.
The `!` marks the call as fallible.
It must be consumed before looking for `where`.
See `parse_expr_or_stmt` in `expr_or_stmt.rs`.

### Class arguments

Class constructor arguments are always fields, stored on `self`.
They take no `def` prefix, so it is `class X(a: Int)` and not `class X(def a: Int)`.
`parse_class` in `src/parse/class.rs` rejects the latter.
Inside the class body they must be accessed via `self.a`, never bare `a`.
This is because `self` is bound, and typed as the class, while checking a class body.
See `gen_class` in `src/check/constrain/generate/class.rs`.
It works the same way a method's own `self` argument does.

A class body runs once per instance, like a constructor.
It does not run once at class-definition time, the way a real Python class body does.
So `hoist_constructor_dependent_stmts` in `src/backend/python/convert/class.rs` moves anything that is not a field or method declaration into a generated `__init__`.
That covers a field initializer referencing `self`, which keeps its class-level slot with `None` in place of the real value.
It also covers any other bare statement, such as `print(self.a)`.
This happens unconditionally, whether or not the statement references `self`.
A docstring is the one exception, and must stay a literal first statement in the class body.
Hoisted statements are then ordered by dependency, via `order_by_self_field_deps`, not just by declaration order.
This matters because a field can read another hoisted field declared later in the body, which would otherwise still be `None` at that point.
`get_fields_and_functions` in `src/check/context/clss/generic.rs` must likewise treat a bare statement as "not part of the signature", rather than rejecting it.
That function runs during context building, before any of this.

## Documentation

`docs/` contains the language specification and philosophy docs, including the formal grammar at `docs/spec/grammar.md`.
Those docs are partially outdated, as their own README says.
The top-level `README.md` has a larger set of annotated Mamba code examples, covering functions, collections, classes and error handling.
Check those for concrete syntax before assuming behavior from the grammar spec alone.

## Contribution conventions

- PRs target `develop`, not `main`/`master` (see `CONTRIBUTING.md`); `main` is release-only.
- Keep `Cargo.toml` dependencies alphabetically sorted (`cargo sort`).

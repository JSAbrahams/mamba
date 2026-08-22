# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project overview

Mamba is a statically-typed, Python-like programming language, implemented here as a **transpiler written in
Rust** that converts `.mamba` source files into `.py` (Python 3) source files. The crate is both a library
(`src/lib.rs`) and a binary (`src/main.rs`, the `mamba` CLI).

## Commands

```sh
cargo build                       # build the transpiler
cargo run -- -i <input> -o <out>  # run the CLI directly (see src/cli.rs for all flags)
cargo test --package mamba        # run the full test suite (matches CI)
cargo fmt --all -- --check        # check formatting (CI enforces this)
cargo clippy -- -D warnings       # lint (CI enforces this, treats warnings as errors)
```

Running the test suite requires a `python3` on `PATH` (`python3.10` on Linux, `python3` on macOS, `python` on
Windows — see `tests_util/src/lib.rs`), since generated Python output is validated with `python -m py_compile`.

To run a single test:

```sh
cargo test --package mamba <test_name>            # e.g. cargo test transpile_src_in_dir
cargo test --package mamba -- --list               # list all test names, including test_case-expanded ones
```

Most `check`/`parse` behavior tests are table-driven via `#[test_case(...)]` macros in `tests/check/valid.rs`,
`tests/check/invalid.rs`, `tests/parse/valid.rs`, `tests/parse/invalid.rs` — the generated test name is derived
from the macro arguments (e.g. `#[test_case("class", "generics")]` → a test whose name embeds `class_generics`).
Use `cargo test -- --list` to find the exact generated name before running one directly.

### Git hooks

Hooks live in `.githooks/` (not `.git/hooks/`) and are opt-in:

```sh
git config core.hooksPath .githooks
```

The `pre-commit` hook runs `cargo fmt --check`, `cargo check --tests --all` (debug + release), `cargo sort`
(Cargo.toml dependencies must stay alphabetically sorted), `cargo clippy -D warnings` (debug + release,
all-features), and `cargo check --benches`. The `commit-msg` hook enforces Conventional-Commits-style subject
lines: `<type>: <summary>` where `<type>` is one of `doc test feat fix style refactor revert git chore perf
build ci cd deploy security`, the subject is under 50 chars and doesn't end in a period, and body lines are
wrapped at 72 chars. Match this style even without the hook installed (see recent `git log` for examples).

## Architecture

The pipeline for a single file is: `String` → **parse** → `AST` → **check** → typed AST (`ASTTy`) →
**generate** → `Core` → `String` (Python). Each stage lives in its own top-level module under `src/`, and each
has its own README worth reading before making non-trivial changes there (`src/parse/README.md`,
`src/check/README.md`, `src/generate/README.md`).

- **`src/parse`**: `lex/` tokenizes source into a `Vec<Token>`, handling indentation by emitting synthetic
  `Indent`/`Dedent` tokens that the parser uses to delimit blocks. The parser (`expression.rs`, `statement.rs`,
  `class.rs`, `definition.rs`, `control_flow_*.rs`, `collection.rs`, `call.rs`, `operation.rs`, `ty.rs`, etc.)
  walks tokens via a `TokenIterator` and builds an `AST` (`parse/ast`), where each node carries a `Position` for
  error reporting. Errors here mean the token stream doesn't conform to the grammar.

- **`src/check`**: the type checker, and where most language semantics live. Three phases:
  1. **Context building** (`check/context`): scans all ASTs (including implicit/explicit imports and built-in
     Python primitives/stdlib shims under `check/resource/primitive` and `check/resource/std`, which are `.py`
     files with Mamba-visible signatures) into a `Context` of function/class signatures — this does not type
     check bodies, only validates signatures are well-formed.
  2. **Constraint generation** (`check/constrain`): walks the AST plus an environment (tracks variable
     definition/scope) and the `Context` (tracks class/function existence and signatures) to produce a list of
     type constraints, noting expected supertype relationships (e.g. an assignment's annotation must be a
     supertype of the RHS expression).
  3. **Substitution & unification**: unifies constrained types by comparing `Name`s, checking supertype
     relationships, and produces a `Position` → `Name` mapping, which is then used to annotate the AST into
     `ASTTy` (`check/ast`).
  Type identity is centered on `check/name`: a `Name` is a set of `TrueName`s (a type union); a `TrueName` wraps
  nullability + mutability flags around a `NameVariant`, which is one of `StringName` (nominal type),
  `Tuple`, or `Function` (args + return `Name`).

- **`src/generate`**: converts `ASTTy` into `Core` (a simplified, near-Python IR that tracks blocks/indentation),
  desugaring language constructs that have no 1:1 Python equivalent and tracking which imports the output needs
  (e.g. `from typing import Tuple`). `Core` is then rendered directly to a Python string. Errors here generally
  indicate either an unimplemented language construct or a type-checker bug (an AST shape the generator wasn't
  expecting).

- **`src/common`**: shared types used across all stages, notably `Position` (source spans, used for error
  messages) and `WithSource`/error-formatting helpers.

- **`src/lib.rs`**: wires the three stages together via `mamba_to_python` (source strings → Python strings) and
  `transpile_dir` (walks an input directory of `.mamba` files, mirrors the structure into an output directory of
  `.py` files). `src/cli.rs` defines the `clap` CLI surface; `src/main.rs` is the thin binary entrypoint.

### Tests

- `tests/parse/{valid,invalid}.rs` and `tests/check/{valid,invalid}.rs`: table-driven tests over fixtures in
  `tests/resource/{valid,invalid}/<category>/<name>.mamba` (+ a `<name>_check.py` reference file for valid
  check tests — see `tests_util::fallable`, which diffs the transpiler's Python AST against the reference file's
  Python AST, not raw text). `valid`/`invalid` fixtures are separate directory trees under `tests/resource/`.
- `tests/main.rs`: black-box CLI tests (invokes the built binary via `assert_cmd`) covering input/output
  directory resolution and error paths.
- `tests_util` (a separate crate, path-dependency of `mamba`'s dev-dependencies) holds shared test helpers:
  fixture path resolution, randomized temp output dirs, and the Python-AST-diff assertion logic used by
  `test_directory`/`test_directory_args`.
- Adding a new valid `check` fixture generally means adding both a `.mamba` file and a matching `_check.py`
  reference under `tests/resource/valid/<category>/`, then a `#[test_case(...)]` entry.

## Documentation

`docs/` contains the (partially outdated, per its own README) language specification and philosophy docs,
including the formal grammar at `docs/spec/grammar.md`. The top-level `README.md` has a larger set of annotated
Mamba code examples (functions, collections, classes, error handling, etc.) worth checking for concrete syntax
before assuming behavior from the grammar spec alone.

## Contribution conventions

- PRs target `develop`, not `main`/`master` (see `CONTRIBUTING.md`); `main` is release-only.
- Keep `Cargo.toml` dependencies alphabetically sorted (`cargo sort`).

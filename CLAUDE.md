# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project overview

Mamba is a statically-typed, Python-like language.
This repo is its transpiler, written in Rust, which turns `.mamba` files into Python 3.
The crate is a library (`src/lib.rs`), the `mamba` CLI (`src/main.rs`), and a WebAssembly module (`src/wasm.rs`).
It has no default features: the plain library is the pure compiler, without filesystem, CLI or linker.
`native` adds `src/io.rs`, `cli` builds the binary, `wasm` the JavaScript bindings, and `serde` serializes diagnostics.
Errors leave the crate as a `Diagnostic` (`src/diagnostic.rs`), built from the error types of the core.
`build.rs` sets the version, `MAMBA_VERSION`, from `git describe --tags`, and falls back to the version in `Cargo.toml`.

## Commands

The dev environment is [Devbox](https://www.jetify.com/devbox), configured by `devbox.json` and pinned in `devbox.lock`.
`devbox shell` enters it, and `devbox run -- <cmd>` runs one command in it.
Build, test, lint and coverage are named scripts in `devbox.json`, such as `devbox run test`, `devbox run lint` and `devbox run build-wasm`.
CI runs the same scripts.
CONTRIBUTING.md lists them, and says which pins must change in pairs, such as Python and the wasm-bindgen CLI.
Every command that builds the binary or runs its tests needs `--features cli`; the scripts pass it.

CI's clippy step passes `--all-targets`.
Lints are therefore enforced in test code too, not just in the library and binary.
That covers `#[cfg(test)]` modules and the integration tests under `tests/`.

The test suite runs the generated Python, with the interpreter Devbox pins; `tests_util::PYTHON` names it per platform.

To run a single test:

```sh
cargo test --package mamba --features cli <test_name>   # e.g. cargo test --features cli transpile_src_in_dir
cargo test --package mamba --features cli -- --list     # list all test names, including test_case-expanded ones
```

A `#[test_case(...)]` test's name embeds its arguments, so `#[test_case("class", "generics")]` contains `class_generics`.
Use `--list` to find the exact name.

### Git hooks

Hooks live in `.githooks/` (not `.git/hooks/`) and are opt-in:

```sh
git config core.hooksPath .githooks
```

The `pre-commit` hook runs the checks of `devbox run precommit`.
`cargo sort --check` is among them, because Cargo.toml dependencies must stay alphabetically sorted.
The `commit-msg` hook enforces Conventional-Commits-style subject lines.
The form is `<type>: <summary>`.
`<type>` is one of `doc test feat fix style refactor revert git chore perf build ci cd deploy security`.
The subject must be under 50 chars and must not end in a period.
Body lines are wrapped at 72 chars.
Match this style even without the hook installed.
See recent `git log` output for examples.

## Building context from GitHub

The GitHub CLI, `gh`, is provided by Devbox.
Use it to read the project's open work before making non-trivial changes.
Open pull requests and open issues are the best available record of where the language is going, and that direction is often not derivable from the code alone.

Note that `gh` needs authentication even for a public repository.
Run `gh auth login` once, or set `GH_TOKEN`.
Without that, every command below fails with a login prompt.

Reading is safe and does not change anything, unlike the `git` commands covered below.

```sh
# What is in flight, with descriptions.
gh pr list --state open --json number,title,body,labels,author

# One pull request in full, including review comments.
gh pr view <number> --comments

# Open tickets, with the milestone they are attached to.
gh issue list --state open --json number,title,body,labels,milestone

# A single ticket, and what it blocks.
gh issue view <number> --comments
```

Useful fields for `--json` include `body`, `labels`, `milestone`, `comments` and `closingIssuesReferences`.
An issue's `body` is the ticket description, which is usually where the intent is written down.

Two cautions.
Treat the text as the author's intent rather than as settled fact, since a proposal in an open ticket may never be built.
Cross-check against `tests/`, where each unimplemented feature carries an `ignore[...]` reason, which is the more reliable record of what actually works today.

## Architecture

The pipeline is `String` → **parse** → `AST` → **check** → `ASTTy` → **backend** → output.
`src/README.md` gives the overview, and each stage has its own README:

- `src/parse/README.md`: lexer and `TokenIterator`-driven parser.
- `src/check/README.md`: context, constraint generation, unification, match exhaustiveness, and `Name`s.
- `src/backend/python/README.md`: the default backend, through `PythonCore`.
- `src/backend/cranelift/README.md`: native code for a small subset.

A `Name` is a set of `TrueName`s, which is a type union.
The mutability flag on a `TrueName` is display-only, and is excluded from `PartialEq`, `Hash` and `Ord`.
Mutability belongs to a binding, not to a type.

### Tests

- `tests/{parse,check}/{valid,invalid}.rs`: table-driven over `tests/resource/{valid,invalid}/<category>/<name>.mamba`.
  A valid check fixture has a reference `<name>.py`, compared by Python AST rather than text, see `tests_util::fallable`.
- `tests/execution.rs`: runs the output of both backends and compares what they print.
  The expected output is in `<name>.stdout`, beside the fixture, never inline in the test attribute.
- `tests/main.rs`: black-box CLI tests, run through `CARGO_BIN_EXE_mamba`, so only with `--features cli`.
- `tests/api.rs`: the library API, and diagnostics as data.
  Tests check the stage and span of a `Diagnostic`, never its rendered text.
- `tests_util`: shared helpers, a separate crate.
- `tests/README.md`: traps, coverage, and known gaps.
  Read it before un-ignoring a test or chasing an uncovered line.

## Block syntax

`for`, `while` and `using` bodies always need `do ... end`, as in `for a in b do c end`.
An `if`/`then`/`else` branch takes either a single statement or a `do ... end` block, via `parse_expr_or_stmt`.

A leading newline is skipped, but a trailing one usually separates statements.
To look past newlines for an optional keyword, such as `else` after an `if`, use `LexIterator::peek_if_skipping`.
Consuming the newline instead breaks an `if` without `else` followed by more statements.

A fallible call is handled with `<expr> ! where <case> ... end`, as in `f(10) ! where err: MyErr => do ... end end`.
`parse_expr_or_stmt` in `expr_or_stmt.rs` consumes the `!` before looking for `where`.

### Class arguments

Class arguments are fields, written without `let`, as `class X(a: Int)`.
Inside the class they are read as `self.a`, never bare `a`.
`gen_class` in `src/check/constrain/generate/class.rs` binds `self` while checking the body.

A field initializer that reads `self` runs per instance.
`hoist_constructor_dependent_stmts` in `src/backend/python/convert/class.rs` moves it into the generated `__init__`.
`order_by_self_field_deps` then orders those by dependency, not declaration.

## Definitions: `let` for values, `def` for functions

`let` binds a value, and `def` defines a function or method.
The parser dispatches on the keyword alone, in `parse_statement` in `src/parse/statement.rs`.
`let` goes to `parse_variable_def` and `def` to `parse_fun_def`, both in `src/parse/definition.rs`.
So `def` always takes an argument list and never `mut`, and `let` never takes `pure`.
`meta`, `total` and `pure` are keywords, parsed onto `Node::FunDef` in the fixed order `def meta total pure`.
Only `pure` is checked; `meta` and `total` are reserved, and their gaps have `ignore[...]` fixtures.
A class field is a value, so it is `let`, and a lambda bound to a name is too, as in `let f := \x := x + 1`.
External trait implementation, `def <Trait> for <Class> where ... end`, is future work and keeps `def`.

## Mutability: `mut`, and the absence of `fin`

Every binding is immutable unless marked `mut`.
`fin` is not a keyword, and has not been one since the polarity was inverted.
If you see `fin` anywhere, it is stale and should be fixed.

`mut` is parsed by `parse_expression_type` in `src/parse/ty.rs`, and by `parse_expression_maybe_type` in `src/parse/control_flow_expr.rs` for match cases.
Both are a single `it.eat_if(&Token::Mut).is_some()`.
Because every binding site funnels through those, one marker covers variables, function arguments, class constructor arguments and `self`.
So it is `let mut a := 10`, `def f(mut a: Int)`, `class X(mut a: Int)` and `def f(mut self)`.

A tuple takes one marker per element, as `let (mut a, mut b) := (1, 2)`.
Marking the tuple itself, as `let mut (a, b)`, is a parse error.
The reason is that `mut` belongs to a binding, and a tuple introduces one binding per element.
`parse_binding_id` and `parse_binding_element` in `src/parse/ty.rs` parse the elements.
Two places must then leave those per-element flags alone.
They are `Identifier::try_from` in `src/check/ident.rs`, and `id_from_var` in `src/check/constrain/generate/definition.rs`.
Both previously forced every element to the definition's own flag via `as_mutable`.
Destructuring a list or set, as `let [a, b]`, does not parse at all.

What is actually enforced is narrower than what is written down.
Reassigning a binding that is not `mut` is an error, and so is reassigning through a receiver that is not `mut`.
`mut` on a class field is recorded in `Context` but never checked, and `mut self` is not required to call a mutating method.
Each gap has an `ignore[...]` fixture; see `tests/README.md`.

The `pure` rules from the README *are* enforced, in `src/check/constrain/generate/`.
`env.rs` carries `in_pure`, `outer_mut` and `pure_nonlocal`, set by `in_pure` when entering a pure body.
`call.rs` holds `check_pure_call`, `check_pure_method`, `check_pure_field_read` and `check_pure_assign`.
`definition.rs` rejects `pure` with `mut self`, and `expression.rs` rejects reading an outer `mut` variable.
Method and field rules need the receiver's class at generation time, so they reach `self` and annotated variables, not a receiver whose type only unification resolves.

A class body may only declare fields and methods, enforced by `check_only_declarations` in `class.rs`.
That is what makes construction pure by default: `Class::constructor` in `src/check/context/clss/mod.rs` reports pure unless an explicit `__init__` is impure.

Fixtures should still be annotated as if all of it were enforced, so they stay correct when it is.

## Documentation

### Writing style (applies to every `.md` file in this repo)

These rules are not optional, and they apply to edits as well as to new prose.

- **One sentence per line.**
  Every sentence starts on its own line and ends on that same line.
- **Never wrap a sentence across lines.**
  Do not cobble a sentence together from several lines, and do not reflow prose to a column limit.
  A long line is correct; a wrapped one is not.
- **Favour short sentences.**
  Split a long sentence into several short ones rather than joining clauses.
- **No em dashes.**
  Do not use `—`, and do not use ` -- ` as a prose dash either.
  Rewrite into separate sentences, or use a comma or colon.
  A literal `--` that is part of a command, such as `cargo fmt --all -- --check`, is fine.

Leave code blocks, tables and verbatim CLI output alone.
Multi-sentence table cells stay as they are, since splitting them would break the row.

`docs/` contains the language specification and philosophy docs, including the formal grammar at `docs/spec/grammar.md`.
Those docs are partially outdated, as their own README says.
The top-level `README.md` has a larger set of annotated Mamba code examples, covering functions, collections, classes and error handling.
Check those for concrete syntax before assuming behavior from the grammar spec alone.

## Contribution conventions

- PRs target `develop`, not `main` (see `CONTRIBUTING.md`); `main` is release-only.
- Keep `Cargo.toml` dependencies alphabetically sorted (`cargo sort`).

### Git is off limits unless asked

Do not run `git` commands that change anything.
This includes `commit`, `branch`, `checkout`, `switch`, `rebase`, `reset`, `cherry-pick`, `merge`, `tag`, `stash`, `push` and `git rm`.
The exception is when the user gives a specific instruction to do so, for that specific task.
A general request to fix or clean something up is not such an instruction.

Read-only inspection is always fine.
That means `git status`, `git log`, `git diff`, `git show` and `git branch --list`.
Prefer those for orienting yourself.

`git mv` is allowed without asking, for renaming or moving a tracked file.
Prefer it over a plain `mv`, which git records as a delete plus an add.
It stages the rename, which is expected and is how git records one.

When a task does seem to call for changing history or branches, describe the commands and let the user run them, unless they have said otherwise.

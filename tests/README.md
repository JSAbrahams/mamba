# Test suite notes

This file collects findings from coverage work on this test suite that aren't obvious from
reading the code, so future test-writing doesn't have to rediscover them. For the test suite's
structure and conventions (fixture layout, `test_case` tables, how `to_python` diffs work), see
the root `CLAUDE.md` — this file is specifically about *what to test next* and *why some things
can't be*.

## `tests/execution.rs`: actually running the output, not just diffing its AST

Every other Python-backend test (`tests_util::test_directory`, used throughout
`tests/check/valid.rs`) only asserts that generated Python is *structurally* equivalent to a
reference `.py` file (a parsed-AST diff) — it never executes the result. `tests/execution.rs` is
deliberately different: it calls `transpile_dir` directly (both with the default `Backend::Python`
and with `Backend::Bin`), then actually runs what comes out — `tests_util::run_python` shells out
to `python3` and captures stdout; the `Backend::Bin` case runs the linked executable directly via
`std::process::Command` — and asserts on the captured output. Both tests currently share one
fixture, `tests/resource/valid/function/hello_world.mamba` (also registered as an ordinary
AST-diff test in `tests/check/valid.rs`, for coverage of the plain `print` path) — pick a fixture
within the Cranelift backend's supported subset (see `src/backend/cranelift/lower.rs`'s doc
comment) if adding more of these, since unlike the Python backend, it doesn't aim to eventually
support everything.

Regenerate coverage with the same exclusions CI/Codecov use (`.github/workflows/coverage.yml`),
so your local numbers match what you see on the dashboard — without `--ignore-filename-regex`,
`tests/*.rs` and `tests_util/src/lib.rs` count toward the total too, which inflates/dilutes the
percentage in a way that doesn't reflect actual compiler coverage:

```sh
cargo llvm-cov --package mamba --lcov --output-path target/lcov.info \
  --ignore-filename-regex '(^|/)tests?/.*|(^|/)tests_util/.*|.*_tests\.rs$'
cargo llvm-cov --package mamba --summary-only --json --output-path target/cov.json \
  --ignore-filename-regex '(^|/)tests?/.*|(^|/)tests_util/.*|.*_tests\.rs$'
```

`target/lcov.info` has per-line `DA:<line>,<hit-count>` records per file if you need to find
exactly which lines are still uncovered.

## The goal is 100%. It isn't reachable yet, and here's exactly why not

100% line coverage via `.mamba`/`.py` fixtures is the actual target, not just "high" coverage —
a language this small should have every checker branch pinned down by a fixture, so a subtle
regression in any language feature fails a test. It isn't at 100% today for two very different
reasons, and they should be treated differently:

1. **A handful of files are permanent, structural exceptions.** `main.rs` and anything that's
   supposed to be a thin CLI/plumbing wrapper around `lib.rs` (`io.rs`, `common/result.rs`,
   `check|parse|generate/result.rs` — mostly `Display`/error-formatting glue) should stay close
   to 0% covered by fixtures *by design*: there's no meaningful language-semantics branch there
   for a `.mamba` file to exercise, only argument parsing and error-message plumbing, which
   belongs in `#[cfg(test)]` unit tests if it's tested at all (see `main.rs`'s own thinness — if
   one of these files ever grows real branching logic, the fix is to move that logic into
   `lib.rs`/`check`/`generate` where it's testable, not to write CLI-level tests around it in
   place). Don't chase these toward 100%; a low number here is the design working as intended.
2. **Everything else uncovered is a real, closeable gap, and should eventually hit 100%.** Most
   of what's listed below is uncovered *because the underlying language feature isn't fully wired
   up yet*, not because it's untestable in principle:
   - **Python-stub-content-gated code** (next section) is closeable *right now* — it's just
     waiting for a realistic stub addition (see `divmod.py` for the pattern) and a fixture proving
     it end-to-end. Treat any uncovered branch here as a to-do, not a permanent gap.
   - **Feature-shaped dead code** (the `args_compatible`/`Context::field` entries below) exists
     because the feature it was clearly written for — call-site arity checking, top-level constant
     lookup — was never finished being wired in. The fix there is to finish the feature, then add
     the fixture that exercises it; deleting the code would just trade an honest 0% for silently
     losing the half-built feature.
   - Genuinely redundant leftover code (duplicate impls superseded by another one, dead-by-
     construction branches) gets deleted outright rather than tracked here — see recent git
     history around this file's introduction for what was removed and why. This file only tracks
     *remaining* gaps, so a deletion doesn't leave a stale entry behind.

## Python-stub files are also test surface, not just runtime data

`src/check/resource/primitive/*.py` and `src/check/resource/std/*.py` are real Python source,
parsed with `python_parser` once per `check_all` call, to seed `Context` with built-in
class/function/field signatures (see `check/context/python.rs::python_files`,
`check/context/clss/python.rs`). Several parsing branches in `check/context/field/python.rs`,
`check/context/parameter/python.rs`, `check/context/python.rs`, and
`check/name/true_name/python.rs` depend entirely on *what these stub files contain* — a
tuple-return-type annotation, a `Union[...]` subscript, a module-level assignment, etc. If a
branch there is uncovered, the fix is a *stub content* change (a realistic addition mirroring
real CPython signatures), not a new `.mamba` fixture — though you still generally want a small
`.mamba`/`.py` fixture afterwards to exercise the new signature end-to-end and prove it actually
works (e.g. `tests/resource/valid/function/divmod.mamba` exercises the `divmod` stub added to
`src/check/resource/std/divmod.py`, whose `-> (int, int)` return annotation is what actually
covers the bare-tuple-literal-as-type-annotation branch in `true_name/python.rs`).

### `__debug__` / `__all__` (`src/check/resource/std/builtins.py`)

Added purely to give context-building one typed and one untyped top-level (non-class) assignment
to parse, covering the `Statement::Assignment` / `Statement::TypedAssignment` arms in
`check/context/python.rs::python_files`. Both are real CPython builtins:

- `__debug__` — a real builtin `bool`, `True` unless Python is run with `-O`; guards debug-only
  code (`assert` compiles away entirely under `-O`).
- `__all__` — a real, common module-level convention: a list of the names a module exports via
  `from module import *`.

**They are not currently usable from Mamba source.** See "Top-level fields are parsed but never
looked up" below — `print(__debug__)` in a `.mamba` fixture still fails with
`Undefined variable: __debug__` even with this stub present. They're kept anyway because they're
accurate documentation of real Python builtins, and their presence alone (regardless of whether
anything ever references them) already exercises those two parsing arms for every test that
calls `check_all`, so no separate fixture is possible or needed for them.

## Remaining dead code (no caller anywhere in `src/`) — all feature-shaped, kept on purpose

Everything genuinely redundant (a duplicate impl superseded by another, a branch dead by
construction) found while chasing uncovered lines has already been deleted. What's left is kept
specifically because it reads like an unfinished feature, not leftover cruft — fix these by
implementing the feature and adding the fixture that exercises it, not by deleting them:

- `Function::args_compatible` and `Function::simple_fun` (`check/context/function/mod.rs`) —
  `args_compatible` reads like it was meant to be the call-site arity/type check for values of
  function type; see the "anonymous function arity" gap below, which is consistent with this
  never having been wired up. Implementing it (calling it from wherever an anonymous function or
  callable value is matched against an expected callable type) and adding an invalid fixture with
  a mismatched arity is the way to close both this and that gap at once. `Display for Function`
  in the same file is kept alongside these two for the same reason — it currently has no other
  caller, but `args_compatible`'s error messages already format `{self}` (a whole `Function`),
  so it stops being dead the moment `args_compatible` is wired in.
- `Context`'s `LookupField` impl, i.e. `Context::field` (`check/context/field/mod.rs`) — see
  "Top-level fields are parsed but never looked up" below. Wiring identifier resolution to
  actually call this for a bare (non-local) name is the way to close that gap.

## Two "dead" items that turned out to be real, unfinished wiring, not cruft

Two items originally flagged and deleted in a dead-code sweep were restored once a genuine
production use was found for each, instead of staying deleted:

- **`GenericClass::all_pure` / `GenericFunction::pure`** (`check/context/{clss,function}/generic.rs`):
  `pure` on a function was already a fully-working *parsed* flag (`def pure f(...)` sets
  `GenericFunction.pure` correctly, see `from_fundef_pure` in `check/context/function/generic.rs`'s
  tests, and `tests/resource/valid/function/pure_function.mamba` exercises it end-to-end through
  `check_all` and codegen) — but `pure`/`all_pure` themselves genuinely had zero callers, because
  nothing bulk-marks a whole class pure from Mamba syntax (there's no such construct — `pure` is
  strictly per-function). The real use was on the *Python-stub* side instead: every method built
  from a primitive/stdlib `.py` class (`check/context/clss/python.rs`'s `TryFrom<&Classdef> for
  GenericClass`) is now marked pure via `.all_pure(true)`, since a built-in operation like
  `int.__add__` or `str.__eq__` is, definitionally, a deterministic operation with no observable
  side effect from Mamba's perspective. This is deliberately *not* applied to top-level Python
  functions (`check/context/function/python.rs`'s `GenericFunction::from`) — `input()` is a real
  counter-example, a top-level builtin that is emphatically not pure — so that path still starts
  functions as impure by default. Note the check/constrain stage still doesn't *enforce* any of
  the `pure` restrictions described in `README.md` (self must be `fin`, no calling impure
  functions, etc. — see the correctness gap noted further down); this only fixed the dead-code
  problem, not that separate, larger gap.
- **`Token::equals_name`** (`parse/lex/token.rs`): was flagged dead, then wired into
  `parse/result.rs::expected_one_of` to replace a duplicated inline
  `t.to_string() == t.name().to_string()` comparison that already existed right next to it.

The lesson for next time: before deleting something that reads like it should obviously be used
for X, check whether X actually has a call site anywhere near the definition (same file, sibling
module) rather than only searching from the *feature* end (`def pure ...` fixtures) inward.

## Dead-code sweep methodology (and one sharp edge)

A whole-tree sweep of every `fn`/`struct`/`enum`/`trait`/`type` definition under `src/parse`,
`src/check`, `src/generate`, and `src/common` — grepping the entire tree (`src/`, `tests/`,
`tests_util/`) for `\bname\b` and flagging anything appearing only at its own definition site —
found and removed three genuinely dead items with no caller and no feature-shape to them:
`TypeErr::append_msg` (`check/result.rs`) and `CaretPos`'s manual `lt`/`le`/`gt`/`ge` overrides
(`common/position.rs` — redundant even had they been called: `PartialOrd`'s default-provided
versions already delegate to `partial_cmp` identically). (Two other items the same sweep flagged,
`GenericFunction::pure`/`GenericClass::all_pure` and `Token::equals_name`, turned out to have real
uses once looked at from the feature side rather than the caller-count side — see above.)
Re-running the same sweep afterwards with no exceptions found nothing further in this category.

**One false positive worth remembering for next time**: `impl Termination for AST`
(`parse/ast/mod.rs`) looked identically dead by this method — zero textual references to
`.report(` anywhere — but deleting it broke the build. `tests/parse/valid.rs`'s `syntax` test
functions return `ParseResult<AST>`, and the standard test harness requires a `#[test]` fn's
return type to implement `Termination`; `Result<T, E>`'s blanket impl needs `T: Termination`,
which only `AST`'s manual impl provides. Nothing in the source text ever spells out `Termination`
or `report` at a call site — the requirement comes entirely from the test harness's generated
code — so a pure grep-for-callers sweep can't see it. The general lesson: a trait impl whose
methods are invoked only through a compiler-inserted bound (`Termination`, but the same applies
to `Drop`, operator traits reached only through their operator syntax, etc.) needs the deletion
verified with a real `cargo build --tests`/`cargo test`, not just a caller-count heuristic —
which is exactly why every deletion in this file went through that verification before landing.

## A whole dead module the sweep's `\bname\b` heuristic couldn't see: `FunUnion`

`check/context/function/union.rs` (`FunUnion`, a "set of overloaded `Function`s" wrapper, plus
`PartialEq`/`Hash`/two `From` impls/`Display`/`TryFromPos<&FunUnion> for Function` — 40 lines)
had zero callers anywhere outside its own file, and its `pub mod union;` declaration
(`check/context/function/mod.rs`) had zero importers anywhere — but the original sweep missed it
entirely, because `FunUnion` is mentioned *many* times within its own file (once per impl target),
so `\bFunUnion\b`'s occurrence count was never close to 1. The type-level sweep needs to count
occurrences *outside the defining file*, not anywhere at all, to catch a type that's heavily
self-referential (lots of impls for itself) while being globally unused. Deleting it also orphaned
`check/result.rs`'s `TryFromPos` trait (its only impl was in `union.rs`), which came out too.
`git log` on the file showed its last real change was in a `Streamline Name logic` commit, well
before this project's current state — consistent with overload resolution having moved to the
`Name`-as-set-of-types system and this wrapper simply never being deleted at the time.

## Two more `in_class` methods with the same dead-by-construction branch as before

`GenericField::in_class` (`check/context/field/generic.rs`) and `GenericFunction::in_class`
(`check/context/function/generic.rs`) both had the identical shape already fixed once for
`GenericFunctionArg::in_class`: an `Option<&StringName>` parameter with a `class.is_none()` (or
`else`) arm returning an error, where *every* call site (`check/context/clss/generic.rs`,
`check/context/clss/python.rs`) already only ever passes `Some(...)`. Simplified both to take
`&StringName` directly and return the plain value instead of a `TypeResult`, updating all 5 call
sites accordingly. Worth checking for this exact shape (`Option<&T>` param + "not in class" style
error + every caller passing `Some`) elsewhere if a similar refactor comes up again — it seems to
have been a repeated pattern across this module rather than a one-off.

`GenericField::try_from`'s `Node::VariableDef` arm (in the same file) turned out to be dead too,
for a related reason: its only caller (`ClassArgument::try_from` in
`check/context/arg/generic.rs`) only ever passes a `Node::FunArg`. Removed that arm; a bare
`Node::VariableDef` still becomes a field correctly through the *separate* `GenericFields`
`TryFrom` impl in the same file (which additionally handles tuple-destructuring, which
`GenericField`, singular, never needed to).

## A real bug found by testing the poorly-covered parts: default-argument type inference

`check/context/arg/generic.rs`'s `GenericFunctionArg::try_from` infers a parameter's type from its
default value when no explicit annotation is given (`def f(a := 5)` infers `Int` for `a`). This
whole branch (`Node::Str`/`Node::Int`/`Node::Real`/`Node::ENum`/boolean-`Node::Id` arms) was
completely uncovered — and turned out to be broken: it built the inferred type from
`clss::python::STRING_PRIMITIVE`/`BOOL_PRIMITIVE`/`INT_PRIMITIVE`/`FLOAT_PRIMITIVE` (the *Python*-side
primitive names — `"str"`, `"bool"`, `"int"`, `"float"`, lowercase), not the Mamba-side names
`clss::STRING`/`BOOL`/`INT`/`FLOAT` (`"Str"`, `"Bool"`, `"Int"`, `"Float"`). The bug was silent as
long as the inferred type was only ever used as the parameter's own default (nothing looks the
type name up in that case) — it only surfaced once something needed to *resolve* that type name,
e.g. checking a call-site argument against it (`f("hi")` erroring with `Type 'str' is undefined.`,
since only the capitalized Mamba name is a registered class in `Context`). Fixed to use the
Mamba-side constants; `tests/resource/valid/function/infer_default_arg_type.mamba` now exercises
all four literal kinds through an actual call site with explicit overrides (not just relying on
the defaults, which is exactly what would have kept this bug hidden), and
`tests/resource/invalid/type/function/arg_default_not_literal.mamba` covers the sibling
"can only infer type of literals" error arm (a non-literal default with no annotation). This is
the clearest example so far of coverage work finding a real correctness bug rather than just
padding a percentage — worth remembering when a "poorly covered" branch looks like real, reachable
logic rather than a dead/defensive one: write the fixture and see what actually happens before
assuming the branch is merely undertested.

## Top-level fields are parsed but never looked up

`check/context/generic.rs::generics` and `check/context/python.rs::python_files` both parse
module-level (non-class) `VariableDef`/`Statement::Assignment` into `Context.fields`. Nothing
downstream ever queries `Context.fields` for a bare identifier: the only lookup path
(`Context`'s `LookupField` impl in `check/context/field/mod.rs`) has no caller (see dead code
above) — confirmed empirically, not just by grep: adding a top-level constant to a stub file and
referencing it by name from a `.mamba` fixture still fails with `Undefined variable: <name>`.
Top-level Mamba variable definitions still work, but only because they're tracked through the
normal environment/scope mechanism used for local variables, not through `Context.fields`. This
is a real, closeable coverage gap, not a permanent one: once identifier resolution actually
consults `Context.fields` for names the environment doesn't already have, `__debug__`/`__all__`
become referenceable and a normal `.mamba`/`.py` fixture can cover the whole path end to end.

## Known type-checker gap: anonymous-function/callable arity isn't enforced

Tried three different ways to make the checker reject an anonymous function passed where a
callable of a different arity was expected: a 2-arg lambda passed as a 1-arg `(Int) -> Int`
parameter, a 1-arg lambda where 2 were needed, and direct assignment to a
`(Int) -> Int`-annotated variable. None errored. `unify_function`'s arity-mismatch branch in
`check/constrain/unify/function.rs` (the `EitherOrBoth::Left(_) | EitherOrBoth::Right(_)` arm)
looks unreachable in practice — consistent with `Function::args_compatible` (which reads like
the intended arity check) being dead code. This is a correctness gap as much as a coverage one:
the arm can't be covered by a fixture until the checker actually calls into this arity-checking
logic somewhere, so closing it means finishing the feature (wire up `args_compatible` or
equivalent), then adding the invalid fixture the arm has been waiting for.

## Cyclic self-field dependencies currently type-check and generate (maybe shouldn't)

`tests/resource/valid/class/cyclic_field_dependency.mamba` documents current behavior: two class
fields whose initializers each reference the other via `self` both type-check and generate
(hitting the cycle-breaker in `generate/convert/class.rs::order_by_self_field_deps`), even though
running the generated Python would raise at the first read of the not-yet-assigned field. If this
gets disallowed in future (detecting the cycle and rejecting it at check time), this fixture
should move to `tests/resource/invalid/type/class/` and get a `matches Err(_)` test_case instead
of being deleted outright, so the "was silently accepted" behavior isn't lost from history.

## Flaky test: `tests/main.rs` under parallel execution

Seen once under `cargo llvm-cov` (which runs slower/instrumented) with default parallelism: one
of the `tests/main.rs` black-box CLI tests failed, but passed both under plain `cargo test` and
under `cargo llvm-cov ... -- --test-threads=1`. Several of these tests write to shared, fixed
paths under `tests/resource/valid/dummy/proj1/` (e.g. `target`, `custom_target`) rather than a
randomized temp dir, which is a plausible source of a cross-test race under parallel execution.
Not something introduced by the coverage work in this file — pre-existing test isolation issue,
worth a look if it starts flaking in CI. Workaround: run with `-- --test-threads=1` if it flakes.

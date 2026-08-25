# Test suite notes

This file collects findings from coverage work on this test suite that aren't obvious from
reading the code, so future test-writing doesn't have to rediscover them. For the test suite's
structure and conventions (fixture layout, `test_case` tables, how `to_python` diffs work), see
the root `CLAUDE.md` — this file is specifically about *what to test next* and *why some things
can't be*.

Regenerate coverage with:

```sh
cargo llvm-cov --package mamba --lcov --output-path target/lcov.info
cargo llvm-cov --package mamba --summary-only --json --output-path target/cov.json
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
   - A smaller amount is genuinely **redundant leftover code** (a duplicate impl superseded by
     another one, e.g. the `Vec<Subscript>` `GenericParameters` impl) rather than an unfinished
     feature — those *are* plain cleanup/deletion candidates, and are called out individually
     below so the two categories don't get conflated.

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

`type X: (Int) -> Int` / `type X: (Int, Str)` (a type alias whose parent is a callable or tuple
type, see `tests/resource/valid/class/type_callable_parent.mamba` /
`type_tuple_parent.mamba`) exercise the *Mamba-source* side of the same kind of thing
(`check/name/true_name/generic.rs`'s `TypeFun`/`TypeTup` arms) — worth knowing these two arms
were already covered by existing tests before these were added (via `callable_fun_arg` and
`assign_tuples`-style fixtures), so those two new fixtures earn their keep on realism/breadth
grounds (a class extending a callable-type alias is a real, previously-untested shape) rather
than by moving that specific file's coverage number.

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

## Confirmed dead code (no caller anywhere in `src/`)

Found while chasing uncovered lines — grepped for every call site, none exist outside the
defining module. Split into the two categories from above: finish-the-feature-then-cover vs.
plain cleanup.

**Unfinished feature — fix by implementing, then add the fixture:**

- `Function::args_compatible` and `Function::simple_fun` (`check/context/function/mod.rs`) —
  `args_compatible` reads like it was meant to be the call-site arity/type check for values of
  function type; see the "anonymous function arity" gap below, which is consistent with this
  never having been wired up. Implementing it (calling it from wherever an anonymous function or
  callable value is matched against an expected callable type) and adding an invalid fixture with
  a mismatched arity is the way to close both this and that gap at once.
- `Context`'s `LookupField` impl, i.e. `Context::field` (`check/context/field/mod.rs`) — see
  "Top-level fields are parsed but never looked up" below. Wiring identifier resolution to
  actually call this for a bare (non-local) name is the way to close that gap.

**Genuine leftover cruft — plain deletion candidates, no feature behind them:**

- `Display for Function` (`check/context/function/mod.rs`) — nothing formats a whole `Function`
  value; other error messages format its `name`/individual arguments instead.
- `StringName::match_name` / `match_name_helper` (`check/name/string_name/mod.rs`) and
  `Name::match_name_helper` (`check/name/mod.rs`) — note the free function `check::name::match_name`
  (lowercase, different item) *is* used (tuple-destructuring assignment) and does the equivalent
  job; only the `StringName`/`Name`-associated versions are dead, apparently superseded.
- `impl From<&Vec<Subscript>> for GenericParameters` (`check/context/parameter/python.rs`) — the
  only call site (`check/context/clss/python.rs:49`) passes a `&Vec<Argument>`, always resolving
  to the *other* `From` impl in the same file.
- `GenericFunctionArg::in_class`'s `class.is_none()` error arm and its dead-by-construction
  `else` branch right after it (`check/context/arg/generic.rs:49-62`) — its only caller
  (`GenericFunction::in_class`) already guards `clss.is_some()` before calling it, so `class` can
  never be `None` here.

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

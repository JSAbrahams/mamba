# Test suite notes

This file collects findings from coverage work on this test suite that aren't obvious from reading the code, so future test-writing doesn't have to rediscover them.
For the test suite's structure and conventions, see the root `CLAUDE.md`.
That covers fixture layout, `test_case` tables, and how `to_python` diffs work.
This file is specifically about *what to test next*, and *why some things can't be*.

## `tests/execution.rs`: actually running the output, not just diffing its AST

Every other Python-backend test only asserts that generated Python is *structurally* equivalent to a reference `.py` file, via a parsed-AST diff.
That is `tests_util::test_directory`, used throughout `tests/check/valid.rs`.
It never executes the result.
`tests/execution.rs` is deliberately different.
It calls `transpile_dir` directly, both with the default `Backend::Python` and with `Backend::Bin`.
It then actually runs what comes out, and asserts on the captured output.
`tests_util::run_python` shells out to `python3` and captures stdout.
The `Backend::Bin` case runs the linked executable directly via `std::process::Command`.
Both tests currently share one fixture, `tests/resource/valid/function/hello_world.mamba`.
That fixture is also registered as an ordinary AST-diff test in `tests/check/valid.rs`, for coverage of the plain `print` path.
If adding more of these, pick a fixture within the Cranelift backend's supported subset.
See `src/backend/cranelift/lower.rs`'s doc comment for that subset.
This matters because, unlike the Python backend, it doesn't aim to eventually support everything.

Regenerate coverage with the same exclusions CI and Codecov use, from `.github/workflows/coverage.yml`.
That way your local numbers match what you see on the dashboard.
Without `--ignore-filename-regex`, `tests/*.rs` and `tests_util/src/lib.rs` count toward the total too.
That skews the percentage in a way that doesn't reflect actual compiler coverage:

```sh
cargo llvm-cov --package mamba --lcov --output-path target/lcov.info \
  --ignore-filename-regex '(^|/)tests?/.*|(^|/)tests_util/.*|.*_tests\.rs$'
cargo llvm-cov --package mamba --summary-only --json --output-path target/cov.json \
  --ignore-filename-regex '(^|/)tests?/.*|(^|/)tests_util/.*|.*_tests\.rs$'
```

`target/lcov.info` has per-line `DA:<line>,<hit-count>` records per file if you need to find exactly which lines are still uncovered.

## The goal is 100%. It isn't reachable yet, and here's exactly why not

100% line coverage via `.mamba`/`.py` fixtures is the actual target, not just "high" coverage.
A language this small should have every checker branch pinned down by a fixture.
That way a subtle regression in any language feature fails a test.
It isn't at 100% today for two very different reasons, and they should be treated differently:

1. **A handful of files are permanent, structural exceptions.**
   These are `main.rs`, and anything that's supposed to be a thin CLI or plumbing wrapper around `lib.rs`.
   That covers `io.rs`, `common/result.rs`, and `check`/`parse`/`backend/python`'s `result.rs`, which are mostly `Display` and error-formatting glue.
   They should stay close to 0% covered by fixtures *by design*.
   There's no meaningful language-semantics branch there for a `.mamba` file to exercise.
   There is only argument parsing and error-message plumbing, which belongs in `#[cfg(test)]` unit tests if it's tested at all.
   See `main.rs`'s own thinness for the intended shape.
   If one of these files ever grows real branching logic, the fix is to move that logic into `lib.rs`, `check` or `backend` where it's testable.
   Do not write CLI-level tests around it in place.
   Don't chase these toward 100%.
   A low number here is the design working as intended.
2. **Everything else uncovered is a real, closeable gap, and should eventually hit 100%.**
   Most of what's listed below is uncovered *because the underlying language feature isn't fully wired up yet*, not because it's untestable in principle:
   - **Python-stub-content-gated code**, covered in the next section, is closeable *right now*.
     It's just waiting for a realistic stub addition, plus a fixture proving it end-to-end.
     See `divmod.py` for the pattern.
     Treat any uncovered branch here as a to-do, not a permanent gap.
   - **Feature-shaped dead code**, being the `args_compatible` and `Context::field` entries below, exists because the feature it was clearly written for was never finished being wired in.
     Those features are call-site arity checking and top-level constant lookup.
     The fix there is to finish the feature, then add the fixture that exercises it.
     Deleting the code would just trade an honest 0% for silently losing the half-built feature.
   - Genuinely redundant leftover code gets deleted outright, rather than tracked here.
     That means duplicate impls superseded by another one, and dead-by-construction branches.
     See recent git history around this file's introduction for what was removed and why.
     This file only tracks *remaining* gaps, so a deletion doesn't leave a stale entry behind.

## Python-stub files are also test surface, not just runtime data

`src/check/resource/primitive/*.py` and `src/check/resource/std/*.py` are real Python source.
They are parsed with `python_parser` once per `check_all` call, to seed `Context` with built-in class, function and field signatures.
See `check/context/python.rs::python_files` and `check/context/clss/python.rs`.
Several parsing branches depend entirely on *what these stub files contain*.
Those branches live in `check/context/field/python.rs`, `check/context/parameter/python.rs`, `check/context/python.rs`, and `check/name/true_name/python.rs`.
What they need is a tuple-return-type annotation, a `Union[...]` subscript, a module-level assignment, and so on.
If a branch there is uncovered, the fix is a *stub content* change, not a new `.mamba` fixture.
That means a realistic addition mirroring real CPython signatures.
You still generally want a small `.mamba`/`.py` fixture afterwards, to exercise the new signature end-to-end and prove it actually works.
For example, `tests/resource/valid/function/divmod.mamba` exercises the `divmod` stub added to `src/check/resource/std/divmod.py`.
That stub's `-> (int, int)` return annotation is what actually covers the bare-tuple-literal-as-type-annotation branch in `true_name/python.rs`.

### `__debug__` / `__all__` (`src/check/resource/std/builtins.py`)

Added purely to give context-building one typed and one untyped top-level, non-class assignment to parse.
That covers the `Statement::Assignment` and `Statement::TypedAssignment` arms in `check/context/python.rs::python_files`.
Both are real CPython builtins:

- `__debug__` is a real builtin `bool`, `True` unless Python is run with `-O`.
  It guards debug-only code, since `assert` compiles away entirely under `-O`.
- `__all__` is a real, common module-level convention.
  It is a list of the names a module exports via `from module import *`.

**They are not currently usable from Mamba source.**
See "Top-level fields are parsed but never looked up" below.
`print(__debug__)` in a `.mamba` fixture still fails with `Undefined variable: __debug__`, even with this stub present.
They're kept anyway, because they're accurate documentation of real Python builtins.
Their presence alone already exercises those two parsing arms for every test that calls `check_all`, whether or not anything ever references them.
So no separate fixture is possible or needed for them.

## Remaining dead code, with no caller anywhere in `src/`, all feature-shaped and kept on purpose

Everything genuinely redundant found while chasing uncovered lines has already been deleted.
That means a duplicate impl superseded by another, or a branch dead by construction.
What's left is kept specifically because it reads like an unfinished feature, not leftover cruft.
Fix these by implementing the feature and adding the fixture that exercises it, not by deleting them:

- `Function::args_compatible` and `Function::simple_fun`, in `check/context/function/mod.rs`.
  `args_compatible` reads like it was meant to be the call-site arity and type check for values of function type.
  See the "anonymous function arity" gap below, which is consistent with this never having been wired up.
  To close both this and that gap at once, implement it and add an invalid fixture with a mismatched arity.
  Implementing it means calling it from wherever an anonymous function or callable value is matched against an expected callable type.
  `Display for Function` in the same file is kept alongside these two for the same reason.
  It currently has no other caller.
  But `args_compatible`'s error messages already format `{self}`, a whole `Function`, so it stops being dead the moment `args_compatible` is wired in.
- `Context`'s `LookupField` impl, meaning `Context::field` in `check/context/field/mod.rs`.
  See "Top-level fields are parsed but never looked up" below.
  Wiring identifier resolution to actually call this for a bare (non-local) name is the way to close that gap.

## Two "dead" items that turned out to be real, unfinished wiring, not cruft

Two items originally flagged and deleted in a dead-code sweep were restored once a genuine production use was found for each, instead of staying deleted:

- **`GenericClass::all_pure` and `GenericFunction::pure`**, in `check/context/{clss,function}/generic.rs`.
  `pure` on a function was already a fully-working *parsed* flag.
  `def pure f(...)` sets `GenericFunction.pure` correctly, as `from_fundef_pure` in `check/context/function/generic.rs`'s tests shows.
  `tests/resource/valid/function/pure_function.mamba` also exercises it end-to-end through `check_all` and codegen.
  But `pure` and `all_pure` themselves genuinely had zero callers.
  Nothing bulk-marks a whole class pure from Mamba syntax, since there is no such construct.
  `pure` is strictly per-function.
  The real use was on the *Python-stub* side instead: every method built from a primitive/stdlib `.py` class (`check/context/clss/python.rs`'s `TryFrom<&Classdef> for GenericClass`) is now marked pure via `.all_pure(true)`, since a built-in operation like `int.__add__` or `str.__eq__` is, definitionally, a deterministic operation with no observable side effect from Mamba's perspective.
  This is deliberately *not* applied to top-level Python functions, in `check/context/function/python.rs`'s `GenericFunction::from`.
  `input()` is a real counter-example, being a top-level builtin that is emphatically not pure.
  That path therefore still starts functions as impure by default.
  Note the check/constrain stage still doesn't *enforce* any of the `pure` restrictions described in `README.md`.
  Those restrictions include self not being `mut`, and not calling impure functions.
  See the correctness gap noted further down.
  This only fixed the dead-code problem, not that separate, larger gap.
- **`Token::equals_name`**, in `parse/lex/token.rs`.
  This was flagged dead, then wired into `parse/result.rs::expected_one_of`.
  There it replaced a duplicated inline `t.to_string() == t.name().to_string()` comparison that already existed right next to it.

The lesson for next time is about where to look.
Before deleting something that reads like it should obviously be used for X, check whether X actually has a call site anywhere near the definition, such as the same file or a sibling module.
Do not only search inward from the *feature* end, such as `def pure ...` fixtures.

## Dead-code sweep methodology (and one sharp edge)

A whole-tree sweep was run over every `fn`, `struct`, `enum`, `trait` and `type` definition under `src/parse`, `src/check`, `src/backend` and `src/common`.
It grepped the entire tree (`src/`, `tests/`, `tests_util/`) for `\bname\b`, flagging anything appearing only at its own definition site.
That found and removed three genuinely dead items with no caller and no feature-shape to them.
Those were `TypeErr::append_msg` in `check/result.rs`, and `CaretPos`'s manual `lt`/`le`/`gt`/`ge` overrides in `common/position.rs`.
The `CaretPos` overrides were redundant even had they been called, since `PartialOrd`'s default-provided versions already delegate to `partial_cmp` identically.
Two other items the same sweep flagged turned out to have real uses, once looked at from the feature side rather than the caller-count side.
Those were `GenericFunction::pure`/`GenericClass::all_pure` and `Token::equals_name`, described above.
Re-running the same sweep afterwards with no exceptions found nothing further in this category.

**One false positive worth remembering for next time** is `impl Termination for AST`, in `parse/ast/mod.rs`.
It looked identically dead by this method, with zero textual references to `.report(` anywhere.
Deleting it broke the build.
`tests/parse/valid.rs`'s `syntax` test functions return `ParseResult<AST>`, and the standard test harness requires a `#[test]` fn's return type to implement `Termination`.
`Result<T, E>`'s blanket impl needs `T: Termination`, which only `AST`'s manual impl provides.
Nothing in the source text ever spells out `Termination` or `report` at a call site.
The requirement comes entirely from the test harness's generated code.
A pure grep-for-callers sweep therefore can't see it.
The general lesson concerns trait impls whose methods are invoked only through a compiler-inserted bound.
`Termination` is one, and the same applies to `Drop`, and to operator traits reached only through their operator syntax.
Verify such a deletion with a real `cargo build --tests` or `cargo test`, not just a caller-count heuristic.
That is exactly why every deletion in this file went through that verification before landing.

## A whole dead module the sweep's `\bname\b` heuristic couldn't see: `FunUnion`

`check/context/function/union.rs` was 40 lines holding `FunUnion`, a "set of overloaded `Function`s" wrapper.
It also held `PartialEq`, `Hash`, two `From` impls, `Display`, and `TryFromPos<&FunUnion> for Function`.
It had zero callers anywhere outside its own file.
Its `pub mod union;` declaration in `check/context/function/mod.rs` had zero importers anywhere.
The original sweep missed it entirely.
That is because `FunUnion` is mentioned *many* times within its own file, once per impl target, so `\bFunUnion\b`'s occurrence count was never close to 1.
The type-level sweep needs to count occurrences *outside the defining file*, not anywhere at all, to catch a type that's heavily self-referential (lots of impls for itself) while being globally unused.
Deleting it also orphaned `check/result.rs`'s `TryFromPos` trait, whose only impl was in `union.rs`, so that came out too.
`git log` on the file showed its last real change was in a `Streamline Name logic` commit, well before this project's current state.
That is consistent with overload resolution having moved to the `Name`-as-set-of-types system, and this wrapper simply never being deleted at the time.

## Two more `in_class` methods with the same dead-by-construction branch as before

`GenericField::in_class` (`check/context/field/generic.rs`) and `GenericFunction::in_class` (`check/context/function/generic.rs`) both had the identical shape already fixed once for `GenericFunctionArg::in_class`: an `Option<&StringName>` parameter with a `class.is_none()` (or `else`) arm returning an error, where *every* call site (`check/context/clss/generic.rs`, `check/context/clss/python.rs`) already only ever passes `Some(...)`.
Simplified both to take `&StringName` directly and return the plain value instead of a `TypeResult`, updating all 5 call sites accordingly.
Worth checking for this exact shape elsewhere if a similar refactor comes up again.
That shape is an `Option<&T>` param, a "not in class" style error, and every caller passing `Some`.
It seems to have been a repeated pattern across this module, rather than a one-off.

`GenericField::try_from`'s `Node::VariableDef` arm (in the same file) turned out to be dead too, for a related reason: its only caller (`ClassArgument::try_from` in `check/context/arg/generic.rs`) only ever passes a `Node::FunArg`.
Removed that arm; a bare `Node::VariableDef` still becomes a field correctly through the *separate* `GenericFields` `TryFrom` impl in the same file (which additionally handles tuple-destructuring, which `GenericField`, singular, never needed to).

## A real bug found by testing the poorly-covered parts: default-argument type inference

`check/context/arg/generic.rs`'s `GenericFunctionArg::try_from` infers a parameter's type from its default value when no explicit annotation is given (`def f(a := 5)` infers `Int` for `a`).
This whole branch covers the `Node::Str`, `Node::Int`, `Node::Real`, `Node::ENum` and boolean-`Node::Id` arms.
It was completely uncovered, and turned out to be broken.
It built the inferred type from `clss::python::STRING_PRIMITIVE`, `BOOL_PRIMITIVE`, `INT_PRIMITIVE` and `FLOAT_PRIMITIVE`.
Those are the *Python*-side primitive names, being the lowercase `"str"`, `"bool"`, `"int"` and `"float"`.
It should have used the Mamba-side names `clss::STRING`, `BOOL`, `INT` and `FLOAT`, being `"Str"`, `"Bool"`, `"Int"` and `"Float"`.
The bug was silent as long as the inferred type was only ever used as the parameter's own default, since nothing looks the type name up in that case.
It only surfaced once something needed to *resolve* that type name, such as checking a call-site argument against it.
`f("hi")` then errored with `Type 'str' is undefined.`, since only the capitalized Mamba name is a registered class in `Context`.
Fixed to use the Mamba-side constants; `tests/resource/valid/function/infer_default_arg_type.mamba` now exercises all four literal kinds through an actual call site with explicit overrides (not just relying on the defaults, which is exactly what would have kept this bug hidden), and `tests/resource/invalid/type/function/arg_default_not_literal.mamba` covers the sibling "can only infer type of literals" error arm (a non-literal default with no annotation).
This is the clearest example so far of coverage work finding a real correctness bug, rather than just padding a percentage.
It is worth remembering when a "poorly covered" branch looks like real, reachable logic rather than a dead or defensive one.
Write the fixture and see what actually happens, before assuming the branch is merely undertested.

## Top-level fields are parsed but never looked up

`check/context/generic.rs::generics` and `check/context/python.rs::python_files` both parse module-level (non-class) `VariableDef`/`Statement::Assignment` into `Context.fields`.
Nothing downstream ever queries `Context.fields` for a bare identifier.
The only lookup path is `Context`'s `LookupField` impl in `check/context/field/mod.rs`, and it has no caller, as the dead code section above notes.
This was confirmed empirically, not just by grep.
Adding a top-level constant to a stub file and referencing it by name from a `.mamba` fixture still fails with `Undefined variable: <name>`.
Top-level Mamba variable definitions still work, but only because they're tracked through the normal environment/scope mechanism used for local variables, not through `Context.fields`.
This is a real, closeable coverage gap, not a permanent one.
Once identifier resolution actually consults `Context.fields` for names the environment doesn't already have, `__debug__` and `__all__` become referenceable.
A normal `.mamba`/`.py` fixture can then cover the whole path end to end.

## Known type-checker gap: anonymous-function/callable arity isn't enforced

Tried three different ways to make the checker reject an anonymous function passed where a callable of a different arity was expected: a 2-arg lambda passed as a 1-arg `(Int) -> Int` parameter, a 1-arg lambda where 2 were needed, and direct assignment to a `(Int) -> Int`-annotated variable.
None errored.
`unify_function`'s arity-mismatch branch looks unreachable in practice.
That is the `EitherOrBoth::Left(_) | EitherOrBoth::Right(_)` arm, in `check/constrain/unify/function.rs`.
That is consistent with `Function::args_compatible`, which reads like the intended arity check, being dead code.
This is a correctness gap as much as a coverage one.
The arm can't be covered by a fixture until the checker actually calls into this arity-checking logic somewhere.
Closing it therefore means finishing the feature, by wiring up `args_compatible` or an equivalent.
Then add the invalid fixture the arm has been waiting for.

## Cyclic self-field dependencies currently type-check and generate (maybe shouldn't)

`tests/resource/valid/class/cyclic_field_dependency.mamba` documents current behavior: two class fields whose initializers each reference the other via `self` both type-check and generate (hitting the cycle-breaker in `backend/python/convert/class.rs::order_by_self_field_deps`), even though running the generated Python would raise at the first read of the not-yet-assigned field.
If this gets disallowed in future (detecting the cycle and rejecting it at check time), this fixture should move to `tests/resource/invalid/type/class/` and get a `matches Err(_)` test_case instead of being deleted outright, so the "was silently accepted" behavior isn't lost from history.

## Fixed: a class could inherit from a function or tuple type

`docs/spec/grammar.md`'s `class-def` rule restricts a parent to `type-not-fun`, but `parse_parent` (`src/parse/class.rs`) parses it with the unrestricted `parse_type`, so the parser alone never rejected `class Foo(a: Int): (Int) -> Int` or `class Foo(a: Int): (Int, Str)`.
Before this fix, the `TryFrom<&AST>` impls for `StringName` and `TrueName` both happily resolved a `Node::TypeFun` or `Node::TypeTup` parent to a callable or tuple `StringName`.
Those impls live in `check/name/string_name/generic.rs` and `check/name/true_name/generic.rs`.
So both examples type-checked, and generated `class Foo(Callable[[int], int]): ...  Callable.__init__(self)` and `class Foo(Tuple[int, str]): ... Tuple.__init__(self)`.
That is Python which would raise at runtime, since neither `typing.Callable` nor `typing.Tuple` has a real `__init__` to call this way.
`GenericParent::try_from`, in `check/context/parent/generic.rs`, now rejects a `Node::TypeFun` or `Node::TypeTup` parent directly, before ever reaching `TrueName::try_from`.
That rejection is shared by both class and trait parent resolution.
See `tests/resource/invalid/type/class/parent_function_type.mamba` and `parent_tuple_type.mamba`.

## Known checker gap: multi-variable builder/comprehension syntax

`[(x, y) | x in a, y in b]`-style builders (list, set, and dict alike) only resolve the *first*
bound variable when constructing the head expression's scope.
A second `in` generator, or a local `y = ...` binding, both produce "Undefined variable: y".
That happens even though the condition parses and checks fine on its own.
Confirmed with minimal repros for list-, set-, and dict-builders; only
a single bound variable (optionally filtered, e.g. `[x | x in a, x > 0]`) currently works. This
affects two of the `readme_example` fixtures, `lists` and `sets_maps`.
See their `=> ignore[...]` reasons in `tests/check/valid.rs`.
It is also called out in the README's Collections section.

## Known generator bug: `--annotate` can emit invalid Python for a shadowed loop variable in tail position

`readme_example/factorial_dynamic` (`tests/check/valid.rs`) is ignored because `--annotate` (`Arguments { annotate: true, .. }`, which `tests_util::test_directory` always sets) generates syntactically invalid Python when a `def`-shadowed loop variable is *also* the trailing expression of a `match` case that becomes a function's return value.
The root cause is in `append_ret`, in `src/backend/python/convert/mod.rs`.
It blindly recurses into the *last statement* of a `Block`, to turn it into a `return`.
But `wrap_scoped`, in `src/backend/python/convert/control_flow.rs`, appends a scope-restore `if/else` as that last statement for cleanup, not as the value.
That restore looks like `if __mamba_i_existed: i = ... else: del i`.
So `append_ret` recurses into the restore branches themselves, emitting `return i = __mamba_i_saved` and `return del i`.
`--annotate` is already documented as "currently still buggy" in the CLI help, in `src/cli.rs`.
This is one concrete reproduction of that.
Fixing it means either having `wrap_scoped` run outside/after `append_ret`, or teaching `append_ret` to skip over a trailing scope-restore `IfElse` and target the statement before it instead.

## Un-ignoring an `invalid` test: don't drop the `=> matches Err(_)` with the `=> ignore[...]`

`tests/check/invalid.rs`'s `fail_check` returns `TypeResult<()>`, and every active case asserts on it with `=> matches Err(_)`.
A `test_case` with *no* return matcher instead passes only if the function returns `Ok`.
So for an `invalid` fixture, deleting the matcher along with the `=> ignore[...]` silently inverts the assertion.
The test then passes precisely when the checker *fails* to reject the bad program.
It reads as a green "feature implemented" when nothing was implemented at all.
When un-ignoring one of these, write `=> matches Err(_)`, not a bare `#[test_case(...)]`.

This was checked at the time of writing.
All four mutability-related ignores in `tests/check/invalid.rs` still fail with the matcher in place.
Those are `collection/dictionary_assume_not_optional`, `definition/nested_non_mut_field`, `definition/reassign_non_mut_field` and `function/call_mut_function_on_non_mut`.
Their `ignore[...]` reasons are therefore accurate, and they are *not* stale.
They only appear to pass if the matcher is dropped.

## Known generator bug: a single-expression body loses its `return` when `--annotate` is off

`def free_fn(x: Int) -> Int := x + 1` generates `def free_fn(x): x + 1`, with no `return`.
The function therefore silently evaluates to `None`.
Adding `-a`/`--annotate` generates the correct `return x + 1`.
This affects plain functions and methods alike, and any body that is a bare expression rather than a `do ... end` block (a `do ... end` body, as in `tests/resource/valid/function/return_last_expression.mamba`, returns correctly either way).

Worth emphasising because **no `--annotate` is the CLI default**, so this is the default output path, and because the fixture suite cannot catch it: `tests_util::test_directory` always sets `annotate: true`, and `tests/check/valid.rs`'s only `annotate: false` case is `to_python_with_args`'s `collection/tuple`.
The runtime tests in `tests/execution.rs` do go through the CLI, so that is the natural place for a regression test once this is fixed.

## Known generator bug: `--annotate` emits a forward reference for a method typed with its own class

A method can be annotated with the class currently being defined, as in `def +(self, other: Vec2) -> Vec2` inside `class Vec2`.
That generates `def __add__(self, other: Vec2) -> Vec2:`.
Python rejects this at class-creation time with `NameError: name 'Vec2' is not defined`, since the name is not bound until the `class` statement finishes.
Python's own fix is to emit the annotation as a string (`other: "Vec2"`) or to add `from __future__ import annotations` to the output's imports.
Only reachable with `--annotate`, which is already documented as buggy in `src/cli.rs`; called out in `docs/features/data/operator_overloading.md`, whose example is the natural fixture for it.

## Flaky test: `tests/main.rs` under parallel execution

Seen once under `cargo llvm-cov` (which runs slower/instrumented) with default parallelism: one of the `tests/main.rs` black-box CLI tests failed, but passed both under plain `cargo test` and under `cargo llvm-cov ... -- --test-threads=1`.
Several of these tests write to shared, fixed paths under `tests/resource/valid/dummy/proj1/` (e.g. `target`, `custom_target`) rather than a randomized temp dir, which is a plausible source of a cross-test race under parallel execution.
This was not introduced by the coverage work in this file.
It is a pre-existing test isolation issue, worth a look if it starts flaking in CI.
Workaround: run with `-- --test-threads=1` if it flakes.

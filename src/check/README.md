<p align="center">
    <img src="../../image/logo.svg" height="150" alt="Mamba logo"/>
</p>

# Check

Type checks an `AST` in three phases, checks its matches, and annotates it as `ASTTy`.

## Context

`context/` collects the signatures of every class and function in scope, without checking bodies.
That includes imported Mamba files and the Python stubs in `resource/primitive` and `resource/std`.
Signatures must be complete: every argument, return type and class field is typed.

## Constraint generation

`constrain/generate` walks the `AST` and emits constraints linking each expression to an expected type.
An `Environment` tracks which variables are in scope.
The `Context` supplies class and function signatures.
Each constraint records which side must be the supertype, such as an annotation over the value assigned to it.

## Unification

`constrain/unify` solves the constraints by comparing `Name`s, accepting a subtype where a supertype is expected.
The result maps each `Position` to a `Name`, which is used to annotate the `AST` as `ASTTy`.

## Match exhaustiveness

`constrain/exhaustive.rs` runs after unification, on every match that constraint generation recorded.
It reads each subject's type from the unified `Position` to `Name` map.
A match must cover every value of its subject, and every arm must be reachable.

## Names

`name/` defines type identity.

- `Name`: a set of `TrueName`s, which is a type union.
- `TrueName`: a `NameVariant` plus nullable and mutable flags.
  Mutability is display-only, and is not part of equality or hashing.
- `NameVariant`: a `StringName` (a nominal type), a `Tuple`, or a `Function` of arguments and a return `Name`.

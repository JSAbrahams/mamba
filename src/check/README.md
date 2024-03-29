<p align="center">
    <img src="../../image/logo.svg" height="150" alt="Mamba logo"/>
</p>

# Check

Any stage may encounter an error, which gives a typing error.

## Building A Context

Before doing anything, the type checker first builds a context. This context contains the signature of all functions and classes which are imported (implicitly or explicitly). It does not type-check
them, but does require they are valid signature:

- Functions must have a type for all arguments and return type if it should be treated as an expression.
- Classes must have functions which are all typed, and top-level variables should have a type annotation.

The transpiler has built-in Python classes which have the same signature as the real thing.

All imported mamba files have their classes and functions scanned in the same manner.

## Constraint Generation

When given an `AST`, the type checker, using the above context and an environment, generates constraints. The environment is used to keep track of variables:

- When they are created.
- When they are accessed, to check whether something is defined or not in a certain scope.

The context is used to:

- Check whether a class or function is defined.
- Extract argument and return types where relevant to generate constraints.

For each location, a constraint is generated which links an expression to an expected type. The generation stage also takes into account which should be the super type. E.g. in an assign, the type
annotation on the right should be the super type of the expression on the right.

The final output is a list of constraints.

## Substitution And Unification

The final step is a substitution and unification algorithm.

- Substitution is performed when we try to unify two expressions.
- Unification is performed when we encounter two types. If two types are not equal, which we check by comparing `Name`s, then an error is returned. One may be a super type of the other, in which case
  it is accepted. Which should be the supertype of the two is indicated during the constraint generation stage.

During the unification stage, when a type is encountered, it's position is noted. This end result is a mapping from `Position`s to type `Name`s. This is followed by a mapping stage where the
input `AST`s are converted to typed AST (`ASTTy`). By default, each `ASTTy` has no type. However, if it happens to have a position which has an existent mapping from `Position` to `Name`, then
this `ASTTy` is annotated with `Name`.

## Type Names

Names are at the core of the type checker, and are used to check whether a type is what we expect it to be.

- A `Name` is the interface. It contains a set of `TrueName`s, as it represents a type union.
- A `TrueName` contains two booleans: (1) Whether something is nullable and (2) whether something is mutable. It also contains a `NameVariant`.
- A `NameVariant` may be one of the following:
    - `StringName`, which is the actual name of a type.
    - `Tuple`, which is a tuple of `Name`.
    - `Function`, which is a list of arguments `Name` and a return `Name`.

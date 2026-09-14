<p align="center">
    <img src="../../image/logo.svg" height="150" alt="Mamba logo"/>
</p>

# Check

Any stage may encounter an error, which gives a typing error.

## Building A Context

Before doing anything, the type checker first builds a context.
This context contains the signature of all functions and classes which are imported, implicitly or explicitly.
It does not type-check them.
It does require that they are valid signatures:

- Functions must have a type for all arguments and return type if it should be treated as an expression.
- Classes must have functions which are all typed, and top-level variables should have a type annotation.

The transpiler has built-in Python classes which have the same signature as the real thing.

All imported mamba files have their classes and functions scanned in the same manner.

## Constraint Generation

When given an `AST`, the type checker generates constraints.
It uses the above context and an environment to do so.
The environment is used to keep track of variables:

- When they are created.
- When they are accessed, to check whether something is defined or not in a certain scope.

The context is used to:

- Check whether a class or function is defined.
- Extract argument and return types where relevant to generate constraints.

For each location, a constraint is generated which links an expression to an expected type.
The generation stage also takes into account which should be the super type.
For example, in an assign, the type annotation on the right should be the super type of the expression on the right.

The final output is a list of constraints.

## Substitution And Unification

The final step is a substitution and unification algorithm.

- Substitution is performed when we try to unify two expressions.
- Unification is performed when we encounter two types.
  If two types are not equal, then an error is returned.
  We check that by comparing `Name`s.
  One may be a super type of the other, in which case it is accepted.
  Which should be the supertype of the two is indicated during the constraint generation stage.

During the unification stage, when a type is encountered, its position is noted.
The end result is a mapping from `Position`s to type `Name`s.
This is followed by a mapping stage, where the input `AST`s are converted to typed AST (`ASTTy`).
By default, each `ASTTy` has no type.
However, it may have a position with an existing mapping from `Position` to `Name`.
In that case this `ASTTy` is annotated with `Name`.

## Type Names

Names are at the core of the type checker, and are used to check whether a type is what we expect it to be.

- A `Name` is the interface.
  It contains a set of `TrueName`s, as it represents a type union.
- A `TrueName` contains two booleans.
  These are whether something is nullable, and whether the binding it was read from was `mut`.
  Mutability is deliberately not part of a name's identity, so `PartialEq`, `Hash` and `Ord` skip it and it only affects how the name is displayed.
  It also contains a `NameVariant`.
- A `NameVariant` may be one of the following:
    - `StringName`, which is the actual name of a type.
    - `Tuple`, which is a tuple of `Name`.
    - `Function`, which is a list of arguments `Name` and a return `Name`.

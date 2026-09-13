⬅ [🏠 Home](../../README.md)

⬅ [2 🛠 Features](../README.md)

⬅ [2.5 🔁 Functions](README.md)

# 2.5.1 Pure Functions

A pure function is referentially transparent.
Wherever a call to it appears, that call could be replaced by its result, and the program would behave identically.
Concretely, for a pure function `f`, if `x = y` then `f(x) = f(y)`.

It also relies on the rules the README lists:
a `self` that is not `mut`, no calling impure functions, only reading fields that are not `mut` or calling `pure` methods on arguments.
Those rules exist so that nothing reachable from a pure function's arguments can be mutated out from under it, directly or indirectly.

## Purity and classes

A pure method is allowed, but the rules limit where one is useful.
A class with any mutating method needs `mut` fields, and a pure function may not read a `mut` field.
No method of such a class can therefore be pure.
This is why `pure` is mostly a tool for plain functions, and why the `Matrix2x2` example in the README has no pure methods.
A class whose fields are all immutable has no such problem, and its methods may be pure.

Constructing a class from a pure function needs the class to say that constructing it is pure.
That is a bodiless `def pure new(..)`, where the `..` stands for the class arguments, and it is checked against the derived field initializers.
The list says how many arguments there are: `new()` for none, `new(_)` for exactly one, `new(..)` for one or more.
Purity is never inferred from those initializers, because a guarantee that appears and disappears as unrelated code changes is worse than one that is stated.
See [Class](../modules/class.md#pure-construction).

Note that `def pure new(..)` says nothing about the class's methods.
It sits on `new` precisely so it cannot be read as a claim about them.

Purity says nothing about termination on its own.
A pure function can still loop forever.
That's what `total` is for, covered next.

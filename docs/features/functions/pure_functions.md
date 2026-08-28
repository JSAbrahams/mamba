⬅ [🏠 Home](../../README.md)

⬅ [2 🛠 Features](../README.md)

⬅ [2.5 🔁 Functions](README.md)

# 2.5.1 Pure Functions

A pure function is referentially transparent.
Wherever a call to it appears, that call could be replaced by its result, and the program would behave identically.
Concretely, for a pure function `f`, if `x = y` then `f(x) = f(y)`.

It also relies on the rules the README lists:
`fin self`, no calling impure functions, only reading `fin` fields or calling `pure` methods on arguments.
Those rules exist so that nothing reachable from a pure function's arguments can be mutated out from under it, directly or indirectly.

Purity says nothing about termination on its own.
A pure function can still loop forever.
That's what `total` is for, covered next.

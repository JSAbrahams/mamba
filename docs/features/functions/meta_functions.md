⬅ [🏠 Home](../../README.md)

⬅ [2 🛠 Features](../README.md)

⬅ [2.5 🔁 Functions](README.md)

# 2.5.3 Meta Functions

A meta function's totality is checked syntactically:
Its body is held to the same restrictions as a `total` function, so the compiler never has to run a meta function to know it terminates, only inspect its shape.
This is what makes evaluating `meta` functions at compile time safe, and avoids a circular dependency on the compiler compiling itself.

`meta` is closed to the standard library.
Opening it to user code is future work.
A `measure()` for a custom `Measurable` type (see [Total Functions](total_functions.md)) would be a safe first case: restricted to a straight-line, non-recursive form, it cannot fail to terminate by construction.
General-purpose `meta` functions have no equivalent restriction, since their value lies in running arbitrary compile-time computation.

If the syntactic check on a `meta` function's totality ever has a gap, the failure mode is a hung compile, not a hung program at runtime.

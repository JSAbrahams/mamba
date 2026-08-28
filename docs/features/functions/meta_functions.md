⬅ [🏠 Home](../../README.md)

⬅ [2 🛠 Features](../README.md)

⬅ [2.5 🔁 Functions](README.md)

# 2.5.3 Meta Functions

**A meta function must be total, not merely recommended.**
Proving a function is total normally means compiling and checking it, but the compiler is mid-compile already when it needs to check a `meta` function, which sounds circular.
We resolve this by not proving it at all:
A meta function's body is held to the same four restrictions as `total` functions.
This is a syntactic well-formedness rule, not an evaluation.
The checker never has to *run* a meta function to know it terminates, any more than a borrow checker has to run a program to know it doesn't alias.
That sidesteps the meta-compiler regress entirely.

**Should users be able to write general-purpose `meta` functions at all?**
For now, we think not, or only in a closed form:
Restrict `meta` to `@builtin`-gated definitions that ship with the standard library, reviewed by us, small in number, rather than something any Mamba user can write.
The `Measurable`/`measure()` case discussed in [Total Functions](total_functions.md) is the exception, because we found a restriction, straight-line, no recursion at all, that's safe to open up precisely because it removes recursion from the picture entirely.
General-purpose `meta` has no equivalent escape hatch:
Its whole value is running arbitrary compile-time computation, so the best we can offer a user-authored `meta` function is "restricted to structural recursion, checked", which is weaker than "cannot possibly fail to terminate".
Given that `meta` is already a niche, mostly-stdlib feature, the risk/benefit favours keeping it closed until there's a concrete case for opening it.

**Worth stating explicitly: even with the structural-recursion restriction, suppose the checker's syntactic rule is ever wrong**, and it lets through something that shouldn't have been accepted.
The failure mode is the compiler hangs evaluating a `meta` function during `total`-checking.
That is a strictly preferable failure to the one `total` exists to prevent.
A hung compile happens on a developer's own machine or in CI, is attributable to a specific function, is interruptible, and blocks the broken code from ever being shipped.
A hung `total` function at runtime is the exact production failure, denial of service, a stuck request thread, resource exhaustion, that the whole feature was built to rule out, except now dressed up with a false badge of having been proven safe.
So while the goal is to make compile-time non-termination impossible by construction, if we ever have to choose between an imperfect static check that occasionally hangs the compiler and a looser one that occasionally ships a non-terminating `total` function, we should choose the former without hesitation, and pair it with a recursion/step budget during meta evaluation (`error: meta evaluation exceeded N steps`), so the failure shows up as a diagnostic rather than a silent freeze.

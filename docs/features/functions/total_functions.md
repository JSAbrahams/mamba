⬅ [🏠 Home](../../README.md)

⬅ [2 🛠 Features](../README.md)

⬅ [2.5 🔁 Functions](README.md)

# 2.5.2 Total Functions

## Ackermann's function

Ackermann's function halts for every input but cannot be marked `total` in Mamba:

```mamba
# Nat is future work: typing `m - 1` as Nat needs interval refinement, which uses the failed
# guard of the arm above to know that m is at least 1.
# See [Interval refinement](../safety/types.md#interval-refinement)
def ackermann(m: Nat, n: Nat) -> Nat := match (m, n) where
    (m, n) if m = 0 => n + 1
    (m, n) if n = 0 => ackermann(m - 1, 1)
    (m, n)          => ackermann(m - 1, ackermann(m, n - 1))
end
```

```
A(m, n) =
    n + 1                  if m = 0
    A(m - 1, 1)            if m > 0 and n = 0
    A(m - 1, A(m, n - 1))  if m > 0 and n > 0
```

It's the standard example of a total, computable function that is not primitive recursive: its recursion depth isn't bounded in advance by a single value that strictly decreases toward a base case, which is exactly the class `total` accepts.
A `for` loop over a `SizedIterator` is primitive recursive this way (its iteration count is fixed by the collection's size), and so is structural descent on a single `Measurable` value.

- `A(m, n - 1)` decreases only `n`.
- `A(m - 1, A(m, n - 1))` decreases `m`, but its second argument is whatever the inner call returns, which can be far larger than the original `n`.

Ackermann's termination is provable, just not by a flat measure.
Comparing `(m, n)` lexicographically, with `m` dominant, works: the outer call always decreases `m`, regardless of how large its second argument becomes.
`measure()` returns a single scalar, though, and `decreases` is a single comparison (`self.measure() < other.measure()`), so there's no way to fold a lexicographic pair into it.

Other systems accept this shape directly: ACL2 by measuring into the ordinals below `ε₀`, and structural checkers built on the size-change principle (Lee, Jones, Ben-Amram, *The Size-Change Principle for Program Termination*, POPL 2001), including Agda's, by tracking that some combination of argument positions decreases along every call path rather than one designated one.
Generalising `Measurable` from a flat scalar to a lexicographic tuple, an ordinal, or a multi-argument call-graph analysis is future work.

## Partial subtraction in `StrictlyDecreases`

Measures land in `Nat`, the non-negative integers.
Zero is in the set, which is what makes the codomain bounded below.
A measure needs that to serve as a termination argument.
See [Types](../safety/types.md#nat) for `Nat` itself, which is future work.

`Nat` is closed under addition but not under subtraction.
That makes `StrictlyDecreases.subtract` a partial operation, and its signature says so:

```mamba
def meta subtract(self, other: Self) -> Nat? := self.measure() - other.measure()
```

When `other.measure()` exceeds `self.measure()`, the difference has no representative in `Nat`.
There is no value to return, so the result is `None`.

The other tempting option is to saturate at zero, in the style of a truncated subtraction.
However, `None` and `0` carry different claims.
`0` says the two measures were equal, which is a real answer and is for all intens and purposes a silent failure.
The latter is something we wish to avoid at all costs in the pursuit of correctness.
`None` says the subtraction left the domain.

Partiality costs nothing in decidability.
Which case holds is settled by `self.measure() >= other.measure()`.
Because `subtract` is `meta`, both the arithmetic and that decision happen at compile time.
A measure comparison that leaves the domain is caught at the definition site, not at runtime.

## `Measurable` and custom types

`Measurable` cannot currently be implemented for custom types.
Doing so safely needs two things.
First, `measure()` must be total, deterministic, and pure, into a well-founded (bounded-below) codomain such as `Nat`.
Second, the compiler must re-verify the decrease at each call site.
It cannot trust a type's `Measurable` implementation on its own.
Given both, `measure()`'s specific logic doesn't matter for soundness, only that it's such a mapping.

This is the same model Coq, Agda, Lean, and ACL2 already use for user-defined well-founded recursion:
Any type, any well-founded relation, with a fresh termination proof discharged at each use rather than trusted from a declaration.

Re-verifying the decrease at every call site is only decidable because `Measurable` is restricted to `Add`, `Sub`, `Eq`, `Comparable` (Presburger arithmetic), which is decidable.
Adding `Mul` between two non-constant values would reintroduce undecidability, so any future relaxation of `Measurable` has to treat multiplication as a hard boundary, not a convenience.
A `measure()` that calls another pure function must also be non-recursive, so the whole call chain can be unfolded and checked at the definition site.

Opening `Measurable` to custom types is future work.

**Further reading:**

- M. Presburger, *Über die Vollständigkeit eines gewissen Systems der Arithmetik ganzer Zahlen*, 1929.
- Y. Matiyasevich, *Enumerable Sets are Diophantine*, 1970.
- C. S. Lee, N. D. Jones, A. M. Ben-Amram, *The Size-Change Principle for Program Termination*, POPL 2001.
- B. Nordström, *Terminating General Recursion*, BIT 28, 1988.
- A. Bove, V. Capretta, *Modelling General Recursion in Type Theory*, Nordic Journal of Computing 12, 2005.
- M. Kaufmann, P. Manolios, J S. Moore, *Computer-Aided Reasoning: An Approach*, 2000.

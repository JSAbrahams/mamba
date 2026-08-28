⬅ [🏠 Home](../../README.md)

⬅ [2 🛠 Features](../README.md)

⬅ [2.5 🔁 Functions](README.md)

# 2.5.2 Total Functions

## Ackermann's function, in full

The README shows this function as an example of something that halts but can never be marked `total`:

```mamba
# some syntax here such as guard arms which are not in the language yet
def ackermann(m: PosInt, n: PosInt) -> PosInt := match (m, n) with
    (m, n) if m = 0 => n + 1
    (m, n) if n = 0 => ackermann(m - 1, 1)
    (m, n)          => ackermann(m - 1, ackermann(m, n - 1))
end
```

This is the classic Ackermann function:

```
A(m, n) =
    n + 1                  if m = 0
    A(m - 1, 1)            if m > 0 and n = 0
    A(m - 1, A(m, n - 1))  if m > 0 and n > 0
```

It halts on every `(m, n)`, and its definition looks like an entirely ordinary pair of recursive calls.
But it is the textbook example of a total, computable function that is **not primitive recursive**.

Primitive recursion means the depth of the recursion is bounded in advance by a single value that strictly decreases toward a base case.
A `for` loop over a `SizedIterator` is primitive recursive:
Its iteration count is fixed by the collection's size before the loop starts.
Structural descent on a single `Measurable` value is primitive recursive too:
Its number of recursive calls is bounded by the starting measure.
This is exactly the expressive class `total` restricts you to.

Concretely, our `StrictlyDecreases`/`Measurable` scheme cannot accept `A`:

- The call `A(m, n - 1)` doesn't decrease `m` at all, only `n`.
- The outer call `A(m - 1, A(m, n - 1))` decreases `m`, but its second argument is whatever the inner call returns.
  This can be vastly larger than the original `n`.

That does not mean `A`'s termination is unprovable.
Compare `(m, n)` lexicographically, with `m` as the dominant component:
`A(m, n - 1)` decreases in the second slot, and `A(m - 1, A(m, n - 1))` decreases in the first slot no matter how large the second slot becomes.
That is a genuine well-founded order.
Its order type is `ω²` (omega squared), an ordinal number describing "pairs of naturals, compared by the first component first".
So Ackermann's termination is a completely ordinary fact, provable by ordinary nested induction.
It is just not provable by the *specific* technique Mamba's `total` checker uses.

What `A` lacks is a *flat* measure, which is what Mamba's scheme requires.
`measure()` returns one comparable value, and `decreases` is one comparison:
`self.measure() < other.measure()`.
There is no way to fold `(m, n)` into a single number so that "the pair got lexicographically smaller" becomes "the folded number got smaller", because `n` can grow without any fixed bound in the very same step that `m` shrinks by one.
No finite weighting of `m` and `n` into a single scalar survives that.

So a function shaped like Ackermann's can never be marked `total` in Mamba today, not because no well-founded measure exists for it, but because Mamba's well-founded order is deliberately flat, and Ackermann needs a lexicographic (or ordinal) one.
Other systems that support richer measures accept it directly:

- ACL2 admits Ackermann-shaped definitions using measures into the ordinals below `ε₀` (epsilon-nought) instead of plain naturals.
- Structural checkers built on multi-path call-graph analysis, the *size-change principle* (Lee, Jones, Ben-Amram, *The Size-Change Principle for Program Termination*, POPL 2001), accept the ordinary two-clause Ackermann definition too, by tracking that some combination of argument positions decreases along every call path, rather than one designated one.
  Agda's termination checker works this way.

Generalising `Measurable` from a flat scalar to a lexicographic tuple, an ordinal, or a multi-argument call-graph analysis is a real, addressable extension.
We don't implement any of that today, but it's worth keeping on the table, and it connects directly to the next question.

## Opening `Measurable` to custom types

`Measurable` is currently a special built-in trait that cannot be implemented for custom types.
The README argues this restriction can be relaxed.
Here is the reasoning behind that in full, since it's worth getting right rather than asserting.

**Does opening `Measurable` up to custom types open a Pandora's box?**
Not automatically.
Whether it's safe comes down to exactly two properties.
Neither of them is "does the user's `measure()` look honest".

1. `measure()` must be a total, deterministic, pure function into a well-founded codomain.
   Well-founded means the codomain has no infinite descending chain.
   `PosInt`/`Nat` qualifies.
   `Int` does not:
   `5, 4, 3, ..., -1, -2, ...` never bottoms out, so `Measurable for Int` needs to measure into `PosInt` (e.g. `self.abs()`), not return `self` directly.
2. The compiler must independently re-check the decrease at every call site.
   It cannot just check that a type implements `Measurable` and stop there.

The second point is the one that actually matters, and it is easy to get backwards.
`measure()`'s *meaning* is irrelevant to soundness.
A user can write `measure()` as `self.weird_field - 7` for a type with no obvious notion of "size", and the scheme stays perfectly sound, as long as the compiler substitutes the concrete call-site expressions into `measure()` and mechanically checks the resulting inequality on every recursive edge, every time.
What would actually break soundness is trusting the existence of a `Measurable` implementation as a blanket permission, without redoing that check per call site.
That is the real Pandora's box:
Not user-defined measures, but a compiler that stops verifying once a trait box is ticked.

This is not a novel design.
It is precisely how mainstream deductive systems already let users define arbitrary well-founded relations on arbitrary types:

- Nordström's account of terminating general recursion in Martin-Löf type theory (Nordström, *Terminating General Recursion*, BIT 28, 1988), and Bove and Capretta's method for modelling general recursion via an inductively defined domain predicate (Bove & Capretta, *Modelling General Recursion in Type Theory*, Nordic Journal of Computing 12, 2005), both let a user supply an arbitrary well-founded relation on an arbitrary type.
  Every recursive call still needs its own decrease proof.
- Coq's `Fix`/`well_founded_induction`, Agda's `Induction.WellFounded` module, and Lean 4's `termination_by`/`decreasing_by` all work the same way.
  Pick any type.
  Pick any well-founded relation on it.
  The tool discharges a fresh proof obligation for every recursive call, usually via an automatic arithmetic decision procedure, Lean's `omega` tactic, for instance.
- ACL2 requires every recursive definition to carry a `:measure`, which can be an arbitrary term into the ordinals below `ε₀`, checked automatically by ACL2's own arithmetic and rewriting engine.
  Nothing about the measure's shape is restricted beyond "well-founded, and the prover can actually discharge the resulting inequality".

None of these systems restrict well-founded relations to some closed set of "primitive" types.
They restrict what has to be *proved* about a user-supplied relation, at the point it is used.
That is the model worth copying:
`Measurable` open to any type, `measure()` open to any pure logic, every recursive call in a `total` function re-verified, not merely trait-gated.

**Re-verifying "does the measure decrease" is itself a decision problem, though.**
This is where the straight-line restriction on `measure()`, no recursion, no loops, matters for a reason beyond "a stray `measure()` can't fail to terminate when evaluated once".
Proving the decrease for *every possible input*, not just one instance, is a decidability question in its own right, and the answer depends on what arithmetic `measure()` is allowed to use.

- Restricted to `Add`, `Sub`, `Eq`, `Comparable`, which is exactly `Measurable`'s current bound, the resulting inequality lives in Presburger arithmetic:
  Linear arithmetic over integers, no multiplication of two non-constant terms.
  Presburger arithmetic is decidable (Presburger, 1929), even though the worst case is expensive (Fischer and Rabin proved a double-exponential lower bound in 1974).
  This is also the fragment SMT-based verifiers lean on for their own `decreases` clauses (Dafny, F*, Lean's `omega`).
- Add `Mul` between two non-constant `Measurable` values, and this guarantee breaks.
  General Diophantine reasoning, arithmetic with unrestricted multiplication, is undecidable.
  This is Hilbert's tenth problem, resolved negatively by Matiyasevich in 1970, building on Davis, Putnam, and Robinson's earlier work.
  There is no algorithm that decides, in general, whether an arbitrary polynomial equation over the integers has an integer solution, and the same wall shows up here.

So `Measurable`'s existing trait bound, `Add, Sub, Eq, Comparable`, deliberately excluding `Mul`, is not arbitrary minimalism.
It is exactly the boundary that keeps "does this measure decrease" a decidable question.
Any future relaxation of `Measurable` needs to treat adding `Mul` as a real decidability boundary, not a convenience feature.

One more requirement, easy to miss.
If `measure()` calls another pure function, the `Str` example in the README calls `.len()`, that function must also be non-recursive.
For the compiler's check to go through, it must be fully unfoldable into the same `Add`/`Sub`/`Eq`/`Comparable` fragment, or treated as an opaque, axiomatically trusted primitive, the way a compiler-provided `len()` can be.
This is a whole-program property, the transitive closure of everything `measure()` reaches, not just a property of `measure()`'s own body.
It is checkable the same way though:
Syntactically, at the definition site, with no evaluation required.

**Conclusion.** `Measurable` does not need to stay closed to built-in types as a matter of correctness.
The correctness requirement is narrower, and already well understood in the literature above:
A total, deterministic, pure `measure()` into a well-founded, `Mul`-free arithmetic fragment, with every recursive call re-verified individually rather than trusted from the trait's existence.
Closing `Measurable` to built-in types only is a fine *simplicity* choice for an early version of the language.
It should not be sold as a *soundness* one.

**Further reading**, on the pieces above:

- M. Presburger, *Über die Vollständigkeit eines gewissen Systems der Arithmetik ganzer Zahlen*, 1929.
  The original decidability result for linear integer arithmetic.
- M. J. Fischer, M. O. Rabin, *Super-Exponential Complexity of Presburger Arithmetic*, 1974.
  The cost of that decidability, in the worst case.
- Y. Matiyasevich, *Enumerable Sets are Diophantine*, 1970.
  The negative resolution of Hilbert's tenth problem, why unrestricted multiplication breaks decidability.
- C. S. Lee, N. D. Jones, A. M. Ben-Amram, *The Size-Change Principle for Program Termination*, POPL 2001.
  A decidable, fully automatic method for proving termination via call-graph decrease analysis, close in spirit to Mamba's own call-tree rule.
- B. Nordström, *Terminating General Recursion*, BIT 28, 1988.
- A. Bove, V. Capretta, *Modelling General Recursion in Type Theory*, Nordic Journal of Computing 12, 2005.
- M. Kaufmann, P. Manolios, J S. Moore, *Computer-Aided Reasoning: An Approach*, 2000.
  ACL2's ordinal-based `:measure` mechanism.

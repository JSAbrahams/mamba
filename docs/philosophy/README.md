⬅ [🏠 Home](../README.md)

# 1 💭 Philosophy of the Language

## 📄 Contents

### [1.1 Inspirations of the Language](inspiration.md)
### [1.2 Readability](readability.md)
### [1.3 Imperative Versus Functional, or, Idealism versus Pragmatism](imperative_vs_functional.md)
### [1.4 Flexibility](flexibility.md)
### [1.5 Safety](safety.md)

This page states what Mamba is for and what its central idea is.
The pages above go into detail on individual themes.
[Inspirations](inspiration.md) covers the languages Mamba borrows from, and where it parts company with each.

## Why Mamba exists

Mamba is a toy language.
It is an experiment, written for the pleasure and the education of writing it.
It is not competing for anybody's production workload.

Most languages inherit their notation from C, and C inherited the constraints of 1972.
Square brackets index arrays because an array was a memory offset.
`=` assigns and `==` compares because assignment was the common case.
`&&` is punctuation because terse punctuation was cheap to lex.
None of that is wrong, and none of the reasons still apply.

Mathematical notation was refined over much longer, for a different purpose: being read and reasoned about by a human with a pencil.
Mamba asks what a language looks like with that as the reference instead of C.

## The one central idea: a mapping is a mapping

In mathematics several things are the same kind of object.
A function `f: A → B` sends each element of `A` to one of `B`.
A sequence `(a_n)` sends each index to a value.
An indexed family does the same over any index set.
The notation says so, since `a_n` is application wearing a subscript.

Programming languages split them apart.
You write `f(x)` for a function, `a[n]` for an array, `m[k]` for a dictionary.
That split is an implementation detail wearing a syntax: an offset, a hash lookup and a jump each got their own bracket.

Mamba collapses it:

```mamba
def f(x: Int) -> Int := x + 1
def a := [10, 20, 30]
def m := { "hello" => 1, "world" => 2 }

print(f(2))        # 3,  a function applied
print(a(1))        # 20, a list applied
print(m("hello"))  # 1,  a mapping applied
```

A list is a mapping whose domain happens to be `0, 1, ..., n`.

The cost is that square brackets are what a C-family programmer reaches for, so this will surprise them.
`[` and `]` are spent on list literals and generics instead.

The point is substitutability.
Written the same way, a function and a table can replace one another without touching a call site.
That allows memoising a slow function by swapping in a table, or replacing a large table with a computed function.

Scala got here first, through its `apply` convention, where a `Map[A, B]` is a `Function1[A, B]`.
The difference is that in Mamba this is the default reading of the notation, not something a type opts into.

Today the unification is only notational.
See [what is built](#what-is-built-and-what-is-not) below.

## The rest of the notation

### Definition and equality are different things

`:=` defines or assigns.
`=` asks whether two things are equal.

```mamba
def x := 2
if x = 2 then print("yes")
```

This is how mathematics reads, and how Algol and Pascal read.
The `=` versus `==` confusion cannot arise, because `=` never assigns.
It also frees `=` to mean structural equality, which is the mathematical meaning.
Identity gets its own word, `is`.

### Words instead of punctuation

`and`, `or`, `not`, `mod`, `in`.
I want the operator to be pronounceable.

```mamba
if alice.is_online() and not bob.is_away() then print("chat")
```

Punctuation stays where mathematics itself uses it, so arithmetic remains symbolic and `^` is exponentiation.

### Sets and sequences are distinguished in the syntax

A `do ... end` block is a sequence.
Order matters.

A `where ... end` block is a set.
Order is not part of the meaning.

```mamba
class Point(x: Int, y: Int) where       # a set, so these are unordered
    def norm_squared(self) -> Int := self.x ^ 2 + self.y ^ 2
end

def main() := do                        # a sequence, so these are ordered
    print("first")
    print("second")
end
```

A class body is a set, because the order of two method declarations carries no information.
A `match` is a set of cases, which is why it reads `match x where ... end`.

The consequence is a future one.
A block declared to have no order can be evaluated in any order, including in parallel.
That would make parallelism a property of the notation rather than a library.
The idea is still in its infancy.

### Comprehensions are set-builder notation

```mamba
def evens := { x | x in numbers, x mod 2 = 0 }
def squares := [x ^ 2 | x in numbers]
```

This is `{ x | x ∈ N, x ≡ 0 mod 2 }` with the symbols spelled out.
Haskell and Python have comprehensions too, so this is common ground.
What differs is that sets, lists and maps all have literal and builder syntax in the grammar, rather than sets being a library type.

## Gradual rigour

Verification tools usually ask for a global commitment.
The discipline is the entry fee, paid before the first useful line.

Mamba tries the opposite.
Ordinary code stays ordinary: mutable, effectful, eager, imperative.
Rigour is opt-in, one definition at a time:

Tier | Keyword | Promise | Checked by
---|---|---|---
Ordinary | none | Types line up. Nothing is null unless it says `?`. Errors are handled or declared. | Type checker
Pure | `pure` | Same input, same output, no side effects. | Restrictions on the body
Total | `total` | Terminates on every input in its domain. | A decreasing measure, per call site
Compile time | `meta` | Evaluated before the program runs. | The `total` restrictions, checked syntactically
Refined | `type ... when` | The value satisfies a stated predicate. | Checked at construction and cast

The tiers stack, since a `meta` function is total, and a total function is of little use impure.

The appeal is that the cost is local.
Write an ordinary script, then decide one function deserves a termination proof, and pay only there.
This is to effects and termination roughly what gradual typing is to types.

Termination is where the mathematics is most explicit.
Mamba does not try to decide halting.
Each argument must map into something numeric and bounded below, via a `measure`, and that measure must strictly decrease on recursive calls.
Termination reduces to arithmetic, which is why these docs call Peano arithmetic the logical bedrock of the system.

The trade-off is that `total` accepts a fixed, mechanically checkable subset, and rejects functions that obviously halt but do not fit.
Ackermann's function is the standing example.
That is the same bargain `const fn` makes in Rust.

## What is built, and what is not

Working today:

- Static typing with inference, null safety with `?`, on-site error handling with `!` and `where`.
- Mutability distinction with `fin`, and traits.
- Round-bracket application for functions, lists and maps.
- `=` as structural equality, `:=` as definition, word operators, `^` and `mod`, ranges, slices.
- Set, list and map literals, and single-variable comprehensions.
- `do ... end` sequences and `where ... end` sets.
- Two backends, Python and an experimental native one via Cranelift.

Notation not yet backed by semantics:

- **A list is not usable as a function.**
  The type checker rejects a `List[Int]` where a `Callable[[Int], Int]` is expected.
  This is the largest gap, since it is the central idea only half kept.
- **Functions are not first class.**
  A lambda works passed directly as an argument, but not bound to a variable and called.

Declared but not implemented:

- `total` and `meta` are in the grammar and the docs, and are rejected by the parser.
- `pure` parses and is tracked, but none of its restrictions are enforced.
- Type refinement, the `type ... when` construct, is entirely aspirational.
- Trait generics parse, but external implementation, `def <Trait> for <Class>`, does not.
- Comprehensions binding more than one variable do not resolve.

`tests/` is the record.
Each unimplemented feature has a case with an `ignore[...]` reason, so the gap is inventoried rather than forgotten.

## Future directions

Ordered by how much each would sharpen the language's identity.

1. **Make the mapping principle real.**
   Give lists, maps and functions a common callable interface in the type system.
   This turns the central idea from notation into semantics.
2. **First-class functions.**
   Lambdas bound to variables, functions as values, composition.
   Without these, the first item has nothing to be interchangeable with.
3. **Enforce `pure`.**
   The tier exists in name only until the restrictions bite, and it is the cheapest one to enforce.
4. **Implement `total`, then `meta`.**
   `Measurable` and the decreasing-measure check are already specified in some detail.
5. **Refinement types with a solver.**
   `type EvenNum: Int when self mod 2 = 0` is the feature most likely to make Mamba useful, and an SMT backend is the realistic way to check it.
6. **Turn code sets into parallelism.**
   The syntax already declares that order does not matter.
7. **Guards in match cases.**
   Piecewise definition is how mathematics defines functions, and side conditions are half of that notation.
   The README's Ackermann example is written in a syntax the parser does not accept.
8. **Grow the native backend.**
   Currently a small slice of the language.

## Non-goals

- Performance.
  The Python backend makes that somebody else's problem.
- Replacing Python.
  Mamba compiles to it and is happy alongside it.
- Purity as a global discipline.
  Haskell is at the end of that road.
- Deciding halting in general.
  `total` is a checkable subset and will stay one.
- Dependent types.
  Refinement plus a solver gets most of the practical benefit for a fraction of the difficulty.
- A large standard library.

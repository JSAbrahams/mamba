⬅ [🏠 Home](../README.md)

⬅ [1 💭 Philosophy of the Language](README.md)

# 1.1 Inspirations of the Language

Languages that influenced Mamba, whether through a construct, a keyword, or a whole philosophy.
This is based on personal experience, so a feature credited to one language often exists in others too.
It only means that is where I met it first.

Language  | Inspired
----------|------------
Python    | Flexibility. Co-existence of functional and OOP paradigms. Large portions of the syntax.
Java      | OOP concepts. Static typing rules.
C#        | OOP concepts. Static typing rules and type inference.
Scala     | That everything is an object, including primitives of the language. Pattern Matching.
Kotlin    | Ranges baked into the language. Type aliases. Covariance and Contravariance using `in` and `out`.
Ada       | Custom data types (or type aliases) with ranges, which also partially inspired the type refinement system. Use of natural language over symbols, such as using `and` as opposed to `&&`.
C++       | OOP concepts and operator overloading.
C         | General programming concepts. Not so much a direct inspiration but more a general influence.
Eiffel    | The design by contract philosophy and the `retry` keyword. Design by contract also inspired the type refinement system.
Haskell   | Pattern Matching. Immutability. Closeness of mapping with mathematical notation, for instance set constructor notation.
Rust      | Error handling mechanisms and strict rules regarding mutability.
Ruby      | Portions of the syntax of the language.
Swift     | Elegant error handling mechanisms.
Go        | Error handling mechanisms, which encourage error handling on site.
MATLAB    | Small portions of the syntax.
Smalltalk | OOP concepts, with a large emphasis on program state.
JavaScript| Interchangeability of variables and functions, and a reliance on higher-order functions.
Perl      | The `forward` keyword.

## Python

The closest relative, and the host ecosystem.
Mamba transpiles to Python, so interoperability is the point rather than an afterthought.
It keeps Python's readability and low ceremony, and changes the rest:

- Types are static and checked, with inference so you rarely write them.
- Nothing may be `None` unless its type says `?`.
- Errors are handled at the call that failed, with `!` and `where`, not in a `try` around a region.
- Nothing may change unless its binding says `mut`.
- Traits replace inheritance.
- Indexing is `a(0)`, not `a[0]`.
- `{ ... }` is a set.
  A dictionary needs `=>` pairs.

The last two will trip up a Python programmer.
Both follow from [the mapping principle](README.md#the-one-central-idea-a-mapping-is-a-mapping).

## Haskell

Mamba takes comprehension notation, a liking for immutability, and the instinct that mathematical notation is worth chasing.
It is not a pure functional language, and the differences are structural:

- Haskell makes purity the default and tracks effects in the type system.
  Mamba makes effects the default and purity an annotation.
  This is the biggest divergence.
- Haskell is lazy.
  Mamba is eager.
- Haskell is Turing complete throughout, by choice.
  `total` is an attempt at a termination check, which Haskell does not have.
- Haskell threads failure through `Maybe` and `Either`.
  Mamba declines that route on purpose, wanting failure handled next to the call.
- Haskell has type classes.
  Mamba has traits, which are close cousins.

A Haskell programmer will find Mamba insufficiently principled.
That is fair.
Mamba asks how much rigour an imperative language can absorb, which is the opposite question.

## Scala

Scala got to the mapping idea first.
It unifies application and indexing through `apply`, and there a `Map[A, B]` really is a `Function1[A, B]`.
It also supplies "everything is an object" and pattern matching.
The difference is size.
Scala is large and expressive, where I am keeping Mamba small.

## Rust

Traits instead of inheritance, strictness about mutability, and `!` on a fallible call.
Two differences stand out:

- Rust is immutable by default and annotates with `mut`.
  Mamba does the same.
- Rust's `Result` with `?` is the monadic approach Mamba declined.

## Kotlin

Null safety built into the type system rather than bolted on as an `Option`, using `?` and `?.`.
Ranges as a language feature come from here too.

## Ada, and SPARK

Ada gives two things.
Word operators over punctuation is the visible one.
Subtypes carrying their own constraints is the deeper one, and the direct ancestor of type refinement.

SPARK is the closest model for where Mamba's rigour could end up.
The approaches differ: SPARK restricts the language and verifies what remains, while Mamba leaves the language alone and lets single definitions escalate.

## Coq, Lean and Agda

Where `total` and `meta` ultimately point.
The distance is large:

- They have dependent types.
  Mamba has refinement types, which are weaker and far more automatable.
- They require totality everywhere.
  Mamba makes it per-function.
- They emit proof objects that can be checked independently.
  Mamba emits a compiler diagnostic.

Lean 4 matters most here, because it is also a general purpose language.
It shows the two can live together.
If Mamba ever grows a real proof story, the target is not Coq.
It is the refinement-plus-solver approach of Dafny, F\* and Liquid Haskell, where an SMT solver discharges the obligations.

## R, Julia, APL and SETL

R shows what a language shaped by a mathematical domain looks like, instead of one shaped by systems programming.
Julia shows that mathematical notation can come first without the result being a toy.
APL is the extreme case, and Iverson's phrase for it, notation as a tool of thought, is the idea I am borrowing.
SETL built a whole language on set theory in the 1960s, and is the honest ancestor of Mamba's comprehensions.

The lesson is that notation shaped by the domain can be worth the unfamiliarity.
The caution, from APL, is that it can go much too far.

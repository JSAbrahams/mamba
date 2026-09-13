<link rel="shortcut icon" type="image/x-icon" href="image/logo.ico">

<p align="center">
    <img src="image/logo.svg" style="height:200px;" alt="Mamba logo"/>
    <br/><br/>
    <a href="https://github.com/JSAbrahams/mamba/actions/workflows/test.yml">
    <img src="https://img.shields.io/github/actions/workflow/status/JSAbrahams/Mamba/test.yml?style=for-the-badge" alt="GitHub Workflow Status">
    </a>
    <a href="https://app.codecov.io/gh/JSAbrahams/mamba/">
    <img src="https://img.shields.io/codecov/c/github/JSAbrahams/mamba?style=for-the-badge" alt="Codecov coverage">  
    </a>
    <a href="https://crates.io/crates/mamba">
    <img src="https://img.shields.io/crates/v/mamba?style=for-the-badge" alt="Crate">  
    </a>
    <br/>
    <a href="https://github.com/JSAbrahams/mamba/blob/main/LICENSE">
    <img src="https://img.shields.io/github/license/JSAbrahams/mamba.svg?style=for-the-badge" alt="License"/>
    </a>
    <a href="https://github.com/JSAbrahams/mamba/milestones">
    <img src="https://img.shields.io/github/milestones/open/JSAbrahams/mamba?style=for-the-badge" alt="Active milestones"/>
    </a>
    <img src="https://img.shields.io/badge/Built%20with-%E2%99%A5-red.svg?style=for-the-badge" alt="Built with Love"/>
</p>

<h1 align="center">Mamba</h1>

This is the Mamba programming language.
Mamba is similar to Python, but with a few key features:

- Strict static typing rules, but with type inference so it doesn't get in the way too much
- Null safety
- Explicit error handling
- A distinction between mutability and immutability
- Pure functions, or, functions without side effects
- Meta functions, for reasoning about the language itself

See [docs](docs/) for a more extensive overview of the language philosophy.

This is a transpiler, written in [Rust](https://www.rust-lang.org/), which converts Mamba source files to Python source files.
There therefore exists some interoperability with Python code.
Python is the default (and by far the most complete) output; there is also an experimental backend which compiles a small subset of the language straight to machine code, see [Machine Output](#-machine-output) below.

This README:

- Gives a quickstart for developers
- Gives a short overview of the syntax and language features in quick succession, as well as the occasional reasoning behind them.

## 🧑‍💻 Quickstart for developers 👨‍💻

The quickest way to get a complete environment is [Devbox](https://www.jetify.com/devbox).
It sets up all the tooling for you.
That means the pinned Rust toolchain, the Python the test suite needs, and the cargo helpers the git hooks call.
It also gives you nushell and starship.
Everything is declared in [`devbox.json`](./devbox.json) and pinned in `devbox.lock`.
So every contributor gets byte-identical versions.
CI runs this same environment, so what passes locally is what passes in CI.

**We recommend developing on a Unix-like system, meaning Linux or macOS.**
That is what gives you Nix, and therefore Devbox, and therefore that alignment.
On Windows, use [WSL](https://learn.microsoft.com/windows/wsl/install) and follow the Linux instructions inside it.
Developing on plain Windows is supported on a best-effort basis: the test suite does run there in CI, but you install the toolchain yourself, and the more niche corners are likelier to differ.
See [CONTRIBUTING.md](./CONTRIBUTING.md) for the details.

Devbox is a thin layer over the [Nix](https://nixos.org/) package manager.
This means **Nix needs to be installed first**.
Devbox will offer to install it for you on first run.
Installing it yourself up front is the smoother path:

```sh
# 1. Install Nix, in case you do not have it
sh <(curl --proto '=https' --tlsv1.2 -L https://nixos.org/nix/install) --daemon
```

Note that no experimental features need enabling, unlike the Nix flake this replaces.
Devbox does not use flakes.

```sh
# 2. Install Devbox
curl -fsSL https://get.jetify.com/devbox | bash
```

The installer places a single `devbox` binary in `/usr/local/bin`.
It therefore asks for `sudo`.
Run it as your normal user, not as root.

```sh
# 3. Start the environment, with nushell and starship set up already
devbox shell
```

The first `devbox shell` takes a while, as it downloads every pinned package.
Afterwards it is near-instant.
Entering the shell also points `git` at the project's hooks (`.githooks`).
You therefore get the pre-commit checks automatically.

To run a one-off command without entering the shell, use `devbox run`:

```sh
devbox run build          # cargo build
devbox run test           # cargo test --package mamba
devbox run lint           # cargo fmt --check, clippy, cargo sort --check
devbox run precommit      # everything the pre-commit hook runs
```

A more minimal setup, to just get started:

```sh
# Install rustup (if you don't have it already), which is the rust toolchain manager
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Alternatively, check out <https://www.rust-lang.org/tools/install>, e.g. if on a system with no `curl`.

To get more elaboration, see the tooling documentation in [CONTRIBUTING.md](./CONTRIBUTING.md).

## ⌨️ Code Examples

Below are some code examples to showcase the features of Mamba.

### ➕ Functions

We can write a simple script that computes the factorial of a value given by the user.

```mamba
# Factorial of x
def pure factorial(x: Int) -> Int := match x where
    0 => 1
    n => n * factorial(n - 1)
end

def num := input("Compute factorial: ")
if num.is_digit() then do
    def result := factorial(Int(num))
    print("Factorial {num} is: {result}.")
end else
    print("Input was not an integer.")
```

We specify the type of argument `x`, in this case an `Int`, by writing `x: Int`.
This is part of the signature of the function, and is required (it cannot be inferred).
This means that the compiler will check for us that factorial is only used with integers as argument.
`Int` is unbounded by design, with no width and no wrapping.
See [docs/features/safety/types.md](docs/features/safety/types.md#unbounded-integers).
Also note that:

- Code blocks are denoted using `do` and `end` because this is a list of statements and expressions that gets executed _in order_.
- For a match expression or statement we denote cases starting with `where` and ending with `end`, as this is a _set_ of cases which we match on.
  You can read `match x where ... end`, where we read this as "match `x` on this set of conditions in `where ... end`".

_Note_ One could use [dynamic programming](https://en.wikipedia.org/wiki/Dynamic_programming) in the above example so that we consume less memory:

```mamba
def pure factorial(x: Int) -> Int := match x where
    0 => 1
    n => do
        def mut ans := 1
        for i in 1 ..= n do ans := ans * i end
        ans
    end
end
```

### 🍡 Collections

In Mamba, sets, lists, and maps are first class citizens.
They are baked into the language, including its grammar.

Lists make use of square brackets:

```mamba
# lists
def a := [0, 2, 51]
def b := ["list", "of", "strings"]
def empty_list := []
# lists of tuples, builder syntax
def ab := [(x, y) | x in a, x > 0, y in b, b != "of" ]

# Indexing is done using round brackets!
print(a(0)) # prints '0'
```

Sets and mappings, which are unordered, make use of curly brackets:

```mamba
# sets
def c := { 10, 20 }
def d := { 3 }
# sets, builder syntax
def cd := { x ^ y | x in c, y in d }
def empty_set := {}

# maps
def e := { "do" => 1, "ree" => 2, "meee" => 3 }
# maps, builder syntax
def ef := { x => y - 2 | x in e, y = x.len() }

# indexing works for lists and maps/mappings (sets cannot be indexed because these are unordered)
print(ab(2)) # prints '(2, "list")'
print(ef(1)) # prints '1'
```

_Note_ Builder syntax currently only resolves a single bound variable (optionally filtered, e.g. `[x | x in a, x > 0]`).
Binding more than one, as in the `ab` and `ef` examples above, is future work.

_Note_ `{}` is always parsed as an empty set.
There is no empty mapping literal yet.

In a way, a list is a type of mapping where the keys are the indexes of each item.
So:

```mamba
def numbers := [32, 504, 59]
```

Is essentially just shorthand for

```mamba
def numbers := { 0 => 32, 1 => 504, 2 => 59 }
```

Where we iterate over the list in the order of the keys.

Unlike C-style languages (which are nearly the whole world at this point), we index collections using `collection(<expression>)`.
We namely don't distinguish between a mapping and a function, because a function is (generally speaking) also a type of mapping.
The above mapping, for instance, is a representation of some function with a very small domain (only three items).
Therefore, we index indexable collections (mappings and lists) using the `collection(<expression>)` notation.

### ✏️🖊️ Mutability

Mutability gives us the power to modify an instance in the language after it is created:

```mamba
def mut a := 10 # we may modify a
def b := 20     # we may not modify b

a := a + 2   # allowed
# b := b + 2 # compilation error
```

A binding is immutable unless we mark it `mut`, as in Rust.
This holds everywhere a name is bound, so it covers variables, function arguments, the `self` argument of a method, and class fields.
A tuple is one binding per element, so it takes one marker per element, as in `def (mut a, b) := (10, 20)`.
The reason is domain.
Mamba is geared towards mathematical use, and a symbol in mathematics denotes one thing for the length of its scope.
Substitution of equals for equals, which is the move that makes such reasoning work, is only valid when a name cannot change underneath you.

### 📋 Types, Properties, and Classes

Next, we introduce the concept of a class.
A class is essentially a blueprint for the behaviour of an instance.

In Mamba, like Python and Rust, a function in a class may take an explicit `self` argument, which gives access to the state of this instance.
Such a function is called a method.

A method is an ordinary function whose first argument is the instance, named `self` by convention.
`p.move(1, 2)` and `move(p, 1, 2)` differ in spelling, not in kind, and the second is what the first means.
A function in a class body that takes no `self` simply a function that needs no instance, and is called on the class.

Because `self` is just an argument, it obeys the argument rules.
We say whether a method may modify the instance by marking that argument, exactly as we would any other.
Write `self` and it is immutable, write `mut self` and it is not.

We showcase this using a simple `Matrix2x2` object.

```mamba
class MatrixErr(message: Str): Exception(message)

class Matrix2x2(mut a: Float, mut b: Float, mut c: Float, mut d: Float) where
    # Accessor for matrix contents
    def contents(self) -> List[Float] := [self.a, self.b, self.c, self.d]

    # Trace of the matrix (a + d)
    def trace(self) -> Float := self.a + self.d

    # Determinant of the matrix (ad - bc)
    def determinant(self) -> Float := self.a * self.d - self.b * self.c

    # Solves this matrix against the vector (u, v) by Cramer's rule.
    # A singular matrix has no unique solution, so this may fail.
    def solve(self, u: Float, v: Float) -> List[Float] ! MatrixErr := do
        def det := self.determinant()
        if det = 0.0 then ! MatrixErr("Determinant is zero.")
        def x := u * self.d - self.b * v
        def y := self.a * v - u * self.c
        [x / det, y / det]
    end

    def scale(mut self, factor: Float) := do
        self.a := self.a * factor
        self.b := self.b * factor
        self.c := self.c * factor
        self.d := self.d * factor
    end

    # Reset turns this matrix into a 2x2 identity matrix, regardless of the initial value.
    def reset(mut self) := do
        self.a := 1.0
        self.b := 0.0
        self.c := 0.0
        self.d := 1.0
    end
end
```

None of these methods can be `pure`, which is worth dwelling on.
`scale` and `reset` mutate the matrix, so the four fields must be `mut`.
A pure function may not read a `mut` field, so no method reading `a` to `d` qualifies.
See [Pure functions](#-pure-functions--041) below.

Notice how `self` is not mutable in `trace`, meaning we can only read variables, whereas in `scale`, `self` is mutable, so we can change properties of `self`.
_In general_, the notation of a class is:

`class MyClass(<one-or-more-constructor-args>) where <one-or-more-expressions> end`

The body of the class is optional, i.e. one can create "just" a data class.
Class arguments are always fields, stored on `self` (e.g. `self.a`, accessible externally as `matrix.a`).
There is no `def` prefix.

As for the class body:

- It may only declare fields and methods, plus an optional leading docstring.
  A bare statement is rejected.
  Such a statement would run once per instance, which hides whether constructing the class has side effects.
  Put that work in an explicit `new` instead, where the signature shows it.
- It is denoted using a code set: Using `where` and `end`.
  This is because the concept of order is not defined in a class body.
- In future, we may generalize the code-set notation to mean a set of statements which may be executed in arbitrary order, and thus **also in parallel**.
  Therefore baking parallel computations into the semantics of the language, as opposed to a library.
  However, this idea is still in its infancy.

#### Constructing a class

Mamba has no constructor to define or override.
A class is constructed through `new`, which every class gets for free, taking exactly its class arguments:

```mamba
class Point(x: Int, y: Int)

def p := Point.new(3, 4)
```

Applying the class to its arguments, as `Point(3, 4)`, is the underlying primitive, and it is only in scope **within `Point` itself**.
Outside, it does not resolve.
That is deliberate.
If it were public, `new` would be advisory: a caller could sidestep it, and two spellings of the same thing would coexist forever.
Because it is not, declaring your own `new` is the only way in, and it can therefore enforce something:

```mamba
class Fraction(num: Int, den: Int) where
    def pure new(num: Int, den: Int) -> Self ! FractionErr :=
        if den = 0 then ! FractionErr("Denominator is zero") else Fraction(num, den)
end
```

A declared `new` replaces the generated one.
`Self` names the enclosing class, and works in a method and in an associated function alike.

A function in a class body that takes no `self` is an **associated function**, called on the class rather than on an instance.
That is what makes `new` ordinary rather than special, and it means named alternatives sit beside it as equals:

```mamba
class Matrix2x2(a: Float, b: Float, c: Float, d: Float) where
    def pure new
    def pure identity() -> Self := return Matrix2x2(1.0, 0.0, 0.0, 1.0)
end

def m := Matrix2x2.identity()
```

Those who have worked with structured languages such as Rust will find this very familiar.
Constructors are not a feature of the language but enforced by convention.

#### Derived fields

A field declared in the body is *derived*: computed from the class arguments rather than passed.
This exists so you need not list every field at the construction site:

```mamba
class Circle(radius: Float) where
    def area: Float := self.radius * self.radius * 3.14159
end

def c := Circle.new(2.0)   # area is computed, never passed
```

A derived field must be assigned a value, unless its type is nullable.
Without one it would silently hold `None` whatever its type claims.
If you meant to pass it, make it a class argument instead.

We can change the relevant parts of the above example to use a class constant:

```mamba
class Point2D(ORIGIN_X: Int, ORIGIN_Y: Int) where
    def mut x: Int := self.ORIGIN_X
    def mut y: Int := self.ORIGIN_Y

    def move(mut self, dx: Int, dy: Int) := do
        self.x := self.x + dx
        self.y := self.y + dy
    end

    # Unlike the matrix before, reset resets this point to the value it was when it was instantiated.
    def reset(mut self) := do
        self.x := self.ORIGIN_X
        self.y := self.ORIGIN_Y
    end

    def info(self) -> Str :=
        "Currently at ({self.x}, {self.y}), originally from ({self.ORIGIN_X}, {self.ORIGIN_Y})"
end
```

Last, we have `trait`s, which in Mamba are more fine-grained building blocks to describe the behaviour of instances.
These are similar to interfaces in Java and Kotlin, and near identical to traits in Rust.
In Mamba, we aim to have many small traits for a more idiomatic way to express the behaviour of objects/classes.
For those familiar with object-oriented programming, we favour a trait-based system over inheritance (like Rust, Mamba doesn't have inheritance).

Consider example with iterators (which briefly showcases language generics):

```mamba
trait Iterator[T] where
    def has_next(self) -> Bool
    def next(self) -> T? # syntax sugar for Option[T]
end

class RangeIter(_start: Int, _end: Int) where
    def mut _current: Int := _start
end

def Iterator[Int] for RangeIter where
    def has_next(self) -> Bool := self._current < self._end

    def next(mut self) -> Int? := if self.has_next() then do
        def value := self._current
        self._current := self._current + 1
        value
    end else None
end
```

Prefer using an adjective (e.g. `Iterable`, `Hashable`, `Comparable`) when defining a trait, as this describes something a class and its instances can do.
The syntax here is `trait <id> where <one-or-more-definitions> end` and we use it as `def <trait> for <class>`.

_Note_ Implementing a trait externally, with `def <trait> for <class> where ... end` as in the `RangeIter` example above, is future work.
For now a class states the traits it implements in its own declaration (`class Person(name: Str): Named where ... end`) and defines their methods in its own body.

Lastly, like Rust, types (traits) can also be used as generics.
This would allow, for instance, for defining a `Hash` trait and enforcing for a hashmap that keys implement said trait.
We can also compose traits, which means that when we define the composite trait for a class we have to implement all definitions at once.
The syntax is very similar to inheritance for classes:

E.g.

```mamba
trait Ordered[T]: Equality, Comparable
```

_Note_ A trait may currently name at most one parent trait (`trait Ordered[T]: Equality`); composing several, as above, is future work.
A _class_, on the other hand, can already list several parents (`class MyClass: MyType, MyType2 where ... end`).

### 🔒 Pure functions (🇻 0.4.1+)

A function is pure when `f(x) = f(y)` for every `x = y`.
`=` is structural equality in Mamba, not identity, so two instances that look alike count as alike.
The idea is borrowed from proof assistants.

Functions are impure by default.
The same holds for methods, which as stated before as also function where the first argument is the instance.
Marking them `pure` guarantees that there are no side-effects, enforced by the following set of restrictions:

- Calling anything that is not itself `pure`.
- Reading or assigning a `mut` variable declared outside the function, whose value can change between calls.
- Assigning to a field of an argument, which reaches through to the caller's value.
- Reading a field of an argument that is `mut`, which can differ between two calls given equal arguments.
- Taking `mut self`, since mutating the receiver is mutating an argument.
- Constructing a class, unless that class declares construction pure.

And keeps:

- Everything the body defines, which is destroyed on return and may be used however you like.
- Reassigning an argument, which rebinds a local name and leaves the caller's value untouched.
- Reading any field that is not `mut`, and calling any method that is `pure`.

Read that list again with the first argument in mind and it collapses to one idea: a pure function may not touch anything that outlives the call, and may not depend on anything that can change between calls.

This is why `pure` suits plain functions better than methods.
A class with a mutating method needs `mut` fields, and no pure function may read one, so such a class has no pure methods at all.
`Matrix2x2` above is exactly that case.
Pure methods remain useful on a class whose fields are all immutable.

#### Pure construction

Purity is never inferred, here or anywhere.
A class states that constructing it is pure by declaring `new` pure, with no argument list and no body:

```mamba
class Point(x: Int, y: Int) where
    def pure new
end

def pure origin() -> Point := Point.new(0, 0)
```

There is no argument list because it is not declaring a signature.
The generated `new` already has the class arguments; this only asserts a property of it.
The assertion is then checked against the derived field initializers, which are the only thing construction runs:

```mamba
class Seeded(n: Int) where
    def pure new
    def seed: Int := random()   # rejected, random is not pure
end
```

Leave the assertion out and `new` is an ordinary impure function, so a pure function may not construct the class.
Writing a bare `def new` asks for what the class already has, and the compiler warns that it is redundant.

Note that `def pure new` constrains construction only, never the methods:

```mamba
class Counter(start: Int) where
    def pure new
    def mut count: Int := self.start

    # perfectly fine, purity was never claimed for methods
    def tick(mut self) := do
        self.count := self.count + 1
        print(self.count)
    end
end
```

This is why the marker sits on `new` rather than on the class.
A `class pure Counter` would read as though `tick` were pure too, which it is not.

Immutable bindings and pure functions together make a program declarative, with no hidden dependencies:

```mamba
# taylor is immutable, its value does not change during execution
def taylor := 7

# factorial must itself be pure, since sin calls it
def pure factorial(x: Int) -> Int := match x where
    0 => 1
    n => n * factorial(n - 1)
end

# the sin function is pure, its output depends solely on the input
def pure sin(x: Int) -> Int := do
    def mut ans := x
    for i in (1 ..= taylor).step(2) do
        ans := ans + (x ^ (i + 2)) / (factorial (i + 2))
    end
    ans
end
```

### 🤚 Total functions (🇻 x+)

A function may also be total, which means:

1. It is defined for all possible values of its domain
2. It will halt on all such inputs

The second property is interesting, because that would imply that the compiler can prove that an arbitrary function can halt.
To build such a compiler, we would need to solve the halting problem (which is impossible).
Instead, we place heavy restrictions on total functions, enforcing that they are weakly normalizing:

1. We may only call total functions
2. Within the _call tree_ of a function, all arguments to nodes in the tree must be _strictly decreasing_ compared to the first parent of a node which is equal to said node.

a.
If in the _call tree_ we call a different total function, the argument does not have to be strictly decreasing. b.
However, it should still be globally decreasing, meaning that we amend the above:

      _"compared to the first parent of the node which is equal to said node, summing over all intermediate nodes"
      This does mean that we must be able to perform basic arithmetic on the types of the function for this (logic) system to work!
      **In some sense, basic (integer) arithmetic forms the logical bedrock of our system**

3. Potentially non-terminating loops, which includes `while`, are not allowed
4. For loops may only be called over collections which implement `SizedIterator`, which is also implemented by the built-in:
   - `RangeToInclusive` :  `..=b`
   - `RangeTo` : `..b`
   - `Range` : `a..b`
   - `RangeInclusive` : `a..=b`

Put another way, we sidestep the issue by ensuring that our system is still sound, but incomplete by acknowledging that we cannot prove termination for arbitrary functions!
**Marking a function `total` does not ask the compiler to decide termination for the function as written.**
It switches the checker into a stricter mode that only accepts the fixed, mechanically checkable subset described by the four rules above.
Write something outside that shape, however obviously it halts to a human reader, and it is rejected.
This is the same trade-off `const fn` makes in Rust or `constexpr` makes in C++.

It's worth being explicit that this really is a strict subset, not a temporary gap we intend to close later.
Not every function that obviously halts can be marked `total`.
**Ackermann's function** is the classic example:

```mamba
# some syntax here such as guard arms which are not in the language yet
def ackermann(m: Nat, n: Nat) -> Nat := match (m, n) where
    (m, n) if m = 0 => n + 1
    (m, n) if n = 0 => ackermann(m - 1, 1)
    (m, n)          => ackermann(m - 1, ackermann(m, n - 1))
end
```

This halts for every input, but Mamba can never mark it `total`.
The trouble is the last case, `ackermann(m - 1, ackermann(m, n - 1))`.
The first argument does get smaller each time.
But the second argument is whatever the inner call returns, which can be a huge number, far bigger than `n` ever was.
Our checker only ever tracks one shrinking number per recursive call.
Here, there just isn't one number that always shrinks.

There is a way to prove this function halts, but it needs comparing two numbers together rather than one, a more powerful (and more complicated) technique than Mamba currently supports.
See [docs/features/functions/total_functions.md](docs/features/functions/total_functions.md) for the full mathematical story, including why some other languages and provers can accept this exact function today.

Take for instance this naive implementation of the Fibonacci sequence:

```mamba
## Fibonacci, implemented using recursion and not dynamic programming
def total pure fibonacci(x: Nat) -> Int := match x where
    0 => 0
    1 => 1
    n => fibonacci(n - 1) + fibonacci(n - 2)
end
```

This would, with some substitution magic, give the following _call tree_ (showing only the important parts):

```
            fibonacci(x)
                |
                + # addition operator 
               / \
fibonacci(x - 1) fibonacci(x - 2)
```

Thus, this function has the property of a final function, and we may thus mark it as `total` if we so choose.
The reason why we above state "compared to the first parent of a node which is equal to said node." is that we can have situations where we call other total functions which have recursive calls to self.
This allows us to call other recursive functions without having to strictly decrease the value of the input, but still enforce that calls to self (and more generally recursive calls to the same function) again are strictly decreasing.

We provide the `StrictlyDecreases` trait so users can define if something is strictly decreasing.
The compiler enforces that this is defined for each argument.
However, this is ripe for abuse, so instead, we require that each argument implements the trait `Measurable`.

```mamba
# if we implement strictly decreasing, we must implement measure
# These are non-overridable method which uses this measure
trait StrictlyDecreases: Measurable where
    def meta decreases(self, other: Self) -> Bool := self.measure() < other.measure()
    def meta equal(self, other: Self) -> Bool := self.measure() = other.measure()
    def meta subtract(self, other: Self) -> Nat? := self.measure() - other.measure()

    # this we must implement
    def meta measure(self) -> Measurable
end
```

This avoids abuse of `decreases` (i.e. one could write `def meta decreases(self, other: Self) := True`).
Instead, ordering is reduced to numeric ordering, which is verifiable and depends on the output of a pure function.
It is for instance defined for the built-in primitive `Int`.

```mamba
# Measure for Int returns abs(self), landing in Nat, since a measure needs a bounded-below domain
def StrictlyDecreases for Int where
    def meta measure(self) -> Measurable := self.abs()
end

# For string, we as an example use the length of the string (also a Nat)
def StrictlyDecreases for Str where
    def meta measure(self) -> Measurable := self.len()
end
```

Both of the above return a `Nat`, the non-negative integers.
`Nat` is part of the library and implements the `Measurable` trait.
`Measurable` is a special built-in trait of the language.
As of writing it cannot be implemented for custom types.
Zero is in `Nat`, which is what a measure needs.
Both `0.abs()` and `"".len()` are `0`.

`Nat` is a refinement of `Int`, not a separate primitive, and is future work.
Like `Int`, it is unbounded rather than a fixed-width unsigned integer.
See [docs/features/safety/types.md](docs/features/safety/types.md#nat) for its definition.

`Nat` is closed under addition but not under subtraction.
That is why `subtract` above is partial.
When `other` measures larger than `self`, the difference lands outside `Nat`.
There is no value to hand back, so `subtract` yields `None`.

This is deliberately not an error.
Leaving the domain is not a fault to report.
It is a question with no answer, and `Nat?` is how the language already says that.

It is equally deliberately not saturation at zero.
`None` and `0` have to stay distinct.
`0` says the two measures were equal.
`None` says the subtraction left the domain.
Collapsing them would report a decrease where there was none.
That is the unsoundness `Measurable` exists to rule out, so `decreases` never reads `None` as a decrease.
Since `subtract` is `meta`, which of the two it yields is settled at compile time.

Implementing `Measurable` for custom types is future work.
`measure()` only needs to be total, deterministic, and pure, into a bounded-below codomain such as `Nat`.
The compiler verifies the decrease independently at each call site.
Which type `measure()` is defined on does not matter.
See [docs/features/functions/total_functions.md](docs/features/functions/total_functions.md#measurable-and-custom-types) for the reasoning.

```mamba
# Trait measurable lives at the heart of this system, and by extension Mamba.
# If a trait is marked as meta, then all functions within must be meta.
@builtin
meta trait Measurable: Add, Sub, Eq, Comparable

# Built in to the standard library
# The idea is that this allows performing arithmetic not just at runtime but at compile-time.
def Measurable for Int
# The following is already defined for Int, but for the sake of our example:
# {
#     def meta less_than(self, other: Int) -> Bool := self < other
#     def meta unary_sub(self) -> Int              := -other
#     def meta add(self, other: Int) -> Int        := self + other
#     def meta equal(self, other: Int) -> Bool     := self = other
# }
```

We require that the measured item implements basic arithmetic so that we can add and subtract as we traverse those trees where we interweave recursive calls.
_Peano arithmetic, essentially, forms the logical bedrock of the system which proves functions are total._ Only meta functions can be evaluated at compile time, see the section on meta functions below.

In general:

- If a function is `pure`, it has no side effects.
- If a function is `total`, it will terminate for all possible inputs.

One does not imply the other, so you need both keywords if you want to say a function is total and pure.

The intended use-case is a bit more niche, likely mostly functions in the standard library, to show that they halt on all possible inputs.
But we can imagine that library writers might find these useful if they wish to be more thorough.

### Meta functions (🇻 x+)

Meta functions are evaluated at compile time, similar to macros in Rust (more so than in C or C++).
Their purpose is to prove properties of the program before code generation, not to generate code.

A meta function:

- May not call non-meta functions or values.
- Is always pure, so the `pure` keyword is omitted.
- Is total: its body is held to the same four restrictions as a `total` function, checked syntactically rather than by running it.

`meta` is closed to the standard library.
Opening it to user code is future work, though a `measure()` for a custom `Measurable` type (see above) would be a safe first case, since its shape can be checked without running it.
See [docs/features/functions/meta_functions.md](docs/features/functions/meta_functions.md).

Meta functions exist primarily as the logical bedrock for provable `total` functions.
A secondary benefit is performance: a meta computation runs once, at compile time, rather than being recomputed at every call.

- A meta function is defined as `def meta my_function(<args>) := ...`.
- A meta variable is defined `def meta my_var: MyType := ...`, with type annotations being non-optional.
- A meta trait is defined as `meta trait MyTrait ...`.
  Within a meta trait, all definitions are also meta.

### ⚠ Error handling

Unlike Python, Mamba does not have `try` `except` and `finally` (or `try` `catch` as it is sometimes known).
Instead, we aim to directly handle errors on-site so the origin of errors is more traceable.
The following is an attempt at mixing and matching `Result` monad (of languages like Rust and Scala), with a more first-class approach of exceptions in languages like Kotlin.
Again, this represents a trade-off between elegance of the type system and simplicity of the grammar versus having first-class language features.
Arguably it may be easier to just use Monads, similar to Rust's solution.
But, we are operating in a different domain, so that may be overly verbose for our purposes.

Let's continue with our matrix example.
The `solve` method above raises a `MatrixErr` when the matrix is singular.
We handle that error on-site:

```mamba
def m := Matrix2x2(1.0, 2.0, 3.0, 4.0)

def solution := m.solve(5.0, 6.0) ! where
    err: MatrixErr(message) => do
        print("Could not solve system: \"{message}\"")
        [0.0, 0.0] # optionally we can also return, but here we assign default value
    end
end

print("Solution is: {solution}")
```

In the above script, if the matrix turns out to be singular, we print an error (gracefully) and assign some other value to `solution`.
Here we showcase how we try to handle errors on-site instead of in a (large) `try` block.
This also prevents us from wrapping large code blocks in a `try`, where it might not be clear what statement or expression might throw what error.

_Note_ A case can currently only bind the error itself (`err: MatrixErr => ...`, with the message read off it as `err.message`).
Destructuring its constructor arguments, as in `err: MatrixErr(message)` above, is future work.

Under the hood, `<call> ! where <cases> end` desugars to a plain `match` on the call's result:

```mamba
match m.solve(5.0, 6.0) where
    err: MatrixErr(message) => print("Could not solve system: \"{message}\"")
end
```

This can also be combined with an assign.
In that case, we must either always return (halting execution or exiting the function), or evaluate to a value.
This is shown below, assuming the following error classes and fallible function are defined:

```mamba
class MyErr(message: Str): Exception(message)
class MyOtherErr(message: Str): Exception(message)

def function_may_throw_err() -> Int ! { MyErr, MyOtherErr } := 10
```

```mamba
def with_error_handling() := do
    def a: Int := function_may_throw_err() ! where
        err: MyErr => do
            print("We have a problem: {err.message}.")
            return  # we return, halting execution
        end
        err: MyOtherErr => do
            print("We have another problem: {err.message}.")
            0  # ... or we assign default value 0 to a
        end
    end

    print("a has value {a}.")
end

with_error_handling()
```

We can also opt to not do any error handling, making the type of `a`:

```mamba
def a: Result[Int, Union[MyErr, MyOtherErr]] := function_may_throw_err()
```

By extension, if we don't handle all cases, then the union becomes smaller.
Only when the union is empty, which happens when every error case is covered, does `a` have type `Int`.

If `a` is type `Result[...,...]`, and we are required to do error handling later.
So if we don't want to handle any of the exception cases at a given point, we just append an `!` to a function.
The exception(s) must be handled further up the stack.

```mamba
def a := function_may_throw_err() !
# if `function_may_throw_err` returned an exception, we will never reach this point
print("a has value {a}.")
```

This also gives an alternative way to write the above example, where we only care about a subset of the exceptions here.

```mamba
def a: Result[Int, MyErr] := function_may_throw_err() ! where
    err: MyOtherErr => do
        print("We have another problem: {err.message}.")
        0  # ... or we assign default value 0 to a
    end
end

a = a ! # Result[Int, MyErr] => Int, where if error case, an exception is raised.

print("a has value {a}.")
```

_Note_ Naming the `Result[...]` type explicitly, and narrowing one by re-raising with `a = a !`, are both future work.
Handling every case on-site (as in the examples further up) and propagating with a bare `!` are what is implemented today.

## 💽 Machine Output

There is an experimental feature where we output a very small subset of the language to machine code.
This is more of a 'fun' feature meant to explore a bit how compiler backends work to an extent.
We mostly limit this to simple arithmetic for now.

To use, us either the `--bin` flag to produce a binary, or `--asm` to print AT&T style assembly to stdout.
We aim to make sure that:

1. The output is identical to running and checking the output of the resulting Python (see `./tests/execution.rs`).
2. That compilation works as is identical on the latest Windows, Linux, and Mac OS.
   We verify this by making use of GitHub agents which run the test suite on each, see `./github/workflows/test.yml`.

In general, we aim to stay within the Rust ecosystem as much as possible.
We prefer writing our own boilerplate, or depending on rust crates, over depending on native C.
The reasoning is that we want to reduce external dependencies, and more importantly, that this arguably improves the educational value this crate provides (for the author).
Having to (re)-implement difficult compilation problems which have been solved in the past (and there are _many_, including edge cases) increases our exposure to them.

## 💻 The Command Line Interface

```
Transpile Mamba to Python code, compile it to a native binary, or print its assembly.

Usage: mamba [OPTIONS]

Options:
  -i, --input <INPUT>    Input file or directory. If file, file taken as input. If directory, recursively search all sub-directories for *.mamba files. If no input given, current directory used as input directory
  -o, --output <OUTPUT>  Output location. With `--python` (the default): output directory to store Python files, structured to reflect the input directory; if not given, a 'target' directory is created in the current directory. With `--bin`: path of the linked executable to produce; if not given, 'a.out' is created in the current directory. Ignored with `--asm`, which always prints to stdout instead of writing a file
      --python           Output Python source (the default)
      --bin              Compile and link a native executable via the Cranelift backend, instead of outputting Python source. Only a small subset of the language is currently supported:literals, arithmetic and comparison operators, if/else, top-level function definitions and calls, and `print`
      --asm              Compile via the Cranelift backend and print the resulting disassembly to stdout, instead of outputting Python source or linking an executable. No file is written -- pipe stdout (e.g. `> out.s`) if you want to save it. Same language subset as `--bin` (see its help). Printed in AT&T syntax (`movq %rsp, %rbp`, source before destination) -- Cranelift's own disassembler doesn't support switching to Intel syntax
      --target <TARGET>  Target triple to pass to Cranelift, e.g. `x86_64-unknown-linux-gnu` (only meaningful with `--bin`/`--asm`; defaults to the host triple)
  -v...                  Set level of verbosity: - `-v`   : info, error, warning printed to stderr (default) - `-vv`  : debug messages are printed - `-vvv` : trace messages are printed
  -d, --debug            Add line numbers to log statements
      --no-module-path   Disable the module path in the log statements
      --no-color         Disable colorized output
  -l, --level            Print log level
  -a, --annotate         Enable type annotation of the output source. Currently still buggy feature
  -h, --help             Print help (see more with '--help')
```

You can type `mamba -help` for a message containing roughly the above information.

# 👥 Contributing

Before submitting your first issue or pull request, please take the time to read both our [contribution guidelines](CONTRIBUTING.md) and our [code of conduct](CODE_OF_CONDUCT.md).

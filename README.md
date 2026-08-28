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

See [docs](/docs/) for a more extensive overview of the language philosophy.

This is a transpiler, written in [Rust](https://www.rust-lang.org/), which converts Mamba source files to Python source
files.
There therefore exists some interoperability with Python code.
Currently we compile down to Python, in future we may compile down to Python bytecode, for instance. 

This README:

- Gives a quickstart for developers
- Gives a short overview of the syntax and language features in quick succession, as well as the occasional reasoning behind them.

## 🧑‍💻 Quickstart for developers 👨‍💻

To get started right away, if you are on a Linux machine and wish to use the Nix flake (which has all the tooling setup, including nushell, githooks, etc.).
Still work in progress (Nix flakes are difficult to get right):

```sh
# Install Nix, in case you do not have it
sh <(curl --proto '=https' --tlsv1.2 -L https://nixos.org/nix/install) --daemon
# Start nix shell, with nushell and starship set up already
nix develop
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
def factorial(x: Int) -> Int := match x where
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
Also note that:

- Code blocks are denoted using `do` and `end` because this is a list of statements and expressions that gets executed _in order_.
- For a match expression or statement we denote cases starting with `where` and ending with `end`, as this is a _set_ of cases which we match on.
  You can read `match x where ... end`, where we read this as "match `x` on this set of conditions in `where ... end`".

_Note_ One could use [dynamic programming](https://en.wikipedia.org/wiki/Dynamic_programming) in the above example so that we consume less memory:

```mamba
def factorial(x: Int) -> Int := match x where
    0 => 1
    n => do
        def ans := 1
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

Builder syntax with more than one `in` (like `y in b` above) isn't fully resolved by the checker yet.

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

```
def a := 10     # we may modify a
def fin b := 20 # we may not modify b

a := a + 2   # allowed
# b := b + 2 # compilation error
```

We opt to make mutability the default (unlike say in Rust, where you have to use the `mut` keyword to make something mutable).
The reason for doing so is domain;
Mamba is geared more for mathematical use, for lack of a better term, meaning this design choice follows from the language philosophy.

### 📋 Types, Properties, and Classes

Next, we introduce the concept of a class.
A class is essentially a blueprint for the behaviour of an instance. 

In Mamba, like Python and Rust, each function in a class has an explicit `self` argument, which gives access to the state of this instance.
Such a function is called a method.
We can for each method state whether we can modify the state of `self` by stating whether it is mutable or not.
If we write `self`, it is mutable, whereas if we write `fin self`, it is immutable and we cannot change its state.
We can do the same for any argument to a function, for that matter.

We showcase this using a simple dummy `Matrix` object.
You will also see some "pure" functions, these will be explained later.

```mamba
class MatrixErr(message: Str): Exception(message)

class Matrix2x2(a: Int, b: Int, c: Int, d: Int) where
    # Accessor for matrix contents
    def contents(fin self) -> List[Int] := [self.a, self.b, self.c, self.d]

    # Trace of the matrix (a + d)
    def pure trace(fin self) -> Int := self.a + self.d

    # Determinant recomputation (pure function)
    def pure determinant(fin self) -> Int := self.a * self.d - self.b * self.c

    def scale(self, factor: Int) := do
        self.a := self.a * factor
        self.b := self.b * factor
        self.c := self.c * factor
        self.d := self.d * factor
    end

    # Reset turns this matrix into an 2x2 identity matrix, regardless of the initial value.
    def reset(self) := do
        self.a := 1
        self.b := 0
        self.c := 0
        self.d := 1
    end
end
```

Notice how `self` is not mutable in `trace`, meaning we can only read variables, whereas in `scale`, `self` is mutable, so we can change properties of `self`.
_In general_, the notation of a class is:

`class MyClass(<one-or-more-constructor-args>) := where <one-or-more-expressions> end`

The body of the class is optional, i.e. one can create "just" a data class.
Constructor arguments are always fields, stored on `self` (e.g. `self.a`, accessible externally as `matrix.a`) — there is no `def` prefix; it's just shorthand for a field without a separate constructor.
The body of the class is evaluated for each object we created, effectively making this the constructor body.

As for the class body:

- It is denoted using a code set: Using `{` and `}`.
  This is because the concept of order is not defined in a class body.
- In future, we may generalize the code-set notation to mean a set of statements which may be executed in arbitrary order, and thus **also in parallel**.
  Therefore baking parallel computations into the semantics of the language, as opposed to a library.
  However, this idea is still in its infancy.

We can change the relevant parts of the above example to use a class constant:

```mamba
class Point2D(ORIGIN_X: Int, ORIGIN_Y: Int) where
    def x: Int := self.ORIGIN_X
    def y: Int := self.ORIGIN_Y

    def move(self, dx: Int, dy: Int) := do
        self.x := self.x + dx
        self.y := self.y + dy
    end

    # Unlike the matrix before, reset resets this point to the value it was when it was instantiated.
    def reset(self) := do
        self.x := self.ORIGIN_X
        self.y := self.ORIGIN_Y
    end

    def info(fin self) -> Str := 
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
    def _current: Int := _start
end

def Iterator[Int] for RangeIter where
    def has_next(self) -> Bool := self._current < self._end

    def next(self) -> Int? := if self.has_next() then do
        def value := self._current
        self._current := self._current + 1
        value
    end else None
end
```

Prefer using an adjective (e.g. `Iterable`, `Hashable`, `Comparable`) when defining a trait, as this describes something a class and its instances can do.
The syntax here is `trait <id> := where <one-or-more-definitions end` and we use it as `def <trait> for <class>`.
Generics on traits, and implementing a trait for a class from outside the class body (as shown above), aren't implemented yet.

Lastly, like Rust, types (traits) can also be used as generics.
This would allow, for instance, for defining a `Hash` trait and enforcing for a hashmap that keys implement said trait.
We can also compose traits, which means that when we define the composite trait for a class we have to implement all definitions at once.
The syntax is very similar to inheritance for classes:

E.g.

```mamba
trait Ordered[T]: Equality, Comparable
```

### 🔒 Pure functions (🇻 0.4.1+)

Mamba has features to ensure that functions are pure, meaning that if `x = y`, for a pure function `f`, `f(x) = f(y)`.
`=` is the equality operator in Mamba, which checks for structural equality and not whether this is the same object in memory (with the same address).
This is inspired originally by pure functions in proof assistant tools.
For use to be able to compare two instances, the instance must implement the `Equality` trait (which we showed above).

By default, functions are not pure.
When we mark a function `pure`, restrictions are enforced by the language:

- `self` **must** be final (if this is a method).
  This means that it cannot mutate the values of self.
  It should be noted that if we mutate self and call a method again, then the output might be different.
  But, this makes sense!
  Self is just another argument to the function, and by mutating the instance we call the same function again but with a different instance, conceptually speaking.
- Call impure functions.

Some additional rules hold for calling and assigning to passed arguments to uphold the pure property (meaning, no side-effects):

- Anything defined within the function body is fair game, it may be used whatever way, as it will be destroyed upon exiting the function.
- An argument may be assigned to, as this will not modify the original reference.
- The field of an argument may not be assigned to, as this will modify the original reference.
- One may only read fields of an argument which are final (`fin`).
- One may only call methods of an argument which are pure (`pure`).
- It should be emphasized that all of the above also hold for accesses to `self` in the case of methods.

When a function is `pure`, its output is always the same for a given input.
It also has no side-effects, meaning that it cannot write anything (assign to mutable variables) or read from them.
Immutable variables and pure functions make it easier to write declarative programs with no hidden dependencies.

```mamba
# taylor is immutable, its value does not change during execution
def fin taylor := 7

# the sin function is pure, its output depends solely on the input
def pure sin(x: Int) -> Int := do
    def ans := x
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

   a. If in the _call tree_ we call a different total function, the argument does not have to be strictly decreasing.
   b. However, it should still be globally decreasing, meaning that we amend the above:

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
def ackermann(m: PosInt, n: PosInt) -> PosInt := match (m, n) where
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
def total pure fibonacci(x: PosInt) -> Int := match x where
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
    def fin meta decreases(self, other: Self) -> Bool := self.measure() < other.measure()
    def fin meta equal(self, other: Self) -> Bool := self.measure() = other.measure()
    def fin meta subtract(self, other: Self) -> Measurable := self.measure() - other.measure()

    # this we must implement
    def meta measure(self) -> Measurable
end
```

This avoids abuse of `decreases` (i.e. one could write `def fin meta decreases(self, other: Self) := True`).
Instead, ordering is reduced to numeric ordering, which is verifiable and depends on the output of a pure function.
It is for instance defined for the built-in primitive `Int`.

```mamba
# Measure for Int returns abs(self), landing in PosInt, since a measure needs a bounded-below domain
def StrictlyDecreases for Int where
    def meta measure(self) -> Measurable := self.abs()
end

# For string, we as an example use the length of the string (also a PosInt)
def StrictlyDecreases for Str where
    def meta measure(self) -> Measurable := self.len() 
end
```

Both of the above return a `PosInt`, which is part of the library and implements the `Measurable` trait.
This is a special built-in trait of the language, which as of writing cannot be implemented for custom types.

Implementing `Measurable` for custom types is future work.
`measure()` only needs to be total, deterministic, and pure, into a bounded-below codomain such as `PosInt`; the compiler verifies the decrease independently at each call site regardless of which type `measure()` is defined on.
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
_Peano arithmetic, essentially, forms the logical bedrock of the system which proves functions are total._
Only meta functions can be evaluated at compile time, see the section on meta functions below.

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
Before, we simply discarded the error by appending `!` to `last_op`.
Instead, we now handle the error on-site:

```mamba
def m := Matrix(1.0, 2.0, 3.0, 4.0)

if m.is_invertible() then
    def inv := m.inverse()
else
    print("Matrix is singular (not invertible).")

def last_op := m.last_op() ! where
    err: MatrixErr(message) => do
        print("Error when getting last op: \"{message}\"")
        "N/A" # optionally we can also return, but here we assign default value
    end
end

print("Last operation was: {last_op}")
```

In the above script, we will always print an error (gracefully) and assign some other value to `last_op`.
Here we showcase how we try to handle errors on-site instead of in a (large) `try` block.
This also prevents us from wrapping large code blocks in a `try`, where it might not be clear what statement or expression might throw what error.

Under the hood, `<call> ! where <cases> end` desugars to a plain `match` on the call's result:

```mamba
match m.last_op() where
    err: MatrixErr(message) => print("Error when getting last op: \"{message}\"")
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

```
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

Before submitting your first issue or pull request, please take the time to read both
our [contribution guidelines](CONTRIBUTING.md) and our [code of conduct](CODE_OF_CONDUCT.md).

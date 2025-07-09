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
- Type refinement features
- Null safety
- Explicit error handling
- A distinction between mutability and immutability
- Pure functions, or, functions without side effects

See [docs](/docs/) for a more extensive overview of the language philosophy.

This is a transpiler, written in [Rust](https://www.rust-lang.org/), which converts Mamba source files to Python source
files.
Mamba code should therefore be interoperable with Python code.
Functions written in Python can be called in Mamba and vice versa (from the generated Python files).
This interoparability is still a work in progress.

The below README:

- Gives a quickstart for developer
- Give a short overview of most of the syntax and language features in quick succession, as well as the occasional reasoning behind them.

## 🧑‍💻 Quickstart for developers 👨‍💻

To get started right away, if on a Linux machine and you wish to use the Nix flake (which has all the tooling setup, including nushell, githooks, etc.).
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
## Factorial of x
def factorial(x: Int) -> Int := match x {
    0 => 1
    n => n * factorial(n - 1)
}

def num := input("Compute factorial: ")
if num.is_digit() then [
    def result := factorial(Int(num))
    print("Factorial {num} is: {result}.")
] else
    print("Input was not an integer.")
```

We specify the type of argument `x`, in this case an `Int`, by writing `x: Int`.
This is part of the signature of the function, and is required (it cannot be inferred).
This means that the compiler will check for us that factorial is only used with integers as argument.
Also note that:

- Code blocks are denoted using `[` and `]` because this is a list of statements and expressions that gets executed _in order_.
- For a match expression or statement, each case is denoted using `{` and `}`, as this is a _set_ of cases which we match on.
  You you can read `match x {}` , where we read this as "match `x` on this set of conditions in `{...}`", though we omit the "on" as to not introduce another keyword.

_Note_ One could use [dynamic programming](https://en.wikipedia.org/wiki/Dynamic_programming) in the above example so that we consume less memory:

```mamba
def factorial(x: Int) -> Int := match x {
    0 => 1
    n => [
        def ans := 1
        for i in 1 ..= n do ans := ans * i
        ans
    ]
}
```

Logically, the `[...]` and `{...}` notation leads us nicely to collections in Mamba.

### 🍡 Collections

In Mamba, sets, lists, and maps are first class citizens.
They are baked into the language, including its grammar.

Lists make use of square brackets:

```mamba
# lists
def a := [0, 2, 51]
def b := ["list", "of", "strings"]
# lists of tuples, buidler syntax
def ab := [(x, y) | x in a, x > 0, y in b, b != "of" ]

# Indexing is done using curly brackets!
print(a(0)) # prints '0'
```

Sets and mappings, which are unordered, make use of curly brackets:

```mamba
# sets
def c := { 10, 20 }
def d := { 3 }
# sets, builder syntax
def cd := { x ^ y | x in c, y in d }

# maps
def e := { "do" => 1, "ree" => 2, "meee" => 3 }
# maps, builder syntax
def ef := { x => y - 2 | x in e, y = x.len() }

# indexing works for lists and maps/mappings (sets cannot be indexed because these are unordered)
print(ab(2)) # prints '(2, "list")'
print(ef(1)) # prints '1'
```

In a way, a list is a type of mapping where the keys are the indexes of each item.
So:

```mamba
def numbers := [32, 504, 59]
```

Is just shorthand for

```mamba
def numbers := { 0 => 32, 1 => 504, 2 => 59 }
```

Unlike C-style languages (which is nearly the whole world at this point), we index collections using `collection(<expression>)`.
The main reason for doing so is that we see collections which can be indexed as mappings.
Consider the above argument that a list is just another type of mapping.

We don't distinguish between a mapping and a function, because a function is (generally speaking) also a type of mapping.
We also argue that the above mapping is a representation of some functino with a very small domain (only three items).
Therefore, we index indexable collections (mappings and list) using the `collection(<expression>)` notation.

### 📋 Types, Classes, and Mutability

We introduce first two concepts here, mutability and classes.
Classes are similar to classes other object oriented language like Python, Kotlin and to an extent Rust.
Mutability gives us the power to modify an object in the language after it is created (we consider everything to be an object, though we don't consider Mamba to be strictly object-oriented).
So for instance

```
def a := 10     # we may modify a
def fin b := 20 # we may not modify b

a := a + 2   # allowed
# b := b + 2 # compilation error
```

We opt to make mutability the default (unlike say in Rust, where you have to use the `mut` keyword to make something mutable).
The reason for doing so is domain;
Mamba is geared more for mathematical use, for lack of a better term, meaning this design choice follows from the language philosophy.
This same philosophy will also influence later how we deal with equality checks between objects in the language and how we copy items in the language, where we favour a pure functional apporach similar to Haskell.

Continuing on classes, in Mamba, like Python and Rust, each method in a class has an explicit `self` argument, which gives access to the state of this class instance.
However, we can for each function state whether we can write to `self` or not by stating whether it is mutable or not.
If we write `self`, it is mutable, whereas if we write `fin self`, it is immutable and we cannot change its fields.
We can do the same for any field.
We showcase this using a simple dummy `Server` object.

```mamba
from ipaddress import IPv4Address

class ServerError(def message: Str): Exception(message)

def fin always_the_same_message := "Connected!"

class MyServer(def ip_address: IPv4Address) := [
    def is_connected: Bool  := False
    # We can use constructor arguments in the body of the class
    def _last_message: Str  := "my ip address when I was created was {ip_address}"

    def last_sent(fin self) -> Str ! ServerError := self._last_message

    def connect(self) := [
        self.is_connected := True
        print(always_the_same_message)
    ]

    def send(self, message: Str) ! ServerError := 
        if self.is_connected then self._last_message := message
        else ! ServerError("Not connected!")

    def disconnect(self) := self.is_connected := False
]
```

Notice how `self` is not mutable in `last_sent`, meaning we can only read variables, whereas in connect `self` is mutable, so we can change properties of `self`.
In general, the notation of a class is:

`class MyClass(<one-or-more-constructor-args>) := [<one-or-more-expressions>]`

Though the body is optional.
As for constructor arguments:

- If they are prefixed with `def`, then they are immediately accessible (e.g. `my_server.ip_address`).
- If they are **not** prefixed with `def`, then they are only constructor arguments.
  This means that they are a class-constant, a constant which is defined in the context of a class.
  This means that they may be used in any part of the class (body, functions, methods).
- The body of the class is evaluated for each object we created, effectively making this the constructor body.

We can change the relevant parts of the above example to use a class constant:

```mamba
from ipaddress import IPv4Address

class MyServer(IP_ADDRESS: IPv4Address) := [
    # The above IP_ADDRESS is a constant defined within the context of this class
    # The intial value of ip_address is the value we passed to the constructor, but it may change
    def ip_address: IPv4Address := IP_ADDRESS

    def change_ip(self, new_address: IPv4Address) := [
        print("When we first created this server, the address was {IP_ADDRESS}")
        print("In the meantime, our address is {self.ip_address}")
        print("And now we will change our address to {new_address}")

        self.ip_address := new_address
        # The following would result in a compilation error, we cannot assign to constants! 
        # IP_ADDRESS := new_address
    ]
]
```

### 🗃 Type refinement (🇻 0.4.1+) (Experimental!)

Mamba also has type refinement features to assign additional properties to types.
Having this as a first-class language feature and incorporating it into the grammar may have benefits, but does increase the comlexit of the language.
Arguably, it might detract from the elegance of the type system as well;
A different solution could be to just have a dedicated interface baked into the standard library for this purpose.

The general syntax is `type MyType [: OtherType] when <expression>`, where specifying `OtherType` is optional.
The expression can be of any form (and size), but **must** evaluate to a boolean.

```mamba
type SpecialInt: Int when self >= 0 and self <= 100 or self mod 2 = 0
```

We also introduce some syntax sugar again, where we can use `{` `}` to write each element of the conjunction on its own line.
_Note on performance: In terms of correctness, the order of the conjunctions obviously doesn't matter, but those who care about performance should know they are evaluated in order, so best to have simple ones first._

```mamba
type SpecialInt: Int when {
    self >= 0
    self <= 100 or self mod 2 = 0
}
```

Type refinement also allows us to specify the domain and co-domain of a function, say, one that only takes and returns positive integers:

```mamba
# we list the conditions below, which are a list of boolean expressions.
# this first-class language feature desugars to an list of checks which are done at the call site.
# we avoid desugaring to a function (at least when transpiling to Python) as to not clash with existing functions.
type PosInt: Int when self >= 0

def factorial(x: PosInt) -> PosInt := match x {
    0 => 1
    n => n * factorial(n - 1)
}
```

At the call site, one could do

```mamba
def x := -42 # some value

# currently this is a compilation error, x is type Int
# we cannot yet evaluate refined types at compile time, only runtime
# factorial(x) # error: 'x' is type Int, but signature is factorial(PosInt)

if x isa PosInt then
    print(factorial(x))
else
    print("x must be positive")
```
In short, types allow us to specify the domain and co-domain of functions with regards to the type of input, say, `Int` or `Str`.

Lets expand our server example from above, and rewrite it slightly:

```mamba
from ipaddress import IPv4Address

type ConnMyServer: MyServer when self.is_connected
type DisConnMyServer: MyServer when not self.is_connected

class ServerErr(def message: Str): Exception(message)

class MyServer(self: DisConnMyServer, def ip_address: IPv4Address) := [
    def is_connected: Bool  := False
    def _last_message: Str? := None

    def last_sent(self) -> Str ! ServerErr :=
        if self.last_message != None then self._last_message
        else ! ServerError("No last message!")

    def connect(self: DisConnMyServer) := self.is_connected := True

    def send(self: ConnMyServer, message: Str) := self._last_message := message

    def disconnect(self: ConnMyServer) := self.is_connected := False
]
```

Within the then branch of the if statement, we know that `self._last_message` is a `Str`.
This is because we performed a check in the if condition.

We now define the type of `self`.
Each type effectively denotes another state that `self` can be in.
For each type, we use `when` to show that it is a type refinement, which certain conditions.

```mamba
import ipaddress
from server import MyServer

def fin some_ip := ipaddress.ip_address("151.101.193.140")
def my_server   := MyServer(some_ip)

# The default state of http_server is DisconnectedHTTPServer, so we don't need to check that here
http_server.connect()

# We check the state
if my_server isa ConnMyServer then [
    # http_server is a Connected Server if the above is true
    my_server.send("Hello World!")
]

print("last message sent before disconnect: \"{my_server.last_sent}\".")
if my_server isa ConnectedMyServer then my_server.disconnect()
```

Type refinement allows, in the context of object oriented programming, thus allows us to also explicitly name the possible states of an object. 
This means that we don't constantly have to check that certain conditions hold.
We can simply ask whether a given object is a certain state by checking whether it is a certain type.

In general, the goal of the compiler will become:

- Limit the amount of checks that need to be done
- Detect when it becomes impossible to raise an exception, i.e. if it is impossible to break an invariant then we will never raise an exception.

Overall, the goal of type refinement it to allow us to express in greater detail the expected behaviour of functions in a more concise manner.
It is similar to "design by contract", though it should hopefully also create a clearer mental model of domains and codomains of functions.

### 🔒 Pure functions (🇻 0.4.1+)

Mamba has features to ensure that functions are pure, meaning that if `x = y`, for a pure function `f`, `f(x) = f(y)`.
`=` is the equality operator in Mamba, which checks for structural equality and not whether this is the same object in memory (with the same address).
This is inspired originally by pure functions in proof assistant tools.

By default, functions are not pure.
When we mark a function `pure`, restrictions are enforced by the language:

- Read non-final properties of `self` (if this is a method).
  This means that its output depends only on the direct input, and input to the constructor of the class.
- Call impure functions.

Some additional rules hold for calling and assigning to passed arguments to uphold the pure property (meaning, no side-effects):

- Anything defined within the function body is fair game, it may be used whatever way, as it will be destroyed upon exiting the function.
- An argument may be assigned to, as this will not modify the original reference.
- The field of an argument may not be assigned to, as this will modify the original reference.
- One may only read fields of an argument which are final (`fin`).
- One may only call methods of an argument which are pure (`pure`).
- It should be emphasized that all of the above also hold accesses to `self` in the case of methods.

When a function is `pure`, its output is always the same for a given input.
It also has no side-effects, meaning that it cannot write anything (assign to mutable variables) or read from them.
Immutable variables and pure functions make it easier to write declarative programs with no hidden dependencies.

```mamba
# taylor is immutable, its value does not change during execution
def fin taylor := 7

# the sin function is pure, its output depends solely on the input
def pure sin(x: Int) -> Int := [
    def ans := x
    for i in (1 ..= taylor).step(2) do
        ans := ans + (x ^ (i + 2)) / (factorial (i + 2))
    ans
]
```

### ⚠ Error handling

Unlike Python, Mamba does not have `try` `except` and `finally` (or `try` `catch` as it is sometimes known).
Instead, we aim to directly handle errors on-site so the origin of errors is more tracable.
The following is an attempt mixing and matching `Result` monad (of languages like Rust and Scala), with a more first-class approach of exceptions in languages like Kotlin.
Again, this represents a trade-off between elegancy of the type system and simplicity of the grammar versus having first-class language features.
Arguably it may be easier to just use Monads, similar to how Rust's solution.
But, we are operating in a different domain, so that may be overly verbose for our purposes.

Lets continue with our Server example.
We modify the above script such that we don't check whether the server is connected or not.
In that case, we must handle the case where `my_server` throws a `ServerErr`:

```mamba
import ipaddress
from server import MyServer

def fin some_ip := ipaddress.ip_address("151.101.193.140")
def my_server   := MyServer(some_ip)

def message := "Hello World!"
my_server.send(message) ! {
    err: ServerErr => print("Error while sending message: \"{message}\": {err}")
}

if my_server isa ConnectedMyServer then my_server.disconnect()
```

In the above script, we will always print the error since we forgot to actually connect to the server.
Here we showcase how we try to handle errors on-site instead of in a (large) `try` block.
This means that we don't need a `finally` block:
We aim to deal with the error where it happens and then continue executing the remaining code.
This also prevents us from wrapping large code blocks in a `try`, where it might not be clear what statement or expression might throw what error.

`my_server.send(message) ! { ... }` is syntax sugar for

```mamba
match my_server.send(message) {
    err: Exception(ServerErr) => print("Error while sending message: \"{message}\": {err}")
}
```

The `{...}` after `!` is also not necessary if we only match on one exception.

So esentially, we add `!` as a way to shorthand match on exceptions.
Currently, we allow both notations, but this comes at the cost of there not being "one way" to handle exceptions.
This can lead to similar problems like with Scala where we have multiple ways to do the same thing.
We can, of course, add warnings to strongly encourage the "right" way to handle exceptions.

This can also be combined with an assign.
In that case, we must either always return (halting execution or exiting the function), or evaluate to a value.
This is shown below:

```mamba
def a: Int := function_may_throw_err() ! {
    err: MyErr => [
        print("We have a problem: {err.message}.")
        return  # we return, halting execution
    ]
    err: MyOtherErr => [
        print("We have another problem: {err.message}.")
        0  # ... or we assign default value 0 to a
    ]
}

print("a has value {a}.")
```

We can also opt to not do any error handling, making the type of `a`:

```
def a: Result[Int, Union[MyErr, MyOtherErr]] := function_may_throw_err()
```

By extension, if we don't handle all cases, then the union becomes smaller.
Only when the union is empty, which happens when every error case is covered, does `a` have type `Int`.

If `a` is is type `Result[...,...]`, and we are required to do error handling later.
So if we don't want to handle any of the exception cases at a given point, we just append an `!` to a function.
The exception(s) must be handeld further up the stack.

```mamba
def a := function_may_throw_err() !
# if `function_may_throw_err` returned an exception, we will never reach this point
print("a has value {a}.")
```

This also gives an alternative way to write the above example, where we only case about a subset of the exceptions here.

```mamba
def a: Result[Int, MyErr] := function_may_throw_err() ! {
    err: MyOtherErr => [
        print("We have another problem: {err.message}.")
        0  # ... or we assign default value 0 to a
    ]
}

a = a ! # Result[Int, MyErr] => Int, where if error case, an exception is raised.

print("a has value {a}.")
```

Finally, we also introduce the `recover` keyword.
The intention is that instead of letting someone else up the stack perform cleanup, we can couple some of the cleanup at this site.
For instance, de-allocation resources which we no longer need.
This is similar to `drop` in Rust, though this applies only to errors/exceptions (as we generally speaking rely on garbage collection).
This is also similar to `finally` in Python, though we don't always run this block, only when we encounter an error.

The general syntax is `<expression-or-statement> recover <expression-or-statement>`
So:

```mamba
def a: Result[Int, MyErr] := function_may_throw_err() ! {
    err: MyOtherErr => print("We have a problem: {err.message}.")
} recover [
    print("cleaning up resource")
    some_cleanup_function()
]
```

## 💻 The Command Line Interface

```
USAGE:
    mamba.exe [FLAGS] [OPTIONS]

FLAGS:
    -a, --annotate          Enable type annotation of the output source.
                            Currently still buggy feature.
    -d, --debug             Add line numbers to log statements
    -h, --help              Prints help information
    -l, --level             Print log level
        --no-module-path    Disable the module path in the log statements
        --no-color          Disable colorized output
    -v                      Set level of verbosity
                            - v   : info, error, warning printed to sterr (Default)
                            - vv  : debug messages are printed
                            - vvv : trace messages are printed
    -V, --version           Prints version information

OPTIONS:
    -i, --input <INPUT>      Input file or directory.
                             If file, file taken as input.
                             If directory, recursively search all sub-directories for *.mamba files.
                             If no input given, current directory used as input directory.
    -o, --output <OUTPUT>    Output directory to store Python files.
                             Output directory structure reflects input directory structure.
                             If no output given, 'target' directory created in current directory.
```

You can type `mamba -help` for a message containing roughly the above information.

# 👥 Contributing

Before submitting your first issue or pull request, please take the time to read both
our [contribution guidelines](CONTRIBUTING.md) and our [code of conduct](CODE_OF_CONDUCT.md).

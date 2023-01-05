<link rel="shortcut icon" type="image/x-icon" href="image/logo.ico">

<p align="center">
    <img src="image/logo.svg" height="200" alt="Mamba logo"/>
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
Mamba is like Python, but with a few key features:

- Strict static typing rules, but with type inference so it doesn't get in the way too much
- Type refinement features
- Null safety
- Explicit error handling
- A distinction between mutability and immutability
- Pure functions, or, functions without side effects

This is a transpiler, written in [Rust](https://www.rust-lang.org/), which converts Mamba source files to Python source
files.
Mamba code should therefore be interoperable with Python code.
Functions written in Python can be called in Mamba and vice versa (from the generated Python files).

## ⌨️ Code Examples

Below are some code examples to showcase the features of Mamba.
We highlight how functions work, how de define classes, how types and type refinement features are applied, how Mamba can be used to ensure pureness, and how error handling works.

### ➕ Functions

We can write a simple script that computes the factorial of a value given by the user.

```mamba
def factorial(x: Int) -> Int => match x
    0 => 1
    n => n * factorial(n - 1)

def num := input("Compute factorial: ")
if num.is_digit() then
    def result := factorial(Int(num))
    print("Factorial {num} is: {result}.")
else
    print("Input was not an integer.")
```

Notice how here we specify the type of argument `x`, in this case an `Int`, by writing `x: Int`.
This means that the compiler will check for us that factorial is only used with integers as argument.

_Note_ One could use [dynamic programming](https://en.wikipedia.org/wiki/Dynamic_programming) in the above example so that we consume less memory:

```mamba
def factorial(x: Int) -> Int => match x
    0 => 1
    n =>
        def ans := 1
        for i in 1 ..= n do ans := ans * i
        ans
```

### 📋 Types, Classes, and Mutability

Classes are similar to classes in Python, though we can for each function state whether we can write to `self` or not by stating whether it is mutable or not.
If we write `self`, it is mutable, whereas if we write `fin self`, it is immutable and we cannot change its fields.
We can do the same for any field. We showcase this using a simple dummy `Server` object.

```mamba
from ipaddress import IPv4Address

class ServerError(def message: Str): Exception(message)

def fin always_the_same_message := "Connected!"

class MyServer(def ip_address: IPv4Address)
    def is_connected: Bool  := False
    def _last_message: Str  := "temp"

    def last_sent(fin self) -> Str raise [ServerError] =>
        self._last_message

    def connect(self) =>
        self.is_connected := True
        print(always_the_same_message)

    def send(self, message: Str) raise [ServerError] =>
        if self.is_connected then
            self._last_message := message
        else
            raise ServerError("Not connected!")

    def disconnect(self) => self.is_connected := False
```

Notice how `self` is not mutable in `last_sent`, meaning we can only read variables, whereas in connect `self` is mutable, so we can change properties of `self`.
We can then use `MyServer` as follows:

```mamba
import ipaddress
from server import MyServer

def fin some_ip := ipaddress.ip_address("151.101.193.140")
def my_server   := MyServer(some_ip)

http_server.connect()
if my_server.is_connected then http_server.send("Hello World!")

# This statement may raise an error, but for now de simply leave it as-is
# See the error handling section for more detail
print("last message sent before disconnect: \"{my_server.last_sent()}\".")
my_server.disconnect()
```

### 🗃 Type refinement (🇻 0.4.1+)

As shown above Mamba has a type system.
Mamba however also has type refinement features to assign additional properties to types.
Lets expand our server example from above, and rewrite it slightly:

```mamba
from ipaddress import IPv4Address

type ConnMyServer: MyServer when self.is_connected
type DisConnMyServer: MyServer when not self.is_connected

class ServerErr(def message: Str): Exception(message)

class MyServer(self: DisConnMyServer, def ip_address: IPv4Address)
    def is_connected: Bool  := False
    def _last_message: Str? := None

    def last_sent(self) -> Str raise [ServerErr] => 
        if self.last_message != None then 
            self._last_message
        else
            raise ServerError("No last message!")

    def connect(self: DisConnMyServer) => self.is_connected := True

    def send(self: ConnMyServer, message: Str) => self._last_message := message

    def disconnect(self: ConnMyServer) => self.is_connected := False
```

Within the then branch of the if statement, we know that `self._last_message` is a `Str`.
This is because we performed a check in the if condition.

Also Notice how above, we define the type of `self`.
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
if my_server isa ConnMyServer then
    # http_server is a Connected Server if the above is true
    my_server.send("Hello World!")

print("last message sent before disconnect: \"{my_server.last_sent}\".")
if my_server isa ConnectedMyServer then my_server.disconnect()
```

Type refinement also allows us to specify the domain and co-domain of a function, say, one that only takes and returns positive integers:

```mamba
type PosInt: Int when 
    self >= 0 else "Must be greater than 0"

def factorial(x: PosInt) -> PosInt => match x
    0 => 1
    n => n * factorial(n - 1)
```

In short, types allow us to specify the domain and co-domain of functions with regards to the type of input, say, `Int`
or `Str`. During execution, a check is done to verify that the variable does conform to the requirements of the
refined type. If it does not, an exception is raised.

Type refinement allows us to do some additional things:

- It allows us to further specify the domain or co-domain of a function
- It allows us to explicitly name the possible states of an object. This means that we don't constantly have to check
  that certain conditions hold. We can simply ask whether a given object is a certain state by checking whether it is a
  certain type.

### 🔒 Pure functions (🇻 0.4.1+)

Mamba has features to ensure that functions are pure, meaning that if `x = y`, for any `f`, `f(x) = f(y)`.
(Except if the output of the function is say `None` or `NaN`.)
By default, functions are not pure, and can read any variable they want, such as in Python.
When we make a function `pure`, it cannot:

- Read non-final properties of `self`.
- Call impure functions.

Some rules hold for calling and assigning to passed arguments to uphold the pure property (meaning, no side-effects):

- Anything defined within the function body is fair game, it may be used whatever way, as it will be destroyed upon exiting the function.
- An argument may be assigned to, as this will not modify the original reference.
- The field of an argument may not be assigned to, as this will modify the original reference.
- One may only read fields of an argument which are final (`fin`).
- One may only call methods of an argument which are pure (`pure`).

When a function is `pure`, its output is always the same for a given input.
It also has no side-effects, meaning that it cannot write anything (assign to mutable variables) or read from them.
Immutable variables and pure functions make it easier to write declarative programs with no hidden dependencies.

```mamba
# taylor is immutable, its value does not change during execution
def fin taylor := 7

# the sin function is pure, its output depends solely on the input
def pure sin(x: Int) =>
    def ans := x
    for i in 1 ..= taylor .. 2 do
        ans := ans + (x ^ (i + 2)) / (factorial (i + 2))
    ans
```

### ⚠ Error handling

Unlike Python, Mamba does not have `try` `except` and `finally` (or `try` `catch` as it is sometimes known).
Instead, we aim to directly handle errors on-site so the origin of errors is more tracable.
The following is only a brief example.

We can modify the above script such that we don't check whether the server is connected or not.
In that case, we must handle the case where `my_server` throws a `ServerErr`:

```mamba
import ipaddress
from server import MyServer

def fin some_ip := ipaddress.ip_address("151.101.193.140")
def my_server   := MyServer(some_ip)

def message := "Hello World!"
my_server.send(message) handle
    err: ServerErr => print("Error while sending message: \"{message}\": {err}")

if my_server isa ConnectedMyServer then my_server.disconnect()
```

In the above script, we will always print the error since we forgot to actually connect to the server.
Here we showcase how we try to handle errors on-site instead of in a (large) `try` block.
This means that we don't need a `finally` block: We aim to deal with the error where it happens and then continue executing the remaining code.
This also prevents us from wrapping large code blocks in a `try`, where it might not be clear what statement or expression might throw what error.

`handle` can also be combined with an assign. In that case, we must either always return (halting execution or exiting the function), or evaluate to a value.
This is shown below:

```mamba
def g() =>
    def a := function_may_throw_err() handle
        err: MyErr =>
            print("We have a problem: {err.message}.")
            return  # we return, halting execution
        err: MyOtherErr =>
            print("We have another problem: {err.message}.")
            0  # ... or we assign default value 0 to a

    print("a has value {a}.")
```

If we don't want to use a `handle`, we can simply use `raise` after a statement or exception to show that its execution might result in an exception, but we don't want to handle that here.
See the sections above for examples where we don't handle errors and simply pass them on using `raise`.

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

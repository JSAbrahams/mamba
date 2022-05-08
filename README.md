<link rel="shortcut icon" type="image/x-icon" href="image/logo.ico">

<h1 align="center">
    <img src="image/logo.svg" height="200" alt="Mamba logo"/>
    <br />
    Mamba
</h1>

<p align="center">
    <i>Safety first!</i>
    <br/><br/>
    <a href="https://github.com/JSAbrahams/mamba/actions/workflows/rust.yml">
    <img src="https://img.shields.io/github/workflow/status/JSAbrahams/mamba/Test?style=for-the-badge" alt="GitHub Workflow Status">
    </a>
    <a href="https://app.codecov.io/gh/JSAbrahams/mamba/">
    <img src="https://img.shields.io/codecov/c/github/JSAbrahams/mamba?style=for-the-badge" alt="Codecov coverage">  
    </a>
    <a href="https://crates.io/crates/mamba">
    <img src="https://img.shields.io/crates/v/mamba?style=for-the-badge" alt="Crate">  
    </a>
    <br/>
    <a href="https://github.com/JSAbrahams/mamba/blob/master/LICENSE">
    <img src="https://img.shields.io/github/license/JSAbrahams/mamba.svg?style=for-the-badge" alt="License"/>
    </a>
    <img src="https://img.shields.io/badge/Built%20with-%E2%99%A5-red.svg?style=for-the-badge" alt="Built with Love"/>
</p>

# Mamba

This is the Mamba programming language. The Documentation can be found [here](https://github.com/JSAbrahams/mamba_doc).
This documentation outlines the different language features, and also contains a formal specification of the language.

In short, Mamba is like Python, but with a few key features:

- Strict static typing rules, but with type inference so it doesn't get in the way too much
- Type refinement features
- Null safety
- Explicit error handling
- A distinction between mutability and immutability
- Pure functions, or, functions without side effects

This is a transpiler, written in [Rust](https://www.rust-lang.org/), which converts Mamba source files to Python source
files. Mamba code should therefore be interoperable with Python code. Functions written in Python can be called in Mamba
and vice versa (from the generated Python files).

## ⌨️ Code Examples

Below are some code examples to showcase the features of Mamba. We highlight how functions work, how de define classes,
how types and type refinement features are applied, how Mamba can be used to ensure pureness, and how error handling
works.

### ➕ Functions

We can write a simple script that computes the factorial of a value given by the user.

```mamba
def factorial(x: Int) -> Int => match x
    0 => 1
    n => n * factorial(n - 1)

def num := input("Compute factorial: ")
if num.is_digit() then
    def result := factorial(int(num))
    print("Factorial {num} is: {result}.")
else
    print("Input was not an integer.")
```

Notice how here we specify the type of argument `x`, in this case an `Int`, by writing `x: Int`. This means that the
compiler will check for us that factorial is only used with integers as argument.

### 📋 Classes and mutability

Classes are similar to classes in Python, though we can for each function state whether we can write to `self` or not by
stating whether it is mutable or not. If we write `self`, it is mutable, whereas if we write `fin self`, it is immutable
and we cannot change its fields. We can do the same for any field. We showcase this using a simple dummy `Server`
object.

```mamba
import ipaddress

class ServerError(def message: String): Exception(message)

def fin always_the_same_message := "Connected!"

class MyServer(def ip_address: IPv4Address)
    def is_connected: Bool    := False
    def _last_message: String := None

    def last_sent(fin self) -> String raise ServerError => if self._last_message /= None 
        then self._last_message
        else raise ServerError("No last message!")

    def connect(self) =>
        self.is_connected := true
        print(always_the_same_message)

    def send(self, message: String) raise ServerError => if self.is_connected 
        then self._last_message := message
        else raise ServerError("Not connected!")

    def disconnect(self) => self.is_connected := False
```

Notice how:

- `self` is not mutable in `last_sent`, meaning we can only read variables, whereas in connect `self` is mutable, so we
  can change properties of `sel

Which we can then use as follows in our script:

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

### 🗃 Types and type refinement

As shown above Mamba has a type system. Mamba however also has type refinement features to assign additional properties
to types.

Lets expand our server example from above, and rewrite it slightly:

```mamba
import ipaddress

type Server
    def ip_address: IPv4Address

    def connect()    -> () raise ServerErr
    def send(String) -> () raise ServerErr
    def disconnect() -> ()

type ConnectedMyServer: MyServer when self.is_connected
type DisconnectedMyServer: MyServer when not self.is_connected

class ServerErr(def message: String): Exception(message)

class MyServer(self: DisconnectedMyServer, def ip_address: IPv4Address): Server
    def is_connected: Bool    := False
    def _last_message: String := None

    def last_sent(self) -> String raise ServerErr => if self.last_message /= None 
        then self._last_message
        else raise ServerError("No last message!")

    def connect(self: DisconnectedMyServer) => self.is_connected := True

    def send(self: ConnectedMyServer, message: String) => self._last_message := message

    def disconnect(self: ConnectedMyServer) => self.is_connected := False
```

Notice how above, we define the type of `self`.

Each type effectively denotes another state that `self` can be in. For each type, we use `when` to show that it is a
type refinement, which certain conditions.

```mamba
import ipaddress
from server import MyServer

def fin some_ip := ipaddress.ip_address("151.101.193.140")
def my_server   := MyServer(some_ip)

# The default state of http_server is DisconnectedHTTPServer, so we don't need to check that here
http_server.connect()

# We check the state
if my_server isa ConnectedMyServer then
    # http_server is a Connected Server if the above is true
    my_server.send("Hello World!")

print("last message sent before disconnect: \"{my_server.last_sent}\".")
if my_server isa ConnectedMyServer then my_server.disconnect()
```

Type refinement also allows us to specify the domain and co-domain of a function, say, one that only takes and returns
positive integers:

```mamba
type PositiveInt: Int when 
    self >= 0 else "Must be greater than 0"

def factorial(x: PositiveInt) -> PositiveInt => match x
    0 => 1
    n => n * factorial(n - 1)
```

In short, types allow us to specify the domain and co-domain of functions with regards to the type of input, say, `Int`
or `String`. During execution, a check is done to verify that the variable does conform to the requirements of the
refined type. If it does not, an exception is raised.

Type refinement allows us to to some additional things:

- It allows us to further specify the domain or co-domain of a function
- It allows us to explicitly name the possible states of an object. This means that we don't constantly have to check
  that certain conditions hold. We can simply ask whether a given object is a certain state by checking whether it is a
  certain type.

### 🔒 Pure functions

Mamba has features to ensure that functions are pure, meaning that if `x = y`, for any `f`, `f(x) = f(y)` (except if the
output of the function is say `None` or `NaN`). By default, functions are not pure, and can read any variable they want,
such as in Python. When we make a function `pure`, it cannot:

- Read mutable variables not passed as an argument (with one exception).
- Read mutable properties of `self`. Mainly since `self` is never given as an argument, so a function output only
  depends on its explicit arguments.
- Call impure functions.

When a function is `pure`, its output is always the same for a given input. When a variable is immutable, when we
add `fin`, it can never change. So, `pure` is a property of functions, and `fin` is a property of variables.

```mamba
# taylor is immutable, its value does not change during execution
def fin taylor := 7

# the sin function is pure, its output depends solely on the input
def pure sin(x: Int) =>
    def ans := x
    for i in 1 ..= taylor step 2 do
        ans := (x ^ (i + 2)) / (factorial (i + 2))
    ans
```

Generally speaking, global variables can cause a lot of headaches. Immutable variables and pure functions make it easy
to write declarative programs with no hidden dependencies.

### ⚠ Error handling

Unlike Python, Mamba does not have `try` `except` and `finally` (or `try` `catch` as it is sometimes known). Instead, we
aim to directly handle errors on-site so the origin of errors is more tracable. The following is only a brief example.
Error handling can at times becomes quite verbose, so we do recommend checking out
the [docs](https://joelabrahams.nl/mamba_doc/features/safety/error_handling.html) on error handling to get a better feel
for error handling.

We can modify the above script such that we don't check whether the server is connected or not. In that case, we must
handle the case where `my_server` throws a `ServerErr`:

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

In the above script, we will always print the error since we forgot to actually connect to the server. Here we showcase
how we try to handle errors on-site instead of in a (large) `try` block. This means that we don't need a `finally`
block: We aim to deal with the error where it happens and then continue executing the remaining code. This also prevents
us from wrapping large code blocks in a `try`, where it might not be clear what statement or expression might throw what
error.

`handle` can also be combined with an assign. In that case, we must either always return (halting execution or exiting
the function), or evaluate to a value. This is shown below:

```mamba
def a := function_may_throw_err() handle
    err: MyErr => 
        print("We have a problem: {err.message}.")
        return  # we return, halting execution
    err: MyOtherErr => 
        print("We have another problem: {err.message}.")
        0  # ... or we assign default value 0 to a
        
print("a has value {a}.")
```

If we don't want to use a `handle`, we can simply use `raise` after a statement or exception to show that its execution
might result in an exception, but we don't want to handle that here. See the sections above for examples where we don't
handle errors and simply pass them on using `raise`.

## Structure

The transpiler can be split up into three distinct stages: parsing of the input, checking the input, and converting the
input to Python code.

### Lexer and Parser: String to AST

Convert a string of characters to a string of tokens. For each token we store the starting position within the file and
its width. This information is used when generating error messages in this and consecutive stages.

During this stage, errors are raised if we encounter an illegal character. We then convert the list of tokens to an
Abstract Syntax Tree (AST) based on the pre-defined grammar of the language.

During this stage syntax errors are raised if we encounter an illegal strings of tokens. I.e. a list of tokens that does
not conform to the grammar of the language. A lex error is thrown if something goes wrong during the lexing stage.

### Type checking

Check that the AST is correctly formed, beyond what the parser can check. We also verify that we only call methods which
are members of an object's class and that functions are used properly, which are imported. We do mutability checks, that
we only call variables which are initalized, and that functions receive only types they expect (at the very least an
expression which may be evaluated). More complex language features are also type checked.

This stage generates type errors, and contains the bulk of applicationl logic. This is also the last stage where error
messages may be generated. Any error after this stage is indicative of an internal error, and should be fixed.

Note that we do type checking before desugaring to improve the quality of error messages. This is the same approach
Haskell uses, at the cost of increased complexity of the type checker.

### Desugar to Python

Convert the AST to a simpler core language which looks similar to Python. This internal language is then again converted
to a string which represents Python code.

Note that in future, the type checker should annotate the tree such that each node has a type. This will allow us to add
type hints to all Python code we output, and to perhaps also more easily desugar more complex language construts (such
as classes).

## 💻 The Command Line Interface

### Usage

```
mamba [FLAGS] [OPTIONS]
```

### FLAGS

```
-d, --debug             Add line numbers to log statements
-h, --help              Prints help information
-l, --level             Print log level
    --no-module-path    Disable the module path in the log statements
    --no-color          Disable colorized output
-v                      Set level of verbosity
                        -    : info, error, warning printed to sterr (Default)
                        - v  : debug messages are printed
                        - vv : trace messages are printed
-V, --version           Prints version information
```

### Options

```
-i, --input <INPUT>      Input file or directory.
                         If file, file taken as input.
                         If directory, recursively search all sub-directories for *.mamba files.
                         If no input given, current directory used as input directory.
-o, --output <OUTPUT>    Output directory to store Python files.
                         Output directory structure reflects input directory structure.
                         If no output given, 'target' directory created in current directory and is used as ouput.
```

You can type `mamba -help` for a message containing roughly the above information.

## 👥 Contributing

Before submitting your first issue or pull request, please take the time to read both
our [contribution guidelines](CONTRIBUTING.md) and our [code of conduct](CODE_OF_CONDUCT.md).

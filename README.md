<link rel="shortcut icon" type="image/x-icon" href="image/logo.ico">

<p align="center">
    <img src="image/logo_text.png" height="250">
</p>

<p align="center">
    <a href="https://travis-ci.org/JSAbrahams/mamba/branches?branch=master"><img src="https://img.shields.io/travis/JSAbrahams/mamba/master.svg?style=for-the-badge&logo=linux" alt="Travis"/></a>
    <a href="https://ci.appveyor.com/project/JSAbrahams/mamba"><img src="https://img.shields.io/appveyor/ci/JSAbrahams/mamba/master.svg?style=for-the-badge&logo=windows" alt="Appveyor"/></a>
    <a href="https://app.codacy.com/project/JSAbrahams/mamba/dashboard"><img src="https://img.shields.io/codacy/grade/74944b486d444bf2b772e7311e9ae2f4.svg?style=for-the-badge" alt="Code Quality"/></a>
    <a href="https://codecov.io/gh/JSAbrahams/mamba"><img src="https://img.shields.io/codecov/c/github/JSAbrahams/mamba.svg?style=for-the-badge" alt="Coverage"/></a>
    <br>
    <a href="https://github.com/JSAbrahams/mamba/blob/master/LICENSE"><img src="https://img.shields.io/github/license/JSAbrahams/mamba.svg?style=for-the-badge" alt="License"/></a>
    <img src="https://img.shields.io/badge/Built%20with-%E2%99%A5-red.svg?style=for-the-badge" alt="Built with Love"/>
</p>

# Mamba

This is the Mamba programming language. 
The Documentation can be found [here](https://joelabrahams.nl/mamba_doc).
This documentation outlines the different language features, and also contains a formal specification of the language.

In short, Mamba is like Python, but with a few key features:
-   Strict typing rules, but with type inference so it doesn't get in the way too much, and type refinement features.
-   Null safety features.
-   More explicit error handling.
-   Clear distinction between state and mutability, and immutability and statelessness.

This is a transpiler, written in [Rust](https://www.rust-lang.org/), which converts Mamba source code to Python source files.
Mamba code should therefore, in theory, be interoperable with Python code.
Functions written in Python can be called in Mamba and vice versa.

## ⌨️ Code Examples

Below are some code examples to showcase the features of Mamba.
For more extensive examples and explanations check out the [documentation](https://joelabrahams.nl/mamba_doc).

### ➕ Functions

We can write a simple script that computes the factorial of a value given by the user.
```mamba
def factorial (x: Int) => match x with
    0 => 1
    n => n * factorial (n - 1)

def num    <- input "Compute factorial: "
def result <- factorial num
print "Factorial [num] is: [result]"
```

Notice how here we specify the type of argument `x`, in this case an `Int`, by writing `x: Int`.
This means that the compiler will check for us that factorial is only used with integers as argument.

### 🔓🔒 Stateful and stateless (and mutability)

Mamba allows us to explicitly state whether something has a state or is indeed without state.
A stateful object can modify its internal state (i.e. by changing a value of an internal field), whereas a stateless object cannot.

We showcase this using a simple dummy `Server` object.
```mamba
import ipaddress

stateful HTTPServer(def ip_address: ipaddress.ip_address)
    def mut connected: Bool              <- false
    def mut private last_message: String <- undefined

    def last_sent(self) =>
        if last_message = undefined then Err("No last message!")
        else                             message

    def connect(mut self) => self.connected <- true

    def send (mut self, message: String) => 
        if self.connected then self.last_message <- message
        else                   return Err("Not connected!")

    def disconnect(mut self) => self.connected <- true
```

Notice how:
-   `HTTPServer` is `stateful`, so we can have mutable top-level definitions such as `connected`, which may change over the lifetime of an object.
-   `last_message` is private, denoted by the `private` keyword.
    This means that we cannot access is directly, meaning we cannot for instance do `server.last_message <- "Mischief"`.
    Instead, we call `server.last_sent`.

Which we can then use as follows in our script:
```mamba
import ipaddress
from server import HTTPServer

def some_ip <- ipaddress.ip_address "151.101.193.140"
def http_server = HTTPServer(some_ip)

http_server connect
if http_server connected then
    http_server send "Hello World!"

print "last message sent before disconnect: \"[http_server.last_sent]\""
http_server disconnect
```

### 🗃 Type and type refinement

As shown above Mamba has a type system.
Mamba however also has type refinement features to assign additional properties to types.

Lets expand our server example from above, and rewrite it slightly:
```mamba
import ipaddress

type Server
    def ip_address:            ipaddres.ip_address

    def connect:    () -> ()       throws [ServerErr]
    def send:       (String) -> () throws [ServerErr]
    def disconnect: () -> ()

type ServerErr(msg: String) isa Err(msg)

stateful HTTPServer(mut self: DisconnectedHTTPServer, def ip_address: ipaddress.ip_address) isa Server
    def mut connected: Bool              <- false
    def mut private last_message: String <- undefined

    def last_sent(self): String => self last_message

    def connect (mut self: DisconnectedHTTPServer) => self connected <- true

    def send_message(mut self: ConnectedHTTPServer, message: String) => self last_message <- message

    def disconnect(mut self: ConnectedHTTPServer) => self connected <- false

type ConnectedHTTPServer isa HTTPServer when
    self connected else ServerErr("Not connected.")

type DiconnectedHTTPServer isa HTTPServer when
    self not connected else ServerErr("Already connected.")
```

Notice how above, we define the type of `self`.

Each type effectively denotes another state that `self` can be in.
For each type, we use `when` to show that it is a type refinement, which certain conditions.

```mamba
import ipaddress
from server import HTTPServer

def some_ip <- ipaddress.ip_address "151.101.193.140"
def http_server = HTTPServer(some_ip)

# The default state of http_server is DisconnectedHTTPServer, so we don't need to check that here
http_server connect

# We check the state
if http_server isa ConnectedHTTPServer then
    # http_server is a ConnectedServer if the above is true
    http_server send "Hello World!"

print "last message sent before disconnect: \"[http_server.last_sent]\""

if http_server isa ConnectedHTTPServer then http_server disconnect
```

Type refinement also allows us to specify the domain and co-domain of a function, say, one that only takes and returns positive integers:
```mamba
type PositiveInt isa Int where self >= 0 else Err("Expected positive Int but was [self].")

# only takes positive integers and returns positive integers
def my_function (x: PositiveInt): PositiveInt => x * 6 + 2
```

In short, types allow us to specify the domain and co-domain of functions with regards to the type of input, say, `Int` or `String`.

Type refinement allows us to to some additional things:
-   It allows us to further specify the domain or co-domain of a function
-   It allows us to explicitly name the possible states of an object.
    This means that we don't constantly have to check that certain conditions hold.
    We can simply ask whether a given object is a certain state by checking whether it is a certain type.

### ⚠ Error handling

Unlike Python, Mamba does not have `try` `except` and `finally` (or `try` `catch` as it is sometimes known).
Instead, we aim to directly handle errors on-site so the origins of errors is more easily tracable.
The following is only a brief example.
Error handling can at times becomes quite verbose, so we do recommend checking out the [docs](https://joelabrahams.nl/mamba_doc/features/safety/error_handling.html) on error handling to get a better feel for error handling.

We can modify the above script such that we don't check whether the server is connected or not.
In that case, we must handle the case where `http_server` throws a `ServerErr`:
```mamba
import ipaddress
from server import HTTPServer

def some_ip <- ipaddress.ip_address "151.101.193.140"
def http_server = HTTPServer(some_ip)

def message <- "Hello World!"
http_server send message handle
    err: ServerErr => print "Error while sending [message]: err"

if http_server isa ConnectedHTTPServer then http_server disconnect
```

In the above script, we will always print the error since we forgot to actually connect to the server.
Here we shocase showcase how we try to handle errors on-site instead of in a `try` block.
This means that we don't need a `finally` block: We aim to deal with the error where it happens and then continue executing the remaining code.
This also prevents us from wrapping large code blocks in a `try`, where it might not be clear what statement or expression might throw what error.

## 👥 Contributing

Before submitting your first issue or pull request, please take the time to read both our [contribution guidelines](CONTRIBUTING.md) and our [code of conduct](CODE_OF_CONDUCT.md).

## 🔨 Tooling

Several tools are used to help maintain the quality of the codebase.
These tools are used by the continuous integration tools to statically check submitted code.
Therefore, to save time, it is a good idea to install these tools locally and run them before pushing your changes.

### Rustfmt

[Rustfmt](https://github.com/rust-lang/rustfmt) formats Rust code and ensures the formatting is consistent across the codebase.

-   **To install** `rustup component add rustfmt --toolchain nightly`
-   **To run** `cargo +nightly fmt`

The configuration of `Rustfmt` can be found in `.rustfmt.toml`.

*Note* The nightly build of `cargo` must be used (`rustup install nightly`).

### Clippy

[Clippy](https://github.com/rust-lang/rust-clippy) catches common mistakes made in Rust.

-   **To install** `rustup component add clippy`
-   **To run** `cargo clippy`

The configuration of `Clippy` can be found in `.clippy.toml`.

*Note* The stable build of `cargo` must be used (`rustup install stable`).

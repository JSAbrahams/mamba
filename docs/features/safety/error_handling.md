⬅ [🏠 Home](../../README.md)

⬅ [2 🛠 Features](../README.md)

⬅ [2.4 ⛑ Safety](README.md)

# 2.4.2 Error Handling

_Note_ Naming a `Result[...]` type explicitly is future work, and is marked as such below.
The rest of this page uses current syntax.
See the [README](../../../README.md#-error-handling) for a fuller treatment.

Errors are a fact of life.
They may be the result of incorrect data, user input, or a programming mistake.
Error handling, ideally, should be done in an explicit manner.
However, at the same time, error handling code should not become overly verbose as it might obfuscate the actual relevant parts of the codebase which perform the actual calculations.
Thus, a balance must be reached.

In some cases it may be that we might want to raise an error.
Exception handling and `try` `catch` blocks are common in modern languages.
These constructs however have been shown to be somewhat troublesome:

- When several lines of code are wrapped in a try catch block, we do not know which expression or statement is the one which might throw an exception.
- Certain languages don't require all exceptions to be part of the function or method signature.
  This means that a method call might result in an exception even if the source code does not reflect this.
  This means that the use of said method either has to either:
  - Manually check that a method does indeed not throw an exception, which becomes exponentially more difficult when a method calls other methods, and so forth.
  - Wrap all method calls in `try` `catch` blocks, which might often be unnecessary and make the application unnecessarily verbose.
  - Assume that the method will not throw an exception, which might be a source of bugs down the line and may result in runtime errors.

As such, we aim to address the above concerns by using a more explicit system of error handling outlined below.
In general we:

- Wish to handle errors where they occur in an explicit manner, or,
- We explicitly state that an expression or statement (or function or method) might throw an error.
  This creates a visual stack trace within the codebase itself, so anyone who reads the code knows where an error might originate from without even having to compile and run the code.

## Raises

An error is a class with `Exception` as its parent:

```mamba
class MyErr(msg: Str): Exception(msg)
class OtherErr(msg: Str): Exception(msg)
```

A class argument is always a field, so `msg` is stored on `self` and is readable as `self.msg`.
Passing it to `Exception(msg)` hands it to the parent as well.

We then write the following functions elsewhere, not within the error class:

```mamba
def g(x: Int) -> Int ! MyErr := if x = 10 then ! MyErr("x was 10") else x

# We can also have a function that raises multiple types of errors
def h(x: Int) -> Int ! { MyErr, OtherErr } :=
    if x > 10 then
        ! MyErr("bigger than 10")
    else
        ! OtherErr("not bigger than 10")
```

The `!` in a signature lists the errors that a function may raise.
A single error is written directly after the `!`, several are wrapped in `{` and `}`.
The `!` in front of an expression in the body is what actually raises an error.

## Result

_Note_ Naming the `Result[...]` type explicitly is future work.
What is implemented today is the `!` notation above, and the handling described below.

We could also use the `Result` type to define a possible return type and error pair:

```mamba
def g(x: Int) -> Result[Int, MyErr] := if x = 10 then ! MyErr("x was 10") else x

# We can also have a function that raises multiple types of errors
def h(x: Int) -> Result[Int, Union[MyErr, OtherErr]] :=
    if x > 10 then
        ! MyErr("bigger than 10")
    else
        ! OtherErr("not bigger than 10")
```

The first way of writing is preferred, as this more clearly separates the return type and possible errors that may be raised.
However, using `Result` may be better in some other situations.
For instance, it allows us to use the type alias feature of the language, which can be convenient in certain situations, such as when we wish to enforce consistency.
See [Type Aliases](../modules/type_alias.md) for a more in-depth explanation.
A trivial case would be:

```mamba
type MyResult: Result[Int, MyErr]

def g(x: Int) -> MyResult := if x = 10 then ! MyErr("x was 10") else x
```

## Handle

We can also explicitly handle an error on site.
We do this by appending `! where <cases> end` to the call, which matches on the type of the error to determine what to do.
A good first step is to log the error.
In this case, we simply print it:

```mamba
def l := g(9) ! where
    err: MyErr => do
        print(err)
        0
    end
end

# here, l is an Int, as the case above evaluates to one
print("l has value {l}.")
```

A case binds the error itself, here as `err`, so its fields are read off it as `err.msg`.
Use `_` instead of a name if the error is not needed.
Each case body must either evaluate to a value, as above, or return.

We may also return if we detect an error.
In that case, the code after would only be executed if no error occurred:

```mamba
def use_g() := do
    def l := g(9) ! where
        err: MyErr => do
            print(err)
            return
        end
    end

    # if we execute this code we know for sure no error was thrown
    # if an error was thrown this will not be executed at all
    print("l has value {l}.")
end
```

Assigning a default value, as in the first example, should be done with care.
Assigning to a definition if an error has occurred might bury the error, causing unexpected behaviour later during execution.

## Propagating

We do not have to handle an error where it occurs.
Appending a bare `!` to a call passes the error on, to be handled further up the stack:

```mamba
def l := g(9) !
# if g raised an error, we will never reach this point
print("l has value {l}.")
```

We can also handle only some of the errors on site and pass the rest on.
A function that handles `OtherErr` itself, but leaves `MyErr` to its caller, states so in its own signature:

```mamba
def only_handles_other(x: Int) -> Int ! MyErr := h(x) ! where
    err: OtherErr => do
        print(err)
        0
    end
end
```

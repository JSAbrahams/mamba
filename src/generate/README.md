<p align="center">
    <img src="../../image/logo.svg" height="150" alt="Mamba logo"/>
</p>

# Generate

Converts `AST` to Python code. Errors if:

- `AST` is not what we expect, which is indicative of an implementation error within the type checker which should catch
  this.
- We are converting a language construct which has not yet been (fully) implemented.

## Convert

First step, converts `AST` to a simpler internal `Core` representation, which is closer to the Python language.

### State And Imports

There is not a one-to-one mapping between Mamba and Python code. As such, the desugar stage maintains state as it
traverses the AST. This is to desugar certain language constructs in a certain manner depending on the context.

Imports are also kept track of and added to the final output. This includes Mamba built-in types which have to be
explicitly imported in Python. I.e. tuples are imported as `from typing import Tuples` when one annotates the output.

## Core

A set of simple `Core` nodes, which are very close to Python constructs. These may almost directly be converted to a
Python string. This step also keeps track of code blocks and relevant indentation and dedents.

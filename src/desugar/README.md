<p align="center">
    <img src="../../image/logo.svg" height="200" alt="Mamba logo"/>
</p>

# Desugar

Converts AST to Python code.

The input AST is converted to a special Python-like AST. This is then in turn converted to a String which should be a
properly formatted Python file.

### State And Imports

There is not a one-to-one mapping between Mamba and Python code. As such, the desugar stage maintains state as it
traverses the AST. This is to desugar certain language constructs in a certain manner depending on the context.

Imports are also kept track of and added to the final output. This includes Mamba built-in types which have to be
explicitly imported in Python. I.e. tuples are imported as `from typing import Tuples` when one annotates the output.

## Errors

- AST not what we expect, which is indicative of an implementation error within the type checker which should catch
  this.
- We are desugaring a language construct which has not yet been (fully) implemented.

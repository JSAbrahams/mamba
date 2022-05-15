# Structure

The general structure is as follows:

- `pipeline`: End-to-end pipeline where the compiler attempts to transpile Mamba files within a directory.
- `parse`: Convert strings to AST. Errors if illegal character encountered or AST would not conform to language grammar.
- `check`: Checks that this AST is well typed. Errors if it is not well-typed.
- `desugar`: Converts AST to Python string. Errors if desugaring unsupported language construct. Also errors in certain
  situations if AST not well-typed, which should've been caught beforehand by the `check` stage.

<h1 style="text-align: center;">
    <img src="image/logo.svg" height="200" alt="Mamba logo"/>
</h1>

# Structure

The general structure is as follows:

- `pipeline`: End-to-end pipeline where the compiler attempts to transpile Mamba files within a directory.
- `parse`: Convert strings to AST. Errors if illegal character encountered or AST would not conform to language grammar.
- `check`: Checks that this AST is well typed. Errors if it is not well-typed.
- `desugar`: Converts AST to Python string. Errors if desugaring unsupported language construct. Also errors in certain
  situations if AST not well-typed, which should've been caught beforehand by the `check` stage.

`main.rs` is the actual executable. This takes command-line arguments. These arguments are explained in the top-level
README. The transpiler outputs status messages (which includes errors) to the command line. If everything went
successfully, the final output is Python files in the specified output directory.

<p align="center">
    <img src="../image/logo.svg" height="150" alt="Mamba logo"/>
</p>

# Structure

The general pipeline, for a single file, is as follows:

(1) `String` -> (2) `AST` -> (3) check -> (4) `Core` -> (5) `String`

The general structure is as follows:

- `parse`: Convert strings to AST. Errors if illegal character encountered or AST would not conform to language grammar.
  This is step (1) and (2) above.
- `check`: Checks that this AST is well typed. Errors if it is not well-typed. This is step (3) above, and is where the
  bulk of the application logic is.
- `generate`: Converts AST to Python string. Errors if converting unsupported language construct. Also errors in certain
  situations if AST not well-typed, which should've been caught beforehand by the `check` stage. This is step (4)
  and (5) above.
- `pipeline`: End-to-end pipeline where the compiler attempts to transpile Mamba files within a directory. This brings
  all of the above together.

`main.rs` is the executable. This takes command-line arguments. These arguments are explained in the top-level README.
The transpiler outputs status messages (which includes errors) to the command line. If everything went successfully, the
final output is Python files in the specified output directory.

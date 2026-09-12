<p align="center">
    <img src="../image/logo.svg" height="150" alt="Mamba logo"/>
</p>

# Structure

The general pipeline, for a single file, is as follows:

(1) `String` -> (2) `AST` -> (3) check -> (4) `PythonCore` -> (5) `String`

The general structure is as follows:

- `parse`: Convert strings to AST.
  Errors if an illegal character is encountered, or if the AST would not conform to the language grammar.
  This is step (1) and (2) above.
- `check`: Checks that this AST is well typed.
  Errors if it is not well-typed.
  This is step (3) above, and is where the bulk of the application logic is.
- `backend`: Converts the checked AST to output.
  Which backend runs is selected by the `Backend` enum in `backend/mod.rs`:
  - `backend/python`: Converts AST to a Python string, via the intermediate `PythonCore` representation.
    This is the default, and is step (4) and (5) above.
    Errors if converting an unsupported language construct.
    Also errors in certain situations if the AST is not well-typed, which should have been caught beforehand by the `check` stage.
  - `backend/cranelift`: Compiles the checked AST straight to native machine code instead, for the `--bin` and `--asm` flags.
    Only a small subset of the language is supported.
    See its README for details.
- `common`: Types shared by all of the above, most notably `Position` for source spans in error messages.
- `io`: Reading source files, writing output files, and walking a directory of `.mamba` files.

`lib.rs` brings all of the above together in `transpile_dir`, the end-to-end pipeline where the compiler attempts to transpile Mamba files within a directory.

`main.rs` is the executable.
This takes command-line arguments.
These arguments are explained in the top-level README.
The transpiler outputs status messages, which include errors, to the command line.
If everything went successfully, the final output is Python files in the specified output directory.

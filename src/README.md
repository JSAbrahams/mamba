<p align="center">
    <img src="../image/logo.svg" height="150" alt="Mamba logo"/>
</p>

# Structure

The pipeline for a single file is:

`String` → **parse** → `AST` → **check** → `ASTTy` → **backend** → output

- `parse`: source to `AST`.
  Errors on an illegal character or on input that does not match the grammar.
- `check`: type checks the `AST` and annotates it as `ASTTy`.
  Most of the language semantics live here.
- `backend`: turns `ASTTy` into output.
  - `backend/python`: Python source, via `PythonCore`, and the default.
  - `backend/cranelift`: native code for `--bin` and `--asm`, covering a small subset.
- `common`: shared types, notably `Position` for error messages.
- `diagnostic`: the errors of every stage as `Diagnostic`, the data the crate gives out, and their rendering as text.
- `io`: with the `native` feature, reading a directory of `.mamba` files, writing the output, and linking an executable with `cc`.
- `wasm`: with the `wasm` feature, the JavaScript bindings.

`lib.rs` wires the stages together in `compile`.
`main.rs` is the CLI binary, with its flags defined in `cli.rs`.

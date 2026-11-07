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
- `backend`: turns `ASTTy` into output, selected by the `Backend` enum in `backend/mod.rs`.
  - `backend/python`: Python source, via `PythonCore`, and the default.
  - `backend/cranelift`: native code for `--bin` and `--asm`, covering a small subset.
- `common`: shared types, notably `Position` for error messages.
- `io`: reading sources, writing outputs, and walking a directory of `.mamba` files.

`lib.rs` wires the stages together in `transpile_dir`.
`main.rs` is the CLI binary, with its flags defined in `cli.rs`.

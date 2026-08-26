<p align="center">
    <img src="../../image/logo.svg" height="150" alt="Mamba logo"/>
</p>

# Cranelift

Compiles a checked `ASTTy` directly to native machine code via [Cranelift](https://cranelift.dev/), instead of
transpiling to Python source. Unlike the Python backend, there is no intermediate `PythonCore`-style tree: lowering
walks the `ASTTy` once and emits Cranelift IR straight into a `cranelift_object::ObjectModule` via imperative builder
calls, which Cranelift itself then turns into machine code.

Three public entry points, all in `mod.rs`, mirroring the Python backend's `write_output`/`gen`/`gen_arguments` shape:

- `write_output` -- compiles and links an executable, written to disk (the `--bin` CLI flag).
- `print_asm` -- compiles and prints the disassembly to stdout instead, no file written (the `--asm` CLI flag).
- `compile` / `disassemble` -- the single-file entry points those two build on: `compile` returns object bytes,
  `disassemble` returns disassembly text (see "Assembly output" below).

## Supported language subset

Only a small slice of Mamba compiles down to machine code, enforced by simply erroring
(`BackendErr::unimplemented`) on anything else:

- `Int`, `Bool`, `Float` primitives -- no collections, strings (beyond a `print` argument), classes, or traits.
- Arithmetic (`+ - * /`) and comparison (`< <= > >= == !=`) operators.
- `if`/`else`, both as a statement and in a function's tail (return) position.
- `for <id> in <a> .. <b>` / `..=` loops over `Int` ranges -- not arbitrary collections, since collections aren't
  supported at all.
- Plain (`:=`) reassignment of an already-declared variable -- not compound assignment (`+=` and friends).
- Top-level function definitions and calls, including forward references within the same file.
- `print`, lowered directly to libc `puts` (string literal) or `printf` (primitive value).

Every other top-level statement in a file is collected into a synthetic `main`, since machine code needs an explicit
entry point the way a `.mamba` file's top-to-bottom script execution doesn't.

## Layout

- `convert/` -- the lowering itself, split by AST category (`definition.rs`, `control_flow.rs`, `call.rs`,
  `operation.rs`, plus a shared `common.rs`), the same way `backend::python::convert` is. `mod.rs` holds the entry
  point (`lower_program`) and the three dispatchers a Mamba node can be lowered as: a statement (`lower_stmt`), the
  tail of a function body (`lower_tail`), or a value-producing expression (`lower_expr`).
- `primitive.rs` -- resolves a checked `Name` to the one Cranelift `Type` it supports (`Int`/`Bool`/`Float`), the
  same role `backend::python::name` plays for Python's richer type surface.
- `link.rs` -- shells out to the system `cc` to link object files into an executable, the same approach `rustc`
  itself uses rather than reimplementing a linker.
- `result.rs` -- `BackendErr`/`BackendResult`, mirroring `backend::python::result`.

## Assembly output

`disassemble` asks Cranelift to compute disassembly text (`Context::set_disasm` + `CompiledCode::vcode`) while
lowering, gated behind a `want_asm: bool` threaded through `convert::lower_program` so `compile` (the `--bin` path)
never pays for it. It's printed in AT&T syntax (source operand before destination, e.g. `movq %rsp, %rbp`) --
that's what Cranelift's own disassembler always produces; real Intel-syntax output would mean re-disassembling the
emitted machine code with an external disassembler (e.g. capstone) instead, which isn't wired up here.

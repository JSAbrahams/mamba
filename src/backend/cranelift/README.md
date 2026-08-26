<p align="center">
    <img src="../../image/logo.svg" height="150" alt="Mamba logo"/>
</p>

# Cranelift

Compiles a checked `ASTTy` directly to native machine code via [Cranelift](https://cranelift.dev/), instead of transpiling to Python source.
Unlike the Python backend, there is no intermediate `PythonCore`-style tree:
lowering walks the `ASTTy` once and emits Cranelift IR straight into a `cranelift_object::ObjectModule` via imperative builder calls, which Cranelift itself then turns into machine code.

Three public entry points, all in `mod.rs`, mirroring the Python backend's `write_output`/`gen`/`gen_arguments` shape:

- `write_output` -- compiles and links an executable, written to disk (the `--bin` CLI flag).
- `print_asm` -- compiles and prints the disassembly to stdout instead, no file written (the `--asm` CLI flag).
- `compile` / `disassemble` -- the single-file entry points those two build on: `compile` returns object bytes,
  `disassemble` returns disassembly text (see "Assembly output" below).

The general idea is that we are able to leverage the type checker so that we _know_ what the type of each node at compile time.
This means that we offer the flexibility of not having to exhaustively define types everywhere.
The type checker still verifies correctness and gives this information to us so that we are able to produce machine code.
Else, without knowing the type in advance, we would not be able to produce machine code except in the most trivial cases.

## Supported language subset

Only a small slice of Mamba compiles down to machine code,
enforced by simply erroring (`BackendErr::unimplemented`) on anything else:

- `Int`, `Bool`, `Float` primitives -- no collections, strings (beyond a `print` argument), classes, or traits.
- Arithmetic (`+ - * /`) and comparison (`< <= > >= == !=`) operators, over `Int` or `Float` -- `operation.rs`'s
  `lower_arith`/`lower_cmp` check the *operand's* resolved type (not just that it's some supported primitive) to
  pick `iadd`/`fadd` and friends, `icmp`/`fcmp`, since Cranelift has no single opcode for both.
- `if`/`else`, both as a statement and in a function's tail (return) position.
- `for <id> in <a> .. <b>` / `..=` loops over `Int` ranges -- not arbitrary collections, since collections aren't supported at all.
- Plain (`:=`) reassignment of an already-declared variable -- not compound assignment (`+=` and friends).
- Top-level function definitions and calls, including forward references within the same file.
- `print`, lowered directly to libc `puts` (string literal) or `printf` (an `Int`/`Bool` value). A `Float` value is
  rejected -- `printf`'s `%lld` would read the raw float bits as an integer, and a `%f`-style call needs SysV
  variadic-call ABI plumbing (setting `%al` to the vector-register count) this backend doesn't have yet.

Every other top-level statement in a file is collected into a synthetic `main`, since machine code needs an explicit
entry point the way a `.mamba` file's top-to-bottom script execution doesn't.

## Layout

- `convert/` -- the lowering itself, split by AST category (`definition.rs`, `control_flow.rs`, `call.rs`, `operation.rs`, plus a shared `common.rs`), the same way `backend::python::convert` is.
  `mod.rs` holds the entry point (`lower_program`) and the three dispatchers a Mamba node can be lowered as:
   a statement (`lower_stmt`), the tail of a function body (`lower_tail`), or a value-producing expression (`lower_expr`).
- `primitive.rs` -- resolves a checked `Name` to the one Cranelift `Type` it supports (`Int`/`Bool`/`Float`),
  the same role `backend::python::name` plays for Python's richer type surface.
- `link.rs` -- shells out to the system `cc` to link object files into an executable,
  the same approach `rustc` itself uses rather than reimplementing a linker.
- `result.rs` -- `BackendErr`/`BackendResult`, mirroring `backend::python::result`.

## Assembly output

`disassemble` asks Cranelift to compute disassembly text (`Context::set_disasm` + `CompiledCode::vcode`) while lowering,
gated behind a `want_asm: bool` threaded through `convert::lower_program` so `compile` (the `--bin` path) never pays for it.
It's printed in AT&T syntax (source operand before destination, e.g. `movq %rsp, %rbp`) as that's what Cranelift's own disassembler always produces;
real Intel-syntax output would mean re-disassembling the emitted machine code with an external disassembler (e.g. capstone) instead.
To keep things simple and to keep external dependencies to a minimum we opt not to do that.

This is instructions only, not a full disassembly of the object, it doesn't cover the data section.
A string literal (e.g. a `print("...")` argument) is emitted as a separate anonymous data blob,
so it never appears in the output;
The instructions that reference it only show an opaque symbol (e.g. `load_ext_name userextname0+0, %rdi`).
This is similar to how import like `puts` shows up as a bare symbol rather than "the puts function".

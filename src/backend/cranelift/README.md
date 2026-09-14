<p align="center">
    <img src="../../image/logo.svg" height="150" alt="Mamba logo"/>
</p>

# Cranelift

Compiles a checked `ASTTy` directly to native machine code via [Cranelift](https://cranelift.dev/), instead of transpiling to Python source.
Unlike the Python backend, there is no intermediate `PythonCore`-style tree.
Lowering walks the `ASTTy` once and emits Cranelift IR straight into a `cranelift_object::ObjectModule`, via imperative builder calls.
Cranelift itself then turns that into machine code.

There are three public entry points, all in `mod.rs`.
They mirror the Python backend's `write_output`/`gen`/`gen_arguments` shape:

- `write_output`: compiles and links an executable, written to disk (the `--bin` CLI flag).
- `print_asm`: compiles and prints the disassembly to stdout instead, with no file written (the `--asm` CLI flag).
- `compile` and `disassemble`: the single-file entry points those two build on.
  `compile` returns object bytes.
  `disassemble` returns disassembly text (see "Assembly output" below).

The general idea is that we are able to leverage the type checker so that we _know_ what the type of each node at compile time.
This means that we offer the flexibility of not having to exhaustively define types everywhere.
The type checker still verifies correctness and gives this information to us so that we are able to produce machine code.
Else, without knowing the type in advance, we would not be able to produce machine code except in the most trivial cases.

## Supported language subset

Only a small slice of Mamba compiles down to machine code, enforced by simply erroring (`BackendErr::unimplemented`) on anything else:

- `Int`, `Bool`, `Float` primitives.
  There are no collections, classes, or traits, and no strings beyond a `print` argument.
- Arithmetic (`+ - * /`) and comparison (`< <= > >= == !=`) operators, over `Int` or `Float`.
  `operation.rs`'s `lower_arith` and `lower_cmp` check the *operand's* resolved type, not just that it is some supported primitive.
  That is how they pick `iadd`/`fadd` and friends, and `icmp`/`fcmp`, since Cranelift has no single opcode for both.
- `if`/`else`, both as a statement and in a function's tail (return) position.
- `for <id> in <a> .. <b>` and `..=` loops over `Int` ranges.
  Arbitrary collections are not supported, since collections are not supported at all.
- Plain (`:=`) reassignment of an already-declared variable.
  Compound assignment (`+=` and friends) is not supported.
- Top-level function definitions and calls, including forward references within the same file.
- `print`, lowered directly to libc `puts` for a string literal, or `printf` for an `Int` or `Bool` value.
  A `Float` value is rejected.
  `printf`'s `%lld` would read the raw float bits as an integer.
  A `%f`-style call needs SysV variadic-call ABI plumbing, setting `%al` to the vector-register count, which this backend does not have yet.

Every other top-level statement in a file is collected into a synthetic `main`, since machine code needs an explicit entry point the way a `.mamba` file's top-to-bottom script execution doesn't.

## Layout

- `convert/`: the lowering itself, split by AST category, the same way `backend::python::convert` is.
  Those categories are `definition.rs`, `control_flow.rs`, `call.rs` and `operation.rs`, plus a shared `common.rs`.
  `mod.rs` holds the entry point, `lower_program`, and the three dispatchers a Mamba node can be lowered as.
  Those are a statement (`lower_stmt`), the tail of a function body (`lower_tail`), or a value-producing expression (`lower_expr`).
- `primitive.rs`: resolves a checked `Name` to the one Cranelift `Type` it supports, being `Int`, `Bool` or `Float`.
  This is the same role `backend::python::name` plays for Python's richer type surface.
- `link.rs`: shells out to the system `cc` to link object files into an executable.
  This is the same approach `rustc` itself uses, rather than reimplementing a linker.
- `result.rs`: `BackendErr` and `BackendResult`, mirroring `backend::python::result`.

## Assembly output

`disassemble` asks Cranelift to compute disassembly text while lowering, via `Context::set_disasm` and `CompiledCode::vcode`.
This is gated behind a `want_asm: bool` threaded through `convert::lower_program`.
That way `compile`, the `--bin` path, never pays for it.
It is printed in AT&T syntax, with the source operand before the destination, as in `movq %rsp, %rbp`.
That is what Cranelift's own disassembler always produces.
Real Intel-syntax output would mean re-disassembling the emitted machine code with an external disassembler, such as capstone.
We opt not to do that, to keep things simple and to keep external dependencies to a minimum.

This is instructions only, not a full disassembly of the object.
It does not cover the data section.
A string literal, such as a `print("...")` argument, is emitted as a separate anonymous data blob.
It therefore never appears in the output.
The instructions that reference it only show an opaque symbol, such as `load_ext_name userextname0+0, %rdi`.
This is similar to how an import like `puts` shows up as a bare symbol, rather than as "the puts function".

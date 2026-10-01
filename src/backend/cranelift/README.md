<p align="center">
    <img src="../../../image/logo.svg" height="150" alt="Mamba logo"/>
</p>

# Cranelift

Compiles a checked `ASTTy` directly to native machine code via [Cranelift](https://cranelift.dev/), instead of transpiling to Python source.
Unlike the Python backend, there is no intermediate `PythonCore`-style tree.
Lowering walks the `ASTTy` once and emits Cranelift IR straight into a `cranelift_object::ObjectModule`, via imperative builder calls.
Cranelift itself then turns that into machine code.

The purpose is Python's arithmetic at native speed, from a binary that is easy to produce on the fly.
It is not to compete with a systems language, see [the philosophy docs](../../../docs/philosophy/README.md#pythons-arithmetic-compiled).

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
  `Int` is unbounded, as it is in the Python backend.
  See "Unbounded integers" below.
- Arithmetic (`+ - * /`) and comparison (`< <= > >= == !=`) operators, over `Int` or `Float`.
  `operation.rs`'s `lower_arith` and `lower_cmp` check the *operand's* resolved type, not just that it is some supported primitive.
  That is how they pick the `Int` or the `Float` lowering, since Cranelift has no single opcode for both.
- `if`/`else`, both as a statement and in a function's tail (return) position.
- `for <id> in <a> .. <b>` and `..=` loops over `Int` ranges.
  Arbitrary collections are not supported, since collections are not supported at all.
- Plain (`:=`) reassignment of an already-declared variable.
  Compound assignment (`+=` and friends) is not supported.
- Top-level function definitions and calls, including forward references within the same file.
- `print`, lowered directly to libc `puts` for a string literal, or to the integer runtime for an `Int` or `Bool` value.
  A `Bool` prints as `1` or `0`.
  A `Float` value is rejected, since formatting one the way Python does is a much harder problem than an integer.

Every other top-level statement in a file is collected into a synthetic `main`, since machine code needs an explicit entry point the way a `.mamba` file's top-to-bottom script execution doesn't.

## Unbounded integers

`Int` is unbounded by design, as [the language docs](../../../docs/features/safety/types.md#unbounded-integers) describe.
The Python backend gets that for free from Python's own `int`.
Here it takes arbitrary-precision arithmetic, which this backend supplies itself.

### Representation

An `Int` is one 64-bit word, so `primitive.rs` still maps it to `types::I64`.
The low bit says what the word holds.

Low bit | Meaning | Range
---|---|---
0 | A small integer, stored shifted left by one. | -2^62 up to 2^62 - 1
1 | A pointer to a heap integer, offset by one. | Anything else

This is a known technique rather than an invention.
[mypyc](https://mypyc.readthedocs.io/en/stable/int_operations.html) represents Python's `int` the same way, and Lisp and Smalltalk systems did so long before.
See [prior art](../../../docs/philosophy/README.md#prior-art) for how other Python compilers handle integers.

A heap integer is a reference count, a sign, a length, and that many base 2^32 limbs, least significant first.
A result that fits a small integer is always returned as one.
Two small words therefore compare as plain integers, and equal values that are small are equal words.

### Fast path and runtime

`convert/int.rs` emits each `+`, `-`, `*` and comparison inline for two small integers.
That is the operation itself, a test of the two low bits, and a test for overflow.
A program whose numbers stay small never allocates.

The runtime takes over when an operand is a heap integer, or when a result overflows a small integer.
`runtime.rs` defines it as Cranelift IR, in every object, with local linkage.
There is no library to ship or link, and the only outside symbols are libc's `calloc`, `free`, `puts` and `exit`.
Its functions are named with a dot, such as `mamba.int.mul`, which no Mamba identifier can contain.
They are left out of `--asm` output.

Operation | How
---|---
`+` and `-` | One pass over the limbs. Signs that differ subtract the smaller magnitude from the larger.
`*` | Schoolbook multiplication, quadratic in the number of limbs.
Comparison | By sign, then by length, then limb by limb from the top.
`print` | Repeated division by 10^9, nine digits at a time.
`Int` to `Float` | The nearest `Float`, rounding half to even, as Python does.

A literal too long for a small integer is built at run time, from pieces of eighteen digits.

### Reference counting

A heap integer is freed when its last reference goes.
The rules are in `convert/int.rs`:

- A variable owns one reference to its value, released when its scope ends or the function returns.
- A temporary owns one reference until something consumes it, and is released if nothing does.
- A function owns the arguments it is passed, and its caller owns what it returns.

Reading a variable inside an expression borrows, and takes no reference.
That is sound because a reassignment is a statement, so nothing can free the value mid-expression.
Integers hold no references to anything, so there are no cycles to leak.

### Limits

- `/` converts both operands to `Float` first, where Python divides the integers exactly.
  The two can differ in the last bit once an operand passes 2^53.
- An `Int` beyond the range of a `Float` cannot be converted.
  The program prints an `OverflowError` and exits with status 1.
  Python raises the same error for a conversion, but can still divide two such integers when the quotient fits.
- Only 64-bit targets are supported, which is every target this build of Cranelift has.
- Deep recursion still overflows the stack, which goes against [the goal](../../../docs/philosophy/safety.md#slow-rather-than-stopped) of a program that slows down rather than stops.

## Layout

- `convert/`: the lowering itself, split by AST category, the same way `backend::python::convert` is.
  Those categories are `definition.rs`, `control_flow.rs`, `call.rs` and `operation.rs`, plus a shared `common.rs`.
  `int.rs` holds what is specific to `Int`: the inline fast paths and the reference counting.
  `mod.rs` holds the entry point, `lower_program`, and the three dispatchers a Mamba node can be lowered as.
  Those are a statement (`lower_stmt`), the tail of a function body (`lower_tail`), or a value-producing expression (`lower_expr`).
- `primitive.rs`: resolves a checked `Name` to the one Cranelift `Type` it supports, being `Int`, `Bool` or `Float`.
  This is the same role `backend::python::name` plays for Python's richer type surface.
- `runtime.rs`: the integer runtime, emitted as Cranelift IR into every object.
  See "Unbounded integers" above.
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

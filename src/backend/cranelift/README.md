<p align="center">
    <img src="../../../image/logo.svg" height="150" alt="Mamba logo"/>
</p>

# Cranelift backend

Compiles a checked `ASTTy` to native code via [Cranelift](https://cranelift.dev/).
There is no intermediate tree: lowering walks the `ASTTy` once and emits IR into a `cranelift_object::ObjectModule`.
The checker's types are what make this possible, since every node's type is known at compile time.

The purpose is Python's arithmetic at native speed, from a binary that is easy to produce on the fly.
It is not to compete with a systems language, see [the philosophy docs](../../../docs/philosophy/README.md#pythons-arithmetic-compiled).

There are three public entry points, all in `mod.rs`.
They mirror the Python backend's `write_output`/`gen`/`gen_arguments` shape:

- `write_output`: compiles and links an executable, for `--bin`.
- `print_asm`: prints the disassembly, for `--asm`.
- `compile` and `disassemble`: the single-file functions those two build on.

## Supported subset

Anything else errors with `BackendErr::unimplemented`.

- `Int`, `Bool` and `Float`, with arithmetic and comparison operators.
- `if`/`else`, as a statement and in tail position.
- `for` over an `Int` range, `..` or `..=`.
- Plain reassignment with `:=`, but no compound assignment.
- Top-level functions and calls, including forward references.
- `print` of a string literal, `Int` or `Bool`, through libc `puts` and `printf`.
  A `Float` is rejected, since variadic float calls need ABI support this backend lacks.

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

`--asm` prints Cranelift's own disassembly, in AT&T syntax.
It shows instructions only, so a string literal appears as an opaque symbol rather than as data.

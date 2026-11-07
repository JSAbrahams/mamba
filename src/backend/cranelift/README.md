<p align="center">
    <img src="../../../image/logo.svg" height="150" alt="Mamba logo"/>
</p>

# Cranelift backend

Compiles a checked `ASTTy` to native code via [Cranelift](https://cranelift.dev/), for `--bin` and `--asm`.
Lowering walks the `ASTTy` once and emits IR into a `cranelift_object::ObjectModule`, with no intermediate tree.

The goal is Python's arithmetic at native speed, from a binary that is easy to produce.
It is not to compete with a systems language, see [the philosophy docs](../../../docs/philosophy/README.md#pythons-arithmetic-compiled).

`mod.rs` holds the entry points.
`write_output` links an executable for `--bin`, and `print_asm` prints the disassembly for `--asm`.
Both build on `compile` and `disassemble`, which handle a single file.

## Supported subset

Anything else is rejected with `BackendErr::unimplemented`.

- `Int`, `Bool` and `Float` values.
  There are no collections, classes or traits, and no strings beyond a `print` argument.
- `+`, `-`, `*`, `/`, unary `+` and `-`, and the six comparisons.
  `//`, `mod` and `^` are not supported.
- `if`/`else` as a statement, or as a function's tail, but not as a value anywhere else.
- `for` over an `Int` range, `..` or `..=`.
- Plain reassignment with `:=`, but not compound assignment.
- Top-level functions and calls, including forward references.
- `print` of a string literal, an `Int` or a `Bool`.
  A `Float` cannot be printed yet, since formatting one the way Python does is a harder problem.

An operation picks its `Int` or `Float` lowering from the values it was given, not from the checker's types.
A checked operand's type can reflect the operator's parameter type instead of its own, such as `Float` for an `Int` compared with `>`.

## Unbounded integers

`Int` is unbounded, as [the language docs](../../../docs/features/safety/types.md#unbounded-integers) describe.
The Python backend gets that from Python's `int`, and this backend supplies arbitrary-precision arithmetic itself.

### Representation

An `Int` is one 64-bit word, so `primitive.rs` maps it to `types::I64`.
The low bit says what the word holds.

Low bit | Meaning | Range
---|---|---
0 | A small integer, stored shifted left by one. | -2^62 up to 2^62 - 1
1 | A pointer to a heap integer, offset by one. | Anything else

[mypyc](https://mypyc.readthedocs.io/en/stable/int_operations.html) represents Python's `int` the same way, as Lisp and Smalltalk systems did before it.
See [prior art](../../../docs/philosophy/README.md#prior-art).

A heap integer is a reference count, a sign, a length, and that many base 2^32 limbs, least significant first.
A result that fits a small integer is always returned as one, so equal small values are equal words.

### Fast path and runtime

`convert/int.rs` emits each `+`, `-`, `*` and comparison inline for two small integers.
That is the operation itself, a test of both low bits, and an overflow test.
A program whose numbers stay small never allocates.

Otherwise the runtime takes over.
`runtime.rs` emits it as Cranelift IR into every object, with local linkage, so there is no library to ship.
The only outside symbols are libc's `calloc`, `free`, `puts` and `exit`.
Its functions are named with a dot, such as `mamba.int.mul`, which no Mamba identifier can contain, and are left out of `--asm` output.

Operation | How
---|---
`+` and `-` | One pass over the limbs. Differing signs subtract the smaller magnitude from the larger.
`*` | Schoolbook multiplication, quadratic in the number of limbs.
Comparison | By sign, then length, then limb by limb from the top.
`print` | Repeated division by 10^9, nine digits at a time.
`Int` to `Float` | The nearest `Float`, rounding half to even, as Python does.
A long literal | Built at run time, from pieces of eighteen digits.

### Reference counting

A heap integer is freed when its last reference goes.
The rules are in `convert/int.rs`:

- A variable owns one reference, released when its scope ends or its function returns.
- A temporary owns one reference until something consumes it, and is released if nothing does.
- A function owns its arguments, and its caller owns what it returns.

Reading a variable inside an expression borrows it.
That is sound because reassignment is a statement, so nothing can free a value mid-expression.
Integers reference nothing, so there are no cycles to leak.

### Limits

- `/` converts both operands to `Float` first, where Python divides the integers exactly.
  The two can differ in the last bit once an operand passes 2^53.
- An `Int` beyond the range of a `Float` cannot be converted.
  The program prints an `OverflowError` and exits with status 1.
- Dividing by zero gives an infinity or `NaN`, where Python raises `ZeroDivisionError`.
- A `Bool` prints as `1` or `0`, where Python prints `True` or `False`.
- Only 64-bit targets are supported, which is every target this build of Cranelift has.
- Deep recursion overflows the stack, against [the goal](../../../docs/philosophy/safety.md#slow-rather-than-stopped) of a program that slows down rather than stops.

## Layout

- `convert/`: the lowering, split by AST category as `backend::python::convert` is.
  `mod.rs` holds `lower_program` and the three ways to lower a node: as a statement, a function's tail, or a value.
  `int.rs` holds the `Int` fast paths and reference counting.
- `primitive.rs`: maps a checked `Name` to its Cranelift `Type`.
- `runtime.rs`: the integer runtime.
- `link.rs`: links object files with the system `cc`, as `rustc` does.
- `result.rs`: `BackendErr` and `BackendResult`.

`--asm` prints Cranelift's disassembly in AT&T syntax.
It shows instructions only, so a string literal appears as an opaque symbol.

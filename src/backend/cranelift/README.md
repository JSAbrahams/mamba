<p align="center">
    <img src="../../../image/logo.svg" height="150" alt="Mamba logo"/>
</p>

# Cranelift backend

Compiles a checked `ASTTy` to native code via [Cranelift](https://cranelift.dev/).
There is no intermediate tree: lowering walks the `ASTTy` once and emits IR into a `cranelift_object::ObjectModule`.
The checker's types are what make this possible, since every node's type is known at compile time.

Entry points, in `mod.rs`:

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

Other top-level statements are collected into a synthetic `main`.

## Known bug: `Int` is 64-bit

`Int` is unbounded in the language, but lowers to `I64` here, so arithmetic wraps.
Unlike every other limitation, this compiles and silently disagrees with the Python backend.
The fix, a bignum or checker-proven range bounds, is future work.

## Layout

- `convert/`: the lowering, split by AST category like `backend/python/convert`.
  `mod.rs` dispatches a node as a statement, a function tail or an expression.
- `primitive.rs`: maps a checked `Name` to a Cranelift `Type`.
- `link.rs`: links object files with the system `cc`.
- `result.rs`: `BackendErr` and `BackendResult`.

## Assembly output

`--asm` prints Cranelift's own disassembly, in AT&T syntax.
It shows instructions only, so a string literal appears as an opaque symbol rather than as data.

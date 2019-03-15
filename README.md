<p align="center">
    <img src="/image/logo_medium.png" height="250">
</p>

# Mamba

The Mamba programming language. Documentation can be found [here](https://github.com/JSAbrahams/mamba_doc).

It's like python, but with null and type safety. T

This is transpiler, which converts Mamba source code to python source files. 
Mamba code should therefore be interoperable with python code, meaning functions written in python can be called in Mamba and vice versa.

## 👥 Contributing

Check out the [contributing](/CONTRIBUTING.md) document for contribution guidelines.

## 🔨 Tooling

Several tools are used to ensure the quality of the codebase.

### Rustfmt

[Rustfmt](https://github.com/rust-lang/rustfmt) formats Rust code and ensures the formatting is consistent across the codebase.

The configuration of `Rustfmt` can be found in `.rustfmt.toml`.

Note that the nightly build of `Rustfmt` must be used.

### Clippy

[Clippy](https://github.com/rust-lang/rust-clippy) catches common mistakes made in Rust.

The configuration of `Clippy` can be found in `.clippy.toml`.

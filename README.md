<p align="center">
    <img src="/image/logo_medium.png" height="250">
</p>

# Mamba

This is the Mamba programming language. 
The Documentation can be found [here](https://github.com/JSAbrahams/mamba_doc).
This documentation outlines the different language features, and also contains a formal specification of the language.

In short, Mamba is Python but with a few key features:
* Type checking
* Null safety
* Clear distinction between state and mutability, and immutability and statelessness

This is a transpiler that converts Mamba source code to Python source files, written in Rust.
Mamba code should therefore be interoperable with Python code, meaning functions written in Python can be called in Mamba and vice versa.

## 👥 Contributing

Check out the [contributing](/CONTRIBUTING.md) document for contribution guidelines.
Please read this document carefully before submitting your first issue or pull request.

## 🔨 Tooling

Several tools are used to maintain the quality of the codebase.
These tools are used by the continuous integration tools to statically check submitted code.
Therefore, to save time, it is a good idea to install these tools locally and run them before pushing your changes.

### Rustfmt

[Rustfmt](https://github.com/rust-lang/rustfmt) formats Rust code and ensures the formatting is consistent across the codebase.

The configuration of `Rustfmt` can be found in `.rustfmt.toml`.

Note that the nightly build of `Rustfmt` must be used.

### Clippy

[Clippy](https://github.com/rust-lang/rust-clippy) catches common mistakes made in Rust.

The configuration of `Clippy` can be found in `.clippy.toml`.

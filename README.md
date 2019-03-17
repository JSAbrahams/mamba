<p align="center">
    <img src="/image/logo_medium.png" height="250">
</p>

[![Travis Build Status][travis-status]](https://travis-ci.org/JSAbrahams/mamba)
[![codecov][codecov-status]](https://codecov.io/gh/JSAbrahams/mamba)
[![Built with love][baby-dont-hurt-me]](https://github.com/JSAbrahams/)

# Mamba

This is the Mamba programming language. 
The Documentation can be found [here](https://github.com/JSAbrahams/mamba_doc).
This documentation outlines the different language features, and also contains a formal specification of the language.

In short, Mamba is like Python, but with a few key features:
* Strict typing rules, but with type inference so it doesn't get in the way too much
* Null safety features
* More explicit error handling
* Clear distinction between state and mutability, and immutability and statelessness

This is a transpiler, written in [Rust](https://www.rust-lang.org/), which converts Mamba source code to Python source files.
Mamba code should therefore, in theory, be interoperable with Python code.
Functions written in Python can be called in Mamba and vice versa.

## 👥 Contributing

Check out the [contributing](/CONTRIBUTING.md) document for contribution guidelines.

Please read this document carefully before submitting your first issue or pull request.

## 🔨 Tooling

Several tools are used to help maintain the quality of the codebase.
These tools are used by the continuous integration tools to statically check submitted code.
Therefore, to save time, it is a good idea to install these tools locally and run them before pushing your changes.

### Rustfmt

[Rustfmt](https://github.com/rust-lang/rustfmt) formats Rust code and ensures the formatting is consistent across the codebase.

- **To install** run `rustup component add rustfmt --toolchain nightly`
- **To run** run `cargo +nightly fmt`

The configuration of `Rustfmt` can be found in `.rustfmt.toml`.

*Note* The nightly build of `cargo` must be used.

To install the nightly build of `cargo`, run `rustup install nightly`.

### Clippy

[Clippy](https://github.com/rust-lang/rust-clippy) catches common mistakes made in Rust.

- **To install** 
    - make sure you have the latest version of `rustup` by running `rustup update`
    - run `rustup component add clippy`
- **To run** run `cargo clippy`

The configuration of `Clippy` can be found in `.clippy.toml`.

*Note* The stable build of `cargo` must be used.
This is installed by default but in case it isn't, run `rustup install stable`.

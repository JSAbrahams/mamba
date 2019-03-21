<p align="center">
    <img src="image/logo_medium.png" height="250">
</p>

<p align="center">
    <a href="https://travis-ci.org/JSAbrahams/mamba"><img src="https://img.shields.io/travis/JSAbrahams/mamba/master.svg?style=for-the-badge&logo=travis" alt="Travis"/></a>
    <a href="https://ci.appveyor.com/project/JSAbrahams/mamba"><img src="https://img.shields.io/appveyor/ci/JSAbrahams/mamba/master.svg?style=for-the-badge&logo=appveyor" alt="Appveyor"/></a>
    <br>
    <a href="https://app.codacy.com/project/JSAbrahams/mamba/dashboard"><img src="https://img.shields.io/codacy/grade/74944b486d444bf2b772e7311e9ae2f4.svg?style=for-the-badge" alt="Code Quality"/></a>
    <a href="https://codecov.io/gh/JSAbrahams/mamba"><img src="https://img.shields.io/codecov/c/github/JSAbrahams/mamba.svg?style=for-the-badge" alt="Coverage"/></a>
    <br>
    <a href="https://github.com/JSAbrahams/mamba/blob/master/LICENSE"><img src="https://img.shields.io/github/license/JSAbrahams/mamba.svg?style=for-the-badge" alt="License"/></a>
    <img src="https://img.shields.io/badge/Built%20with-%E2%99%A5-red.svg?style=for-the-badge" alt="Built with Love"/>
</p>

# Mamba

This is the Mamba programming language. 
The Documentation can be found [here](https://joelabrahams.nl/mamba_doc).
This documentation outlines the different language features, and also contains a formal specification of the language.

In short, Mamba is like Python, but with a few key features:
-   Strict typing rules, but with type inference so it doesn't get in the way too much, and type refinement features.
-   Null safety features.
-   More explicit error handling.
-   Clear distinction between state and mutability, and immutability and statelessness.

This is a transpiler, written in [Rust](https://www.rust-lang.org/), which converts Mamba source code to Python source files.
Mamba code should therefore, in theory, be interoperable with Python code.
Functions written in Python can be called in Mamba and vice versa.

## 👥 Contributing

Before submitting your first issue or pull request, please take the time to read both our [contribution guidelines](CONTRIBUTING.md) and our [code of conduct](CODE_OF_CONDUCT.md).

## 🔨 Tooling

Several tools are used to help maintain the quality of the codebase.
These tools are used by the continuous integration tools to statically check submitted code.
Therefore, to save time, it is a good idea to install these tools locally and run them before pushing your changes.

### Rustfmt

[Rustfmt](https://github.com/rust-lang/rustfmt) formats Rust code and ensures the formatting is consistent across the codebase.

-   **To install** run `rustup component add rustfmt --toolchain nightly`
-   **To run** run `cargo +nightly fmt`

The configuration of `Rustfmt` can be found in `.rustfmt.toml`.

*Note* The nightly build of `cargo` must be used.
To install the nightly build of `cargo`, run `rustup install nightly`.

### Clippy

[Clippy](https://github.com/rust-lang/rust-clippy) catches common mistakes made in Rust.

-   **To install** 
    - make sure you have the latest version of `rustup` by running `rustup update`
    - run `rustup component add clippy`
-   **To run** run `cargo clippy`

The configuration of `Clippy` can be found in `.clippy.toml`.

*Note* The stable build of `cargo` must be used.
This is installed by default but in case it isn't, run `rustup install stable`.

# 👥 Contributing Guidelines

Contributions are encouraged!

A good place to start is by looking at [good first issues](https://github.com/JSAbrahams/mamba/labels/good%20first%20issue).

Please read our [code of conduct](/CODE_OF_CONDUCT.md) before contributing.

## 📝 Procedures

We standardise the process of creating issues and pull requests to make it easier to manage the project.
Please adhere to these standards.

*Note* In general, it is better to only comment on open pull requests and issues.
Comments on closed issues and pull requests are likely to be ignored.

### ❗ Submitting an Issue

-   Do use one of the provided templates if possible.
-   Do check if there are already similar issues or pull requests before submitting a new issue.
-   Do add an issue to the relevant project if applicable.
-   Do assign the relevant milestone to an issue if applicable.
-   Do reference other issues if applicable.

### ❓ Submitting a Pull Request

-   Do use the provided template.
-   Do check that there are no other pull requests that are doing the same thing. 
    -   If you think your solution is better than an existing pull request, it is better to comment there first and engage in discussion before opening your own pull request.
-   Do reference other issues and pull requests that are relevant
-   Do actively engage with the comments on the pull request. 
    -   An active discussion might lead to an even better solution or new ideas!
-   Do make sure the build passes, ideally by running `rustfmt` and `clippy` locally before pushing.
-   Do add tests when fixing a bug or adding new functionality.
-   Do make sure that this PR is targets one single issue, as large pull requests are difficult to review and unlikely to get merged
-   Do make sure that the base branch is the correct branch by observing the Git branching model.

### Git Branching Model

For all the below, every merge is preceded by a Pull Request.

-   New featuers branch off and are merged with the development branch.
-   Large features get their own branch, and sub-features branch from this branch and are merged with this branch, before the feature branch is merged with develop.
-   Once develop has amassed enough features for a new release, we new branch is created where a release is staged (e.g. `v0.3.1`).
-   Once approved, the new feature branch is merged with master.
-   The new merged release is tagged, and a new release is published on GitHub and published to [Cargo](https://crates.io/crates/mamba).

## 🔨 Tooling

Several tools are used to help maintain the quality of the codebase.
These tools are used by the continuous integration tools to statically check submitted code.
Therefore, to save time, it is a good idea to install these tools locally and run them before pushing your changes.

### Rustfmt

[Rustfmt](https://github.com/rust-lang/rustfmt) formats Rust code and ensures the formatting is consistent across the codebase.

-   **To install** `rustup component add rustfmt --toolchain nightly`
-   **To run** `cargo +nightly fmt`

The configuration of `Rustfmt` can be found in `.rustfmt.toml`.

*Note* The nightly build of `cargo` must be used (`rustup install nightly`).

### Clippy

[Clippy](https://github.com/rust-lang/rust-clippy) catches common mistakes made in Rust.

-   **To install** `rustup component add clippy`
-   **To run** `cargo clippy`

The configuration of `Clippy` can be found in `.clippy.toml`.

*Note* The stable build of `cargo` must be used (`rustup install stable`).

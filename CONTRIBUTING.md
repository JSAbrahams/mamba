# 👥 Contributing Guidelines

Contributions are encouraged!

A good place to start is by looking
at [good first issues](https://github.com/JSAbrahams/mamba/labels/good%20first%20issue).

Please read our [code of conduct](/CODE_OF_CONDUCT.md) before contributing.

## 🔨 Tooling

Several tools are used to help maintain the quality of the codebase.
These tools are used by the continuous integration tools to statically check submitted code.
Therefore, to save time, it is a good idea to install these tools locally and run them before pushing your changes.

To get up and running, make sure:

- You have git installed.
- Ideally, you have Bash 4.0 or greater.
- You have [rustup](https://www.rust-lang.org/tools/install) installed.

All other tooling is downloaded automatically, such as [cargo clippy](https://doc.rust-lang.org/clippy/usage.html) and [rustfmt](https://github.com/rust-lang/rustfmt).

To make sure you always run the githooks:

```sh
git config core.hooksPath .githooks
```

Alternatively, if you have nix (shell) installed, you can use the nix flake which sets up everything for you:

```sh
nix develop
```

Flakes are still a Nix experimental feature, so this needs them enabled — either add
`experimental-features = nix-command flakes` to your `~/.config/nix/nix.conf` (or `/etc/nix/nix.conf`) once, or pass
`--extra-experimental-features 'nix-command flakes'` to the command above.

### Installing Nix

Nix is distributed as a small installer script on nixos.org.
Below are the recommended commands.
If you are not running a Linux distro, this will probably not work for you.

```sh
sh <(curl --proto '=https' --tlsv1.2 -L https://nixos.org/nix/install) --daemon
```

### 🧪 Tests and coverage

The test suite needs a Python 3.10 on `PATH` (it shells out to `python3.10` by name on Linux), since generated Python
is validated by compiling and running it. The nix flake provides this.

```sh
cargo test --package mamba    # run the full suite
```

CI runs the suite with [nextest](https://nexte.st/) instead, so to reproduce a CI run exactly:

```sh
cargo nextest run --package mamba --config-file .config/nextest.toml --profile ci
```

Coverage is measured with [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov), using the same exclusions as
CI/Codecov (see `.github/workflows/coverage.yml`):

```sh
cargo llvm-cov --lcov --output-path ./target/lcov.info \
  --ignore-filename-regex '(^|/)tests?/.*|(^|/)tests_util/.*|.*_tests\.rs$'
```

Both `cargo-nextest` and `cargo-llvm-cov` are provided by the nix flake. Before treating an uncovered line as a
missing test, read [tests/README.md](./tests/README.md) — it records which lines are uncovered by design, which are
gated on unfinished features, and a few traps worth knowing about.

### 🔤 Cargo.toml ordering

Dependencies in `Cargo.toml` must stay in alphabetical order, which the `pre-commit` hook enforces with
[cargo-sort](https://github.com/DevinR528/cargo-sort) (also provided by the nix flake):

```sh
cargo sort --check   # report only, what the hook runs
cargo sort           # rewrite Cargo.toml into sorted order
```

Note the `--check`: a bare `cargo sort` sorts the file in place and exits successfully, so it is the fixing command,
not the checking one.

## 📝 Procedures

We standardise the process of creating issues and pull requests to make it easier to manage the project. Please adhere
to these standards.

*Note* In general, it is better to only comment on open pull requests and issues. Comments on closed issues and pull
requests are likely to be ignored.

### ❗ Submitting an Issue

- Do use one of the provided templates if possible.
- Do check if there are already similar issues or pull requests before submitting a new issue.
- Do assign the relevant milestone to an issue if applicable.
- Do reference other issues if applicable.

### ❓ Submitting a Pull Request

- Do use the provided template.
- Do check that there are no other pull requests that are doing the same thing.
  If you think your solution is better than an existing pull request, it is better to comment there first and engage
  in discussion before opening your own pull request.
- Do reference other issues and pull requests that are relevant.
- Do actively engage with the comments on the pull request.
  An active discussion might lead to an even better solution or new ideas.
- Do add tests when fixing a bug or adding new functionality.
- Do make sure that this PR is targets one single issue, as large pull requests are difficult to review and unlikely to
  be merged.
- Do make sure that the base branch is the correct branch:
  - The base branch will never be `main` (unless you are a core contributor).
  - The base branch generally will be `develop`.
  - If there is a specific feature branch (though unlikely in practice), do use this as a base branch instead if relevant.

### Git Branching Model

For all the below, every merge is preceded by a Pull Request.

- New featuers branch off and are merged with the development branch.
- Large features get their own branch, and sub-features branch from this branch and are merged with this branch, before
  the feature branch is merged with develop.
- Once develop has amassed enough features for a new release, we new branch is created where a release is staged (
  e.g. `v0.3.1`).
- Once approved, the new feature branch is merged with master.
- The new merged release is tagged, and a new release is published on GitHub and published
  to [Cargo](https://crates.io/crates/mamba).

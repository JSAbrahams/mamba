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

Alternatively, you can use [Devbox](https://www.jetify.com/devbox).
This is the recommended path.
It sets all of the above up for you and pins every version.
You therefore do not need rustup, or any of the cargo helpers, installed yourself.

### 📦 Using Devbox

Devbox reads [`devbox.json`](./devbox.json) at the project root.
It gives you a shell containing exactly the tools this project needs:

- the Rust toolchain (`rustc`, `cargo`, `rustfmt`, `clippy`) and `rust-analyzer`
- `cargo-nextest`, `cargo-llvm-cov` and `cargo-sort`, which the git hooks and CI call
- Python 3.10, which the test suite shells out to (see [Tests and coverage](#-tests-and-coverage))
- `nushell` and `starship`, so you get the project's shell and prompt
- the odds and ends: `clang` (its `cc` links the `--bin` output), `llvm`, `git`, `jq`, `direnv`, an editor

Resolved versions are locked in `devbox.lock`, which is committed.
Every contributor and CI therefore get identical versions.
That file is Devbox's equivalent of `Cargo.lock`.
Do not edit it by hand.
It is updated by `devbox add` and `devbox update`.

#### Installing Nix

Devbox is a convenience layer over the [Nix](https://nixos.org/) package manager.
Nix must therefore be installed first.
Devbox offers to install it on first run.
Doing it yourself up front avoids an interactive prompt.
That also makes it the right order in containers and CI.

Nix is distributed as a small installer script on nixos.org.
Below are the recommended commands.
If you are not running a Linux distro, this will probably not work for you.

```sh
sh <(curl --proto '=https' --tlsv1.2 -L https://nixos.org/nix/install) --daemon
```

#### Installing Devbox

```sh
curl -fsSL https://get.jetify.com/devbox | bash
```

This downloads a single static binary into `/usr/local/bin`.
That is why it asks for `sudo`.
Run it as your normal user rather than as root.
Devbox installs Nix in single-user mode on Linux if it has to, and that needs a non-root user.

#### Starting Devbox

```sh
devbox shell     # enter the environment (nushell + starship)
exit             # leave it again
```

The first `devbox shell` is slow, since it downloads every pinned package.
Subsequent runs are near-instant.
Entering the shell also runs `git config core.hooksPath .githooks` for you.
The hooks above are therefore configured automatically.
It sets `push.autoSetupRemote` too, so pushing a new branch does not need `--set-upstream`.

To run a single command in the environment without entering the shell, use `devbox run`.
The named scripts are defined under `shell.scripts` in `devbox.json`:

```sh
devbox run build       # cargo build
devbox run test        # cargo test --package mamba
devbox run test-ci     # the nextest invocation CI uses
devbox run lint        # cargo fmt --check, clippy, cargo sort --check
devbox run coverage    # cargo llvm-cov with CI's exclusions
devbox run precommit   # every cargo command the pre-commit hook runs
```

`devbox run -- <any command>` works too, for anything without a named script.

#### Adding or updating a package

```sh
devbox add <package>@<version>   # e.g. devbox add cargo-audit@latest
devbox update                    # refresh resolved versions in devbox.lock
devbox search <package>          # find a package and its available versions
```

Packages are resolved through [Nixhub](https://www.nixhub.io/).
Anything in nixpkgs is therefore available, usually at a choice of versions.
Prefer pinning an exact version for anything that affects build output, such as the toolchain and Python.
Use `@latest` for incidental tooling.

### 🧪 Tests and coverage

The test suite needs a Python 3.10 on `PATH`.
It shells out to `python3.10` by name on Linux.
This is because generated Python is validated by compiling and running it.
Devbox provides this.

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

Both `cargo-nextest` and `cargo-llvm-cov` are provided by Devbox.
Before treating an uncovered line as a missing test, read [tests/README.md](./tests/README.md).
It records which lines are uncovered by design, and which are gated on unfinished features.
It also records a few traps worth knowing about.

### 🔤 Cargo.toml ordering

Dependencies in `Cargo.toml` must stay in alphabetical order, which the `pre-commit` hook enforces with
[cargo-sort](https://github.com/DevinR528/cargo-sort) (also provided by Devbox):

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

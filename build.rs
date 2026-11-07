//! Sets `MAMBA_VERSION`, the version that `mamba::version()` and `mamba --version` give.
//!
//! It is the output of `git describe --tags`, such as `v0.3.6` on a tag or `v0.3.6-12-g3f2a1bc` after one.
//! Without git, or without tags, as in a build from crates.io or a shallow clone, it is the version in Cargo.toml.

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("set by cargo"));
    println!("cargo:rerun-if-changed=build.rs");
    watch_git_env();

    let version = if is_repository_root(&manifest_dir) {
        watch_git(&manifest_dir);
        git(&manifest_dir, &["describe", "--tags"])
    } else {
        None
    };
    let version = version.unwrap_or_else(|| env::var("CARGO_PKG_VERSION").expect("set by cargo"));
    println!("cargo:rustc-env=MAMBA_VERSION={version}");
}

/// Whether `dir` is the root of a git work tree.
///
/// A crate unpacked inside some other repository, as a vendored dependency is, must not take that repository's version.
fn is_repository_root(dir: &Path) -> bool {
    let top = git(dir, &["rev-parse", "--show-toplevel"]).map(PathBuf::from);
    match (
        top.and_then(|top| top.canonicalize().ok()),
        dir.canonicalize(),
    ) {
        (Some(top), Ok(dir)) => top == dir,
        _ => false,
    }
}

/// Rebuild when `GIT_DIR` or `GIT_WORK_TREE` changes, since git reads them to find the repository it describes.
fn watch_git_env() {
    println!("cargo:rerun-if-env-changed=GIT_DIR");
    println!("cargo:rerun-if-env-changed=GIT_WORK_TREE");
}

/// Rebuild when a commit, a checkout or a new tag changes what `git describe` gives.
///
/// In a linked worktree, HEAD lives in its own git directory and the refs in the common one.
/// A path that does not exist, such as a missing packed-refs, is not watched, since that would rerun this every build.
fn watch_git(dir: &Path) {
    let git_dirs = ["--git-dir", "--git-common-dir"]
        .iter()
        .filter_map(|flag| git(dir, &["rev-parse", flag]))
        .map(|git_dir| dir.join(git_dir));
    let watched =
        git_dirs.flat_map(|git_dir| ["HEAD", "refs", "packed-refs"].map(|file| git_dir.join(file)));

    for path in watched.filter(|path| path.exists()) {
        println!("cargo:rerun-if-changed={}", path.display());
    }
}

/// The trimmed output of `git args` in `dir`, or [None] if git is missing or fails.
fn git(dir: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .ok()?;
    let text = String::from_utf8(output.stdout).ok()?;
    let text = text.trim();
    (output.status.success() && !text.is_empty()).then(|| String::from(text))
}

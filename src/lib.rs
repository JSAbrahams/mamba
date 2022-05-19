#![feature(drain_filter)]

extern crate ansi_term;
#[macro_use]
extern crate log;
extern crate loggerv;

use std::fs::create_dir;
use std::path::{Path, PathBuf};

use crate::check::check_all;
use crate::generate::gen_all;
use crate::parse::{parse_all, ParseInput};

pub mod common;

pub mod check;
pub mod generate;
pub mod parse;

pub mod io;

const TARGET: &str = "target";
const SOURCE: &str = "src";

#[cfg(test)]
mod test_util {
    // Manual include, otherwise, we have to make this part of the interface to make use of these
    // utility functions in tests. We don't want this to be part of the interface.
    include!("../tests/common.rs");
}

/// Convert `*.mamba` files to `*.py`.
///
/// For input, the rules are as follows:
/// If file, file taken as input.
/// If directory, recursively search all sub-directories for mamba files.
/// If no input given, current directory taken is directory.
///
/// For output, the rules are as follows:
/// Output directory to story mamba files.
/// Output directory structure reflects input directory structure.
/// If no output given, target directory created in current directory and output
/// stored here.
pub fn transpile_directory(
    current_dir: &Path,
    source: Option<&str>,
    target: Option<&str>,
) -> Result<PathBuf, Vec<(String, String)>> {
    let src_path = source.map_or(current_dir.join(SOURCE), |p| current_dir.join(p));
    if !src_path.is_file() && !src_path.is_dir() {
        let msg = format!("Source directory does not exist: {}", src_path.as_os_str().to_str().unwrap());
        return Err(vec![(String::from("pathfinding"), msg)]);
    } else if src_path.is_file() && !src_path.exists() {
        let msg = format!("Source file does not exist: {}", src_path.as_os_str().to_str().unwrap());
        return Err(vec![(String::from("pathfinding"), msg)]);
    }

    let out_dir = current_dir.join(target.unwrap_or(TARGET));
    if !out_dir.exists() {
        create_dir(&out_dir).map_err(|e| vec![(String::from("io"), e.to_string())])?;
    }
    info!("Input is '{}'", src_path.display());
    info!("Output will be stored in '{}'", out_dir.display());

    let relative_paths = io::relative_files(src_path.as_path()).map_err(|error| vec![error])?;
    let in_absolute_paths = if src_path.is_dir() {
        relative_paths.iter().map(|os_string| src_path.join(os_string)).collect()
    } else {
        vec![src_path.clone()]
    };
    let out_absolute_paths: Vec<PathBuf> =
        relative_paths.iter().map(|os_string| out_dir.join(os_string)).collect();

    info!(
        "Transpiling {} file {}",
        out_absolute_paths.len(),
        if out_absolute_paths.len() > 1 { "s" } else { "" }
    );

    let mut sources = vec![];
    for source_path in in_absolute_paths.clone() {
        let source = io::read_source(&source_path).map_err(|error| vec![error])?;
        sources.push(source);
    }

    let source_pairs = sources.iter().zip(in_absolute_paths.iter());
    let source_option_pairs: Vec<_> =
        source_pairs.map(|(source, path)| (source.clone(), Some(path.clone()))).collect();
    let mamba_source = mamba_to_python(source_option_pairs.as_slice(), &src_path)?;

    for (source, out_path) in mamba_source.iter().zip(out_absolute_paths) {
        let out_path = out_path.with_extension("py");
        io::write_source(source, &out_path).map_err(|error| vec![error])?;
    }

    Ok(out_dir)
}

/// Convert mamba source to python source.
///
/// For each mamba source, a path can optionally be given for display in error
/// messages. This path is not necessary however.
pub fn mamba_to_python(
    source: &[(String, Option<PathBuf>)],
    source_dir: &PathBuf,
) -> Result<Vec<String>, Vec<(String, String)>> {
    // Strip until source
    let strip_prefix = |p: PathBuf| {
        p.strip_prefix(source_dir)
            .map(|p| {
                PathBuf::from(&source_dir.iter().last().unwrap_or_else(|| "".as_ref())).join(p)
            })
            .unwrap_or(p)
    };
    let source: Vec<ParseInput> =
        source.iter().map(|(src, path)| (src.clone(), path.clone().map(strip_prefix))).collect();

    let asts = parse_all(&source).map_err(|errs| {
        errs.iter()
            .map(|err| (String::from("syntax"), format!("{}", err)))
            .collect::<Vec<(String, String)>>()
    })?;
    trace!("Parsed {} files", asts.len());

    let modified_trees = check_all(asts.as_slice()).map_err(|errs| {
        errs.iter()
            .map(|err| (String::from("type"), format!("{}", err)))
            .collect::<Vec<(String, String)>>()
    })?;
    trace!("Checked {} files", modified_trees.len());

    let core_tree = gen_all(modified_trees.as_slice()).map_err(|errs| {
        errs.iter()
            .map(|err| (String::from("unimplemented"), format!("{}", err)))
            .collect::<Vec<(String, String)>>()
    })?;
    trace!("Converted {} checked files to Python", core_tree.len());

    Ok(core_tree.iter().map(|(core, ..)| core.to_source()).collect())
}

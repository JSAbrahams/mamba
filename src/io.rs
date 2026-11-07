//! Compile Mamba projects on disk, with the `native` feature.

use std::ffi::OsString;
use std::fs;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use glob::glob;
use log::info;
use pathdiff::diff_paths;

use crate::backend::cranelift::link;
use crate::{compile, Artifact, Diagnostic, Options, Output, Source, Stage};

/// The source directory when none is given.
const SOURCE: &str = "src";
/// The output directory of Python and assembly when none is given.
const TARGET: &str = "target";
/// The executable when none is given.
const EXECUTABLE: &str = "a.out";

/// The Mamba sources of a project, read from disk.
#[derive(Debug, Clone)]
pub struct Project {
    /// The source file or directory.
    pub path: PathBuf,
    /// The path of each source relative to [Project::path], or its file name if that is a file.
    pub relative: Vec<OsString>,
    /// Each source, named for diagnostics by its path from the parent of [Project::path].
    pub sources: Vec<Source>,
}

/// Read the `*.mamba` files of `src`, relative to `dir`, or of `dir/src` if `src` is [None].
///
/// If `src` is a directory, its sub-directories are searched too.
pub fn read_project(dir: &Path, src: Option<&str>) -> Result<Project, Vec<Diagnostic>> {
    let path = dir.join(src.unwrap_or(SOURCE));
    if !path.exists() {
        let msg = format!("Source directory does not exist: {}", path.display());
        return Err(vec![Diagnostic::error(Stage::Io, msg)]);
    }

    let relative = relative_files(&path).map_err(io_error)?;
    let absolute: Vec<PathBuf> = if path.is_dir() {
        relative.iter().map(|file| path.join(file)).collect()
    } else {
        vec![path.clone()]
    };
    info!(
        "Compiling {} file{}",
        absolute.len(),
        if absolute.len() == 1 { "" } else { "s" }
    );

    let sources = absolute
        .iter()
        .map(|file| {
            let text = read_source(file).map_err(io_error)?;
            Ok(Source::named(display_name(file, &path), text))
        })
        .collect::<Result<_, Vec<Diagnostic>>>()?;

    Ok(Project {
        path,
        relative,
        sources,
    })
}

/// Compile the project `src` in `dir`, see [read_project], and write the output to `out`, relative to `dir`.
///
/// - [Output::Python] writes a `.py` file for each source, in a directory tree like that of the sources.
///   `out` is that directory, `target` if [None].
/// - [Output::Asm] writes a `.s` file for each source in the same way.
/// - [Output::Object] links the objects into an executable with `cc`.
///   `out` is the executable, `a.out` if [None].
///
/// Returns the output directory or executable.
pub fn compile_dir(
    dir: &Path,
    src: Option<&str>,
    out: Option<&str>,
    options: &Options,
) -> Result<PathBuf, Vec<Diagnostic>> {
    let project = read_project(dir, src)?;
    let artifacts = compile(&project.sources, options)?;
    write_output(dir, out, &project, artifacts, &options.output)
}

/// Write what [compile] made of `project` to `out`, relative to `dir`, as [compile_dir] does.
pub fn write_output(
    dir: &Path,
    out: Option<&str>,
    project: &Project,
    artifacts: Vec<Artifact>,
    output: &Output,
) -> Result<PathBuf, Vec<Diagnostic>> {
    match output {
        Output::Python => write_tree(&dir.join(out.unwrap_or(TARGET)), project, &artifacts, "py"),
        Output::Asm { .. } => {
            write_tree(&dir.join(out.unwrap_or(TARGET)), project, &artifacts, "s")
        }
        Output::Object { .. } => link_executable(&dir.join(out.unwrap_or(EXECUTABLE)), artifacts),
    }
}

/// Write each artifact to `out_dir`, at the path of its source with extension `extension`.
fn write_tree(
    out_dir: &Path,
    project: &Project,
    artifacts: &[Artifact],
    extension: &str,
) -> Result<PathBuf, Vec<Diagnostic>> {
    if !out_dir.exists() {
        fs::create_dir(out_dir).map_err(|err| io_error(format!("{err}: {}", out_dir.display())))?;
    }
    info!("Output will be stored in '{}'", out_dir.display());

    for (artifact, relative) in artifacts.iter().zip(&project.relative) {
        let out_path = out_dir.join(relative).with_extension(extension);
        let text = artifact.as_text().unwrap_or_default();
        write_source(text, &out_path).map_err(io_error)?;
    }
    Ok(out_dir.to_path_buf())
}

/// Link object files into the executable `out_file`.
fn link_executable(out_file: &Path, artifacts: Vec<Artifact>) -> Result<PathBuf, Vec<Diagnostic>> {
    if let Some(parent) = out_file.parent() {
        if !parent.exists() {
            fs::create_dir(parent).map_err(|err| io_error(err.to_string()))?;
        }
    }
    info!(
        "Output executable will be stored at '{}'",
        out_file.display()
    );

    let mut object_files = vec![];
    for artifact in artifacts {
        let mut file = tempfile::Builder::new()
            .suffix(".o")
            .tempfile()
            .map_err(|err| io_error(err.to_string()))?;
        file.write_all(&artifact.into_bytes())
            .map_err(|err| io_error(err.to_string()))?;
        object_files.push(file.into_temp_path());
    }

    link::link(&object_files, out_file).map_err(io_error)?;
    Ok(out_file.to_path_buf())
}

/// The name of `file` in diagnostics: its path from the parent of the source directory `source_dir`.
fn display_name(file: &Path, source_dir: &Path) -> PathBuf {
    match file.strip_prefix(source_dir) {
        // The source is a file, not a directory.
        Ok(relative) if relative.as_os_str().is_empty() => file
            .file_name()
            .map_or_else(|| file.to_path_buf(), PathBuf::from),
        Ok(relative) => source_dir.file_name().map_or_else(
            || relative.to_path_buf(),
            |name| Path::new(name).join(relative),
        ),
        Err(_) => file.to_path_buf(),
    }
}

fn io_error(msg: impl Into<String>) -> Vec<Diagnostic> {
    vec![Diagnostic::error(Stage::Io, msg)]
}

pub fn read_source(source_path: &Path) -> Result<String, String> {
    let mut source = String::new();
    OpenOptions::new()
        .read(true)
        .open(source_path)
        .map_err(|e| format!("{}: {}", e, source_path.display()))?
        .read_to_string(&mut source)
        .map_err(|e| format!("{}: {}", e, source_path.display()))?;
    Ok(source)
}

pub fn write_source(source: &str, out_path: &Path) -> Result<usize, String> {
    match out_path.parent() {
        Some(parent) => {
            fs::create_dir_all(parent).map_err(|e| format!("{}: {}", e, parent.display()))?
        }
        None => return Err(format!("No parent directory: {}", out_path.display())),
    };

    // LF instead of CRLF line endings
    let source = source.replace("\r\n", "\n");
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(out_path)
        .map_err(|e| format!("{}: {}", e, out_path.display()))?
        .write(source.as_ref())
        .map_err(|e| format!("{}: {}", e, out_path.display()))
}

/// Get all `*.mamba` files paths relative to given path.
///
/// If path is file, return file name.
/// If directory, return all `*.mamba` files as relative paths to given path.
pub fn relative_files(in_path: &Path) -> Result<Vec<OsString>, String> {
    if in_path.is_file() {
        let in_file_name = in_path.file_name().unwrap_or_else(|| unreachable!());
        return Ok(vec![in_file_name.to_os_string()]);
    }

    let pattern_path = in_path.to_owned().join("**").join("*.mamba");
    let pattern = pattern_path.as_os_str().to_string_lossy();
    let glob =
        glob(pattern.as_ref()).map_err(|e| format!("Unable to recursively find files: {e}"))?;

    let mut relative_paths = vec![];
    for absolute_result in glob {
        let absolute_path = absolute_result.map_err(|e| e.to_string())?;
        let relative_path = diff_paths(absolute_path.as_path(), in_path)
            .ok_or_else(|| String::from("Unable to create relative path"))?;
        relative_paths.push(relative_path.into_os_string());
    }

    Ok(relative_paths)
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use crate::io::compile_dir;
    use crate::{Options, Stage};

    #[test]
    fn compile_dir_reports_missing_source_directory() {
        let dir = Path::new("/does/not/exist/anywhere");
        let errs = compile_dir(dir, None, None, &Options::default()).unwrap_err();

        assert_eq!(errs.len(), 1);
        assert_eq!(errs[0].stage, Stage::Io);
        assert!(errs[0].message.contains("Source directory does not exist"));
    }
}

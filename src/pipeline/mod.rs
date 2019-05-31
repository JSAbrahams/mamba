use crate::core::to_source;
use crate::desugar::desugar;
use crate::lexer::tokenize;
use crate::parser::parse;
use crate::type_checker::check;
use glob::glob;
use leg::*;
use pathdiff::diff_paths;
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

const OUT_FILE: &str = "target";

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
pub fn mamba_to_python(
    current_dir: &Path,
    in_path: Option<&str>,
    out_path: Option<&str>
) -> PathBuf {
    let out_dir = current_dir.join(out_path.unwrap_or(OUT_FILE));
    info(format!("Transpiling to {:#?}", out_dir).as_str(), None, None);

    let relative_file_paths = match in_path {
        Some(in_path) =>
            if Path::new(in_path).is_dir() {
                match relative_paths(current_dir.join(Path::new(in_path)).as_path()) {
                    Ok(relative) => Ok(relative
                        .iter()
                        .map(|p| Path::new(in_path).join(p).into_os_string())
                        .collect()),
                    err => err
                }
            } else {
                Ok(vec![OsString::from(in_path)])
            },
        None => relative_paths(current_dir)
    };

    match relative_file_paths {
        Ok(relative_file_paths) =>
            match pipeline(current_dir, out_dir.as_path(), relative_file_paths.as_slice()) {
                Ok(..) => success(format!("Transpiled to {:#?}", out_dir).as_str(), None, None),
                Err(err) => error(err.as_str(), None, None)
            },
        Err(err) => error(err.as_str(), None, None)
    };

    out_dir.to_owned()
}

/// Get all `*.mamba` files paths relative to given directory.
fn relative_paths(in_dir: &Path) -> Result<Vec<OsString>, String> {
    let pattern_path = in_dir.to_owned().join("**").join("*.mamba");
    let pattern = pattern_path.as_os_str().to_string_lossy();

    let mut relative_paths = vec![];
    for absolute_result in glob(pattern.as_ref()).map_err(|e| format!("OH NO {:#?}", e))? {
        let absolute_path = absolute_result.map_err(|e| e.to_string())?;
        let relative_path = diff_paths(absolute_path.as_path(), in_dir).ok_or("")?;
        relative_paths.push(relative_path.into_os_string());

        info(format!("{:#?}", absolute_path).as_str(), Some("search"), None);
    }

    Ok(relative_paths)
}

pub fn pipeline(
    in_dir: &Path,
    out_dir: &Path,
    relative_file_paths: &[OsString]
) -> Result<(), String> {
    let mut ast_trees = vec![];
    for in_path in relative_file_paths.iter().map(|p| in_dir.join(p)) {
        let mut input_file =
            OpenOptions::new().read(true).open(in_path).map_err(|e| e.to_string())?;

        let mut source = String::new();
        input_file.read_to_string(&mut source).map_err(|e| e.to_string())?;

        let tokens = tokenize(source.as_ref())?;
        ast_trees.push(*parse(&tokens).map_err(|e| e.to_string())?);
    }

    let typed_ast_trees = check(ast_trees.as_slice())?;
    debug_assert_eq!(typed_ast_trees.len(), relative_file_paths.len());

    let mut i = 0;
    for typed_ast_tree in typed_ast_trees.iter() {
        let core_tree = desugar(&typed_ast_tree);
        let source = to_source(&core_tree);

        let out_path = out_dir.join(&relative_file_paths[i]).with_extension("py");

        wait(format!("Writing {:#?}", out_path).as_str(), Some("out"), None);

        let mut output_file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(out_path.clone())
            .map_err(|e| e.to_string())?;

        output_file.write_all(source.as_ref()).map_err(|e| e.to_string())?;
        success(format!("Wrote {:#?} successfully", out_path).as_str(), Some("out"), None);

        i += 1;
    }

    Ok(())
}

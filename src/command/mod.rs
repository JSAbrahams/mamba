use crate::core::to_py_source;
use crate::desugarer::desugar;
use crate::lexer::tokenize;
use crate::parser::parse;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

pub fn mamba_to_python_direct(input_path: &Path) -> Result<PathBuf, String> {
    let file_path = match input_path.parent() {
                        Some(parent) => parent,
                        None =>
                            return Err(format!("Input was not in a directory: {}",
                                               input_path.to_string_lossy())),
                    }.join(match input_path.file_stem() {
                               Some(path) => path,
                               None =>
                                   return Err(format!("Input file did not have a name: {}",
                                                      input_path.to_string_lossy())),
                           });

    let output_path_string = format!("{}.py", file_path.to_string_lossy());
    let output_path = Path::new(&output_path_string);
    match mamba_to_python(input_path, output_path) {
        Ok(output_path) => Ok(output_path),
        Err(err) => Err(err)
    }
}

pub fn mamba_to_python(input: &Path, output: &Path) -> Result<PathBuf, String> {
    let res_output = output.to_owned();

    let input_file_option = OpenOptions::new().read(true).open(input);
    let output_file_options = OpenOptions::new().write(true).create(true).open(output);

    let (mut input_file, mut output_file) = match (input_file_option, output_file_options) {
        (Ok(input_file), Ok(output_file)) => (input_file, output_file),
        (Err(err), _) => return Err(err.to_string()),
        (_, Err(err)) => return Err(err.to_string())
    };

    let mut input_string = String::new();
    match input_file.read_to_string(&mut input_string) {
        Ok(_) => (),
        Err(err) => return Err(err.to_string())
    }

    let output_string = match transpile(&input_string) {
        Ok(python) => python,
        Err(err) => return Err(err.to_string())
    };

    match output_file.write(output_string.as_ref()) {
        Ok(_) => Ok(res_output),
        Err(err) => Err(format!("{}", err))
    }
}

fn transpile(input: &str) -> Result<String, String> {
    let tokens = match tokenize(input) {
        Ok(tokens) => tokens,
        Err(err) => return Err(err)
    };

    let ast_tree = match parse(&tokens) {
        Ok(ast_tree) => ast_tree,
        Err(err) => return Err(format!("{}", err))
    };

    let core_tree = desugar(&ast_tree);
    Ok(to_py_source(&core_tree))
}

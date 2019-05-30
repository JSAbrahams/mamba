use crate::core::to_py_source;
use crate::desugar::desugar;
use crate::lexer::tokenize;
use crate::parser::parse;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

/// Transpile a `*.mamba` file to a Python source `*.py` file, which has the
/// same name and is stored in the same directory.
pub fn mamba_to_python_direct(input_path: &Path) -> Result<PathBuf, String> {
    let out_name = input_path
        .file_name()
        .expect(format!("Unable to open {:#?}", input_path.as_os_str()).as_str());
    let output_path = input_path
        .parent()
        .expect(format!("Unable to get parent {:#?}", input_path.as_os_str()).as_str())
        .join(out_name);
    let output_path = Path::new(output_path.as_path()).with_extension("py");

    mamba_to_python(input_path, output_path.as_path())
}

/// Transpile a `*.mamba` file to Python source and store it in the given output
/// directory.
pub fn mamba_to_python(input: &Path, output: &Path) -> Result<PathBuf, String> {
    let mut input_file = OpenOptions::new()
        .read(true)
        .open(input)
        .expect(format!("Unable to open input {:#?}", input).as_str());
    let mut output_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(output)
        .expect(format!("Unable to open output {:#?}", output).as_str());

    let mut input_string = String::new();
    input_file.read_to_string(&mut input_string).expect("Unable to read from input");
    let output_string = transpile(&input_string)?;
    output_file.write(output_string.as_ref()).expect("Unable to write to output");
    Ok(output.to_owned())
}

fn transpile(input: &str) -> Result<String, String> {
    let tokens = tokenize(input)?;
    let ast_tree = parse(&tokens).map_err(|e| e.to_string())?;
    let core_tree = desugar(&ast_tree);
    Ok(to_py_source(&core_tree))
}

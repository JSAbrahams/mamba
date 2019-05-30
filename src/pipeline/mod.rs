use crate::core::to_py_source;
use crate::desugar::desugar;
use crate::lexer::tokenize;
use crate::parser::parse;
use crate::type_checker::check;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

/// Transpile a `*.mamba` file to Python source and store it in the given output
/// directory.
///
/// If output is None, then output is stored alongside `*.mamba` file as a
/// `*.py` file.
pub fn mamba_to_python(input: &Path, output: Option<&Path>) -> Result<PathBuf, String> {
    let output_path = match output {
        Some(output) => output.to_path_buf(),
        None => create_output(input)?
    };

    let mut input_file = OpenOptions::new()
        .read(true)
        .open(input.clone())
        .expect(format!("Unable to open input {:#?}", input).as_str());
    let mut output_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(output_path.clone())
        .expect(format!("Unable to open output {:#?}", output_path).as_str());

    let mut input_string = String::new();
    input_file.read_to_string(&mut input_string).expect("Unable to read from input");
    let output_string = transpile(&input_string)?;
    output_file.write(output_string.as_ref()).expect("Unable to write to output");
    Ok(output_path)
}

fn create_output(input_path: &Path) -> Result<PathBuf, String> {
    let out_name = input_path
        .file_name()
        .expect(format!("Unable to open {:#?}", input_path.as_os_str()).as_str());

    Ok(input_path
        .parent()
        .expect(format!("Unable to get parent {:#?}", input_path.as_os_str()).as_str())
        .join(out_name)
        .with_extension("py"))
}

fn transpile(source: &str) -> Result<String, String> {
    let tokens = tokenize(source)?;
    let ast_tree = parse(&tokens).map_err(|e| e.to_string())?;
    let typed_ast_tree = check(&ast_tree)?;
    let core_tree = desugar(typed_ast_tree);
    Ok(to_py_source(&core_tree))
}

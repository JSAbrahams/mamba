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

    let owned = output_path.to_owned();
    let mut input_file = OpenOptions::new().read(true).open(input).map_err(|e| e.to_string())?;
    let mut output_file =
        OpenOptions::new().write(true).create(true).open(output_path).map_err(|e| e.to_string())?;

    let mut input_string = String::new();
    input_file.read_to_string(&mut input_string).expect("Unable to read from input");
    let input_strings = [input_string];
    let output_string = pipeline(&input_strings)?;
    output_file.write_all(output_string[0].as_ref()).expect("Unable to write to output");
    Ok(owned)
}

fn create_output(input_path: &Path) -> Result<PathBuf, String> {
    let out_name = input_path.file_name().ok_or("Input has no filename.")?;
    Ok(input_path.parent().ok_or("Input has no parent.")?.join(out_name).with_extension("py"))
}

fn pipeline(sources: &[String]) -> Result<Vec<String>, String> {
    let mut ast_trees = vec![];
    let mut out_sources = vec![];

    for source in sources {
        let tokens = tokenize(source.as_ref())?;
        ast_trees.push(*parse(&tokens).map_err(|e| e.to_string())?);
    }

    for typed_ast_tree in check(ast_trees.as_slice())? {
        let core_tree = desugar(typed_ast_tree);
        out_sources.push(to_py_source(&core_tree));
    }

    Ok(out_sources)
}

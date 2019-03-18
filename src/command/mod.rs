use crate::core::to_py_source;
use crate::desugarer::desugar;
use crate::lexer::tokenize;
use crate::parser::parse;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;

pub fn mamba_to_python_direct(path: &Path) -> Result<File, String> {
    let file_path =
        match path.parent() {
            Some(parent) => parent,
            None => return Err(format!("Not in a directory: {}", path.to_string_lossy()))
        }.join(match path.file_stem() {
                   Some(path) => path,
                   None =>
                       return Err(format!("File does not have name: {}", path.to_string_lossy())),
               });

    let new_file_path = format!("{}.py", file_path.to_string_lossy());

    let mut output_file = match File::create(new_file_path) {
        Ok(file) => file,
        Err(err) => return Err(format!("{}", err))
    };

    let mut input_file = match File::open(path) {
        Ok(file) => file,
        Err(err) => return Err(format!("{}", err))
    };

    match mamba_to_python(&mut input_file, &mut output_file) {
        Ok(_) => Ok(output_file),
        Err(err) => Err(err)
    }
}

pub fn mamba_to_python(input: &mut File, output: &mut File) -> Result<(), String> {
    let mut input_string = String::new();
    match input.read_to_string(&mut input_string) {
        Ok(_) => (),
        Err(err) => return Err(format!("{}", err))
    }

    let output_string = match transpile(&input_string) {
        Ok(python) => python,
        Err(err) => return Err(format!("{}", err))
    };

    match output.write(output_string.as_ref()) {
        Ok(_) => Ok(()),
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

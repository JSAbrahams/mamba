use crate::core::to_py_source;
use crate::desugarer::desugar;
use crate::lexer::tokenize;
use crate::parser::parse;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;

pub fn quick_transpile(path: &Path) -> File {
    let new_file_path = format!("{}.py", match path.parent() {
        Some(parent) => parent,
        None => panic!()
    }.join(match path.file_stem() {
        Some(path) => path,
        None => panic!()
    }).to_string_lossy());

    println!("{}", new_file_path);

    let mut output_file = match File::create(new_file_path) {
        Ok(file) => file,
        Err(err) => panic!("{}", err)
    };

    let mut input_file = match File::open(path) {
        Ok(file) => file,
        Err(err) => panic!("{}", err)
    };

    transpile_file(&mut input_file, &mut output_file);
    output_file
}

pub fn transpile_file(input: &mut File, output: &mut File) {
    let mut input_string = String::new();
    match input.read_to_string(&mut input_string) {
        Ok(_) => (),
        Err(err) => panic!("{}", err)
    }

    let output_string = match transpile(&input_string) {
        Ok(python) => python,
        Err(err) => panic!("{}", err)
    };

    match output.write(output_string.as_ref()) {
        Ok(_) => (),
        Err(err) => panic!("{}", err)
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

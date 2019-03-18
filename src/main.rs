use std::path::Path;

pub mod command;
pub mod core;
pub mod desugarer;
pub mod lexer;
pub mod parser;
pub mod type_checker;

const INPUT_FLAG: &str = "-i";
const OUTPUT_FLAG: &str = "-o";

fn main() -> Result<(), String> {
    let mut input: Option<String> = None;
    let mut output: Option<String> = None;

    let mut args = std::env::args();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            INPUT_FLAG => input = Option::from(args.next().expect("Expected input file path.")),
            OUTPUT_FLAG => output = Option::from(args.next().expect("Expected output file path.")),

            other => return Err(format!("Flag not recognized: {}", other))
        }
    }

    match (input, output) {
        (Some(input), Some(output)) => match command::mamba_to_python(
            Path::new(&input),
            Path::new(&output)) {
            Ok(_) => Ok(()),
            Err(err) => Err(err)
        },
        (Some(input), None) => match command::mamba_to_python_direct(Path::new(&input)) {
            Ok(_) => Ok(()),
            Err(err) => Err(err)
        },
        _ => Err(String::from("No input file path given."))
    }
}

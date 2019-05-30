use std::env::Args;
use std::path::PathBuf;

const INPUT_FLAG: &str = "-i";
const OUTPUT_FLAG: &str = "-o";

pub struct CLI {
    pub is_directory: bool,
    pub input:        PathBuf,
    pub output:       Option<PathBuf>
}

impl CLI {
    pub fn new(args: &mut Args) -> Result<Self, String> {
        let mut input = None;
        let mut output = None;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                INPUT_FLAG =>
                    input = Some(PathBuf::from(args.next().expect("Expected input file path."))),
                OUTPUT_FLAG =>
                    output = Some(PathBuf::from(args.next().expect("Expected output file path."))),
                _ => ()
            }
        }

        let is_directory = input.clone().expect("Expected input").is_dir() && output.is_none()
            || output.clone().unwrap().is_dir();
        let input = input.expect("Expected input");
        Ok(CLI { is_directory, input, output })
    }
}

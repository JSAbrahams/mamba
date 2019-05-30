use mamba::cli::CLI;
use mamba::pipeline::mamba_to_python;

pub fn main() -> Result<(), String> {
    let cli = CLI::new(&mut std::env::args())?;
    if let Some(out_path) = cli.output {
        mamba_to_python(cli.input.as_path(), Some(out_path.as_path()))?
    } else {
        mamba_to_python(cli.input.as_path(), None)?
    };
    Ok(())
}

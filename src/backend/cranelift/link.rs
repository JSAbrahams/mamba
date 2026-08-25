use std::path::Path;
use std::process::Command;

/// Link the given object files into a single executable at `output`, by shelling out to the
/// system `cc` -- the same approach `rustc` itself uses, rather than reimplementing a linker.
pub fn link(object_paths: &[impl AsRef<Path>], output: &Path) -> Result<(), String> {
    let mut cmd = Command::new("cc");
    cmd.args(object_paths.iter().map(AsRef::as_ref));
    cmd.arg("-o").arg(output);

    let result = cmd
        .output()
        .map_err(|e| format!("Could not run 'cc' to link the executable: {e}"))?;

    if result.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Linking failed:\n{}",
            String::from_utf8_lossy(&result.stderr)
        ))
    }
}

use crate::common::exists_and_delete;
use crate::common::resource_path;
use mamba::pipeline::mamba_to_python;

#[test]
fn mamba_to_python_no_output_verify() -> Result<(), String> {
    let input = resource_path(true, &["class"], "types.mamba");
    mamba_to_python(input.as_ref(), None)?;

    Ok(assert!(exists_and_delete(true, &["class"], "types.py")))
}

#[test]
fn mamba_to_python_verify() -> Result<(), String> {
    let input = resource_path(true, &["class"], "types.mamba");
    let output = resource_path(true, &["class"], "types.py");
    mamba_to_python(input.as_ref(), Some(output.as_ref()))?;

    Ok(assert!(exists_and_delete(true, &["class"], "types.py")))
}

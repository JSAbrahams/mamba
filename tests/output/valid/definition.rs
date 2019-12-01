use crate::common::{exists_and_delete, python_src_to_stmts, resource_content, resource_path};
use crate::output::common::PYTHON;
use mamba::pipeline::transpile_directory;
use std::path::Path;
use std::process::Command;

#[test]
fn long_f_string() -> Result<(), Vec<(String, String)>> {
    transpile_directory(
        &Path::new(resource_path(true, &["definition"], "").as_str()),
        Some("long_f_string.mamba"),
        None
    )?;

    let cmd = Command::new(PYTHON)
        .arg("-m")
        .arg("py_compile")
        .arg(resource_path(true, &["definition", "target"], "long_f_string.py"))
        .output()
        .unwrap();
    if cmd.status.code().unwrap() != 0 {
        panic!("{}", String::from_utf8(cmd.stderr).unwrap());
    }

    let python_src = resource_content(true, &["definition"], "long_f_string_check.py");
    let out_src = resource_content(true, &["definition", "target"], "long_f_string.py");

    let python_ast = python_src_to_stmts(&python_src);
    let out_ast = python_src_to_stmts(&out_src);

    assert_eq!(python_ast, out_ast);
    Ok(assert!(exists_and_delete(true, &["definition", "target"], "long_f_string.py")))
}

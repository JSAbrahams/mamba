use std::path::PathBuf;

use crate::check::context::Context;
use crate::check::context::python::python_files;
use crate::check::result::TypeResult;

impl Context {
    /// Loads pre-defined Python primitives into Context.
    pub fn into_with_primitives(self) -> TypeResult<Self> {
        let python_dir = resource("primitive");
        let (py_types, py_fields, py_functions) = python_files(&python_dir)?;

        let types = self.classes.union(&py_types).cloned().collect();
        let functions = self.functions.union(&py_functions).cloned().collect();
        let fields = self.fields.union(&py_fields).cloned().collect();
        Ok(Context { classes: types, functions, fields })
    }

    /// Loads pre-defined Python standard library into Context.
    pub fn into_with_std_lib(self) -> TypeResult<Self> {
        let python_dir = resource("std");
        let (py_types, py_fields, py_functions) = python_files(&python_dir)?;

        let types = self.classes.union(&py_types).cloned().collect();
        let functions = self.functions.union(&py_functions).cloned().collect();
        let fields = self.fields.union(&py_fields).cloned().collect();
        Ok(Context { classes: types, functions, fields })
    }
}

fn resource(resource: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("check")
        .join("resource")
        .join(resource)
}

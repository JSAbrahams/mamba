use std::collections::HashSet;
use std::path::PathBuf;

use crate::check::context::clss::generic::GenericClass;
use crate::check::context::field::generic::GenericField;
use crate::check::context::function::generic::GenericFunction;
use crate::check::context::python::python_files;
use crate::check::context::Context;
use crate::check::result::TypeResult;

impl Context {
    /// Loads pre-defined Python primitives into Context.
    pub fn into_with_primitives(self) -> TypeResult<Self> {
        let python_dir = resource("primitive");
        let (py_types, py_fields, py_functions) = python_files(&python_dir)?;

        let classes: HashSet<GenericClass> = self.classes.union(&py_types).cloned().collect();
        let functions: HashSet<GenericFunction> =
            self.functions.union(&py_functions).cloned().collect();
        let fields: HashSet<GenericField> = self.fields.union(&py_fields).cloned().collect();

        let python_dir = resource("std");
        let (py_types, py_fields, py_functions) = python_files(&python_dir)?;

        let classes: HashSet<GenericClass> = classes.union(&py_types).cloned().collect();
        let functions: HashSet<GenericFunction> = functions.union(&py_functions).cloned().collect();
        let fields: HashSet<GenericField> = fields.union(&py_fields).cloned().collect();

        Ok(Context {
            classes,
            functions,
            fields,
        })
    }

    /// Loads pre-defined Python standard library into Context.
    pub fn into_with_std_lib(self) -> TypeResult<Self> {
        let python_dir = resource("std");
        let (py_types, py_fields, py_functions) = python_files(&python_dir)?;

        let types = self.classes.union(&py_types).cloned().collect();
        let functions = self.functions.union(&py_functions).cloned().collect();
        let fields = self.fields.union(&py_fields).cloned().collect();
        Ok(Context {
            classes: types,
            functions,
            fields,
        })
    }
}

fn resource(resource: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("check")
        .join("resource")
        .join(resource)
}

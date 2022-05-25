use std::path::PathBuf;

pub trait IntoWithSource {
    fn into_with_source(self, source: &Option<String>, path: &Option<PathBuf>) -> Self;
}

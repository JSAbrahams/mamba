use std::path::PathBuf;

pub trait WithSource {
    fn with_source(self, source: &Option<String>, path: &Option<PathBuf>) -> Self;
}

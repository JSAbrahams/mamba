use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

#[macro_export]
macro_rules! assert_ok {
    ($expr:expr) => {{
        match $expr {
            Ok(_) => (),
            Err(err) => panic!("{}", err)
        }
    }};
}

pub fn valid_resource_content(file: &str) -> String { resource_content("valid", file) }

pub fn valid_resource_path(file: &str) -> String { resource_path("valid", file) }

pub fn invalid_resource_content(file: &str) -> String { resource_content("invalid", file) }

pub fn invalid_resource_path(file: &str) -> String { resource_path("invalid", file) }

fn resource_content(subdir: &str, file: &str) -> String {
    let mut content = String::new();
    let path = resource_path(subdir, file);
    File::open(path).unwrap().read_to_string(&mut content);

    content
}

fn resource_path(subdir: &str, file: &str) -> String {
    let mut source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    source_path.push(if cfg!(windows) {
                         format!("tests\\resources\\{}\\{}", subdir, file)
                     } else {
                         format!("tests/resources/{}/{}", subdir, file)
                     });

    String::from(source_path.to_string_lossy())
}

pub fn check_valid_resource_exists_and_delete(file: &str) -> bool {
    let path_string = valid_resource_path(file);
    remove(&path_string)
}

pub fn check_invalid_resource_exists_and_delete(file: &str) -> bool {
    let path_string = invalid_resource_path(file);
    remove(&path_string)
}

fn remove(path_string: &String) -> bool {
    let path = Path::new(&path_string);
    if path.exists() {
        fs::remove_file(path);
        true
    } else {
        false
    }
}

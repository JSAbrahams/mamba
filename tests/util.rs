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

fn resource_path(file: &String) -> String {
    let mut source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    source_path.push(if cfg!(windows) {
        String::from("tests\\resources\\")
    } else {
        String::from("tests/resources/")
    });
    source_path.push(file);

    String::from(source_path.to_string_lossy())
}

#[clippy::ignore(unused_must_use)]
fn resource_string_content(file: &String) -> String {
    let mut content = String::new();

    let path = resource_path(file);
    File::open(path).unwrap().read_to_string(&mut content);

    content
}

pub fn valid_resource_contents(file: &str) -> String {
    if cfg!(windows) {
        resource_string_content(&format!("{}{}", "valid\\", file))
    } else {
        resource_string_content(&format!("{}{}", "valid/", file))
    }
}

pub fn valid_resource_path(file: &str) -> String {
    if cfg!(windows) {
        resource_path(&format!("{}{}", "valid\\", file))
    } else {
        resource_path(&format!("{}{}", "valid/", file))
    }
}

pub fn valid_resource_exists(file: &str) -> bool {
    Path::exists(file.as_ref())
}

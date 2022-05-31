use std::fmt::Display;
use std::path::PathBuf;

pub trait WithSource {
    fn with_source(self, source: &Option<String>, path: &Option<PathBuf>) -> Self;
}

pub fn an_or_a<D>(parsing: D) -> &'static str where D: Display {
    let parsing = format!("{}", parsing).to_ascii_lowercase();

    if let Some('s') = parsing.chars().last() {
        return "";
    } else if parsing.chars().next().is_none() {
        return "";
    }

    match parsing.chars().next() {
        Some(c) if ['a', 'e', 'i', 'o', 'u'].contains(&c.to_ascii_lowercase()) => "an ",
        _ => "a "
    }
}

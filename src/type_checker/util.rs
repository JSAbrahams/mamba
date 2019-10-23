use std::fmt::Display;

pub fn comma_delimited<I, D>(iterable: I) -> String
where
    I: IntoIterator<Item = D>,
    D: Display {
    let mut string = String::new();
    iterable.into_iter().for_each(|item| string.push_str(&format!("{}, ", item)));
    if string.len() > 2 {
        string.remove(string.len() - 2);
    }
    String::from(string.trim_end())
}

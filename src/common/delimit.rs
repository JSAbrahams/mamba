use std::fmt::Display;

pub fn custom_delimited<I, D>(iterable: I, delimiter: &str, prepend: &str) -> String
where
    I: IntoIterator<Item = D>,
    D: Display {
    let mut string = String::new();
    iterable.into_iter().for_each(|item| string.push_str(&format!("{}{}", item, delimiter)));
    if !string.is_empty() {
        format!("{}{}", prepend, string.trim_end_matches(delimiter))
    } else {
        String::new()
    }
}

pub fn comma_delm<I, D>(iterable: I) -> String
where
    I: IntoIterator<Item = D>,
    D: Display {
    custom_delimited(iterable, ", ", "")
}

pub fn newline_delimited<I, D>(iterable: I) -> String
where
    I: IntoIterator<Item = D>,
    D: Display {
    custom_delimited(iterable, "\n", "")
}

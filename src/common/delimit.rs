use std::fmt::Display;
use std::fmt::Write;

pub fn custom_delimited<I, D>(iterable: I, delimiter: &str, prepend: &str) -> String
where
    I: IntoIterator<Item = D> + Clone,
    D: Display,
{
    if iterable.clone().into_iter().next().is_none() {
        return String::new();
    }

    let mut s = String::from(prepend);
    iterable.into_iter().for_each(|item| write!(&mut s, "{item}{delimiter}").unwrap());
    String::from(s.trim_end_matches(delimiter))
}

pub fn comma_delm<I, D>(iterable: I) -> String
where
    I: IntoIterator<Item = D> + Clone,
    D: Display,
{
    custom_delimited(iterable, ", ", "")
}

pub fn newline_delimited<I, D>(iterable: I) -> String
where
    I: IntoIterator<Item = D> + Clone,
    D: Display,
{
    custom_delimited(iterable, "\n", "")
}

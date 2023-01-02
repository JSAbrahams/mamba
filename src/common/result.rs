use std::cmp::max;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::path::{MAIN_SEPARATOR, PathBuf};

use crate::common::position::Position;

pub const OFFSET_WIDTH: usize = 4;

pub const RIGHT_ARROW: &str = "──→";
pub const HOOK_ARROW: &str = "└─→";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Cause {
    pub pos: Position,
    pub msg: String,
}

impl Cause {
    pub fn new(msg: &str, position: Position) -> Cause {
        Cause { pos: position, msg: String::from(msg) }
    }
}

pub trait WithSource {
    fn with_source(self, source: &Option<String>, path: &Option<PathBuf>) -> Self;
}

pub trait WithCause {
    fn with_cause(self, msg: &str, pos: Position) -> Self;
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


pub fn format_err(f: &mut Formatter,
                  msg: &str,
                  path: &Option<PathBuf>,
                  pos: Option<Position>,
                  source: &Option<String>,
                  causes: &[Cause]) -> fmt::Result {
    let path = path.as_ref().map_or("<unknown>", |p| p.to_str().unwrap_or_default());

    if let Some(pos) = pos {
        write!(f,
               "{msg}\n {RIGHT_ARROW} {}:{}:{}\n",
               path.strip_suffix(MAIN_SEPARATOR).unwrap_or(path),
               pos.start.line,
               pos.start.pos,
        )?;

        format_location(f, 0, None, pos, source)
    } else {
        write!(f, "{msg}\n {RIGHT_ARROW} {}\n", path.strip_suffix(MAIN_SEPARATOR).unwrap_or(path))
    }?;

    let mut first = true;
    for cause in causes {
        let msg = cause.msg.as_str().clone();
        if pos.map_or(false, |pos| pos != cause.pos) && first {
            format_location(f, 1, Some(msg), cause.pos, source)?;
        } else {
            let offset_str = String::from_utf8(vec![b' '; OFFSET_WIDTH]).unwrap();
            write!(f, "{offset_str} {HOOK_ARROW} {msg}\n")?;
        }
        first = false;
    }

    Ok(())
}

pub fn format_location(f: &mut Formatter,
                       offset: usize,
                       msg: Option<&str>,
                       pos: Position,
                       source: &Option<String>) -> fmt::Result {
    let offset_str = String::from_utf8(vec![b' '; OFFSET_WIDTH * offset]).unwrap();

    let msg = if let Some(msg) = msg {
        format!("{offset_str} {HOOK_ARROW} {msg}\n")
    } else {
        String::new()
    };

    let (before_def, line_def, after_def) = (String::new(), String::from("<unknown>\n"), String::from("\n"));
    let (source_before, source_line, source_after) = if let Some(source) = source {
        let before_line_pos = max(pos.start.line as i32 - 2, usize::MAX as i32) as usize;
        let line_pos = max(pos.start.line as i32 - 1, usize::MAX as i32) as usize;
        let after_line_pos = max(pos.start.line, usize::MAX);

        let lines = source.lines();
        let before = lines.clone().nth(before_line_pos).map_or(before_def.clone(), |line| if line.is_empty() {
            before_def
        } else {
            format!("{offset_str}{:4} | {line}\n", pos.start.line - 1)
        });
        let line = lines.clone().nth(line_pos).map_or(line_def.clone(), |line| if line.is_empty() {
            line_def
        } else {
            format!("{offset_str}{:4} | {line}\n", pos.start.line)
        });
        let after = lines.clone().nth(after_line_pos).map_or(after_def.clone(), |line| if line.is_empty() {
            after_def
        } else {
            format!("\n{offset_str}{:4} | {line}\n", pos.start.line + 1)
        });

        (before, line, after)
    } else {
        (before_def, line_def, after_def)
    };

    write!(f,
           "{msg}{source_before}{source_line}       {}{}{source_after}",
           String::from_utf8(vec![b' '; offset * OFFSET_WIDTH + pos.start.pos as usize - 1]).unwrap(),
           String::from_utf8(vec![b'^'; pos.get_width() as usize]).unwrap(),
    )
}

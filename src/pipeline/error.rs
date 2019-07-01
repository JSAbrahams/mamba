use crate::parser::parse_result::ParseErr;
use std::path::PathBuf;

pub fn syntax_err(err: &ParseErr, source: &str, in_path: &PathBuf) -> String {
    let source_line = source.lines().nth(err.line as usize - 1);
    format!(
        "--> {}:{}:{}
     |
{:3}  | {}
     | {}{} {}\n",
        in_path.display(),
        err.line,
        err.pos,
        err.line,
        source_line.unwrap_or(""),
        String::from_utf8(vec![b' '; err.pos as usize - 1]).unwrap(),
        String::from_utf8(vec![b'^'; err.width as usize]).unwrap(),
        err.msg
    )
}

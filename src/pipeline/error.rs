use crate::parser::parse_result::ParseErr;
use std::cmp::min;
use std::path::PathBuf;

const SYNTAX_ERR_MAX_DEPTH: usize = 2;

pub fn syntax_err(err: &ParseErr, source: &str, in_path: &PathBuf) -> String {
    let cause_formatter = &err.causes[0..min(err.causes.len(), SYNTAX_ERR_MAX_DEPTH)]
        .iter()
        .rev()
        .fold(String::new(), |acc, cause| {
            acc + &format!(
                "{:3}  |- {}\n     | {}^ in {} ({}:{})\n",
                cause.line,
                source.lines().nth(cause.line as usize - 1).unwrap_or(""),
                String::from_utf8(vec![b' '; cause.pos as usize]).unwrap(),
                cause.cause,
                cause.line,
                cause.pos,
            )
        });

    format!(
        "--> {}:{}:{}
     | {}
{}
{:3}  |- {}
     | {}{}",
        in_path.display(),
        err.line,
        err.pos,
        err.msg,
        cause_formatter,
        err.line,
        source.lines().nth(err.line as usize - 1).unwrap_or(""),
        String::from_utf8(vec![b' '; err.pos as usize]).unwrap(),
        String::from_utf8(vec![b'^'; err.width as usize]).unwrap()
    )
}

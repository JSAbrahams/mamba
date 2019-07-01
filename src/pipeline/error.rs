use crate::parser::parse_result::ParseErr;
use std::cmp::min;
use std::path::PathBuf;

const SYNTAX_ERR_MAX_DEPTH: usize = 3;

pub fn syntax_err(err: &ParseErr, source: &str, in_path: &PathBuf) -> String {
    let source_line = source.lines().nth(err.line as usize - 1);

    let trimmed_causes = &err.causes[0..min(err.causes.len(), SYNTAX_ERR_MAX_DEPTH)];

    let mut offset = -2;
    let cause_formatter = trimmed_causes.iter().fold(String::new(), |acc, (cause, line, pos)| {
        let source_line = if *line > 0 { source.lines().nth(*line as usize - 1) } else { Some("") };

        offset += 2;
        acc + &format!(
            "     |{} |- {}\n     |{}^ {} ({}:{})\n",
            String::from_utf8(vec![b' '; 2 + (err.pos + offset) as usize]).unwrap(),
            source_line.unwrap_or(""),
            String::from_utf8(vec![b' '; (err.pos + pos + 5 + offset) as usize]).unwrap(),
            cause,
            line,
            pos,
        )
    });

    format!(
        "--> {}:{}:{}
     |
{:3}  | {}
     | {}{} {}\n{}",
        in_path.display(),
        err.line,
        err.pos,
        err.line,
        source_line.unwrap_or(""),
        String::from_utf8(vec![b' '; err.pos as usize - 1]).unwrap(),
        String::from_utf8(vec![b'^'; err.width as usize]).unwrap(),
        err.msg,
        cause_formatter
    )
}

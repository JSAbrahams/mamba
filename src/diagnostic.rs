//! Diagnostics as data: what went wrong, where, and in which stage of the compiler.
//!
//! The core reports its own error types, such as [ParseErr] and [TypeErr].
//! They are turned into a [Diagnostic] at the edge of the crate, so that every user of the crate sees one shape.
//! A [Diagnostic] can be [rendered](Diagnostic::render) as text for a terminal, or, with the `serde` feature, serialized.

use std::collections::BTreeSet;
use std::fmt::{self, Display, Formatter, Write};
use std::path::{Path, PathBuf};

use crate::backend::cranelift::result::BackendErr;
use crate::backend::python::result::UnimplementedErr;
use crate::check::result::TypeErr;
use crate::common::position::{CaretPos, Position};
use crate::common::result::Cause;
use crate::parse::result::ParseErr;

/// How many of the causes of a parse error become labels.
///
/// A parse error carries the whole chain of rules it was in, from the innermost outward.
/// Beyond the innermost one or two the chain says little, and the outermost is always the file itself.
const PARSE_CAUSE_LABELS: usize = 1;

/// Lines of source shown before and after the lines that carry a label.
const CONTEXT_LINES: usize = 2;

/// A span longer than this many lines is shown by its first and last lines only.
const MAX_SPAN_LINES: usize = 4;

/// Width of a tab when a line of source is shown.
const TAB_WIDTH: usize = 4;

/// The stage of the compiler that reported a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum Stage {
    /// Turning text into a syntax tree, including any error of the lexer.
    Parse,
    /// Type checking.
    Type,
    /// Generating Python, assembly or machine code.
    Generate,
    /// Reading sources or writing output, only with the `native` feature.
    Io,
}

/// How serious a diagnostic is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum Severity {
    /// Compilation failed.
    Error,
    /// Compilation went on.
    Warning,
}

/// A place in a source: a line and a column, both counted from 1.
///
/// The column counts characters (Unicode scalar values), not bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct LineCol {
    pub line: usize,
    pub column: usize,
}

/// A range in a source, from `start` up to but not including `end`.
///
/// The range is given two ways: as lines and columns, and as byte offsets.
/// The offsets count from the start of the source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Span {
    pub start: LineCol,
    pub end: LineCol,
    pub byte_start: usize,
    pub byte_end: usize,
}

/// A span of source with something to say about it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Label {
    pub span: Span,
    pub message: Option<String>,
    /// Whether this is where the error is, rather than context for it.
    pub primary: bool,
}

/// Something the compiler has to say about a source, most often why it does not compile.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Diagnostic {
    pub stage: Stage,
    pub severity: Severity,
    pub message: String,
    /// The name of the source, if it has one.
    pub source: Option<PathBuf>,
    /// Where in the source, primary label first. Empty if the compiler does not know where.
    pub labels: Vec<Label>,
    pub notes: Vec<String>,
}

/// How [Diagnostic::render] writes its text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderStyle {
    /// Text only, for files, logs and the browser console.
    #[default]
    Plain,
    /// With ANSI colour codes, for a terminal.
    Ansi,
}

impl LineCol {
    pub fn new(line: usize, column: usize) -> LineCol {
        LineCol { line, column }
    }
}

impl Span {
    /// The span from `start` up to `end` in `text`.
    ///
    /// Both are clamped to `text`, and an empty or reversed span is widened to one character where the line allows it.
    pub fn new(text: &str, start: LineCol, end: LineCol) -> Span {
        let lines = Lines::new(text);
        let (start, end) = (lines.clamp(start), lines.clamp(end));
        lines.span(lines.byte_offset(start), lines.byte_offset(end))
    }

    /// The span from byte offset `start` up to byte offset `end` in `text`, clamped and widened as by [Span::new].
    pub fn from_bytes(text: &str, start: usize, end: usize) -> Span {
        Lines::new(text).span(start, end)
    }

    /// The span of a core [Position], or [None] if the position is invisible.
    pub(crate) fn from_position(text: &str, position: Position) -> Option<Span> {
        if position.start.line == 0 {
            return None;
        }
        Some(Lines::new(text).core_span(position.start, position.end))
    }

    /// The place just after the last character of `text` that is not white space.
    fn end_of(text: &str) -> Span {
        let end = text.trim_end().len();
        Span::from_bytes(text, end, end)
    }
}

impl Label {
    pub fn primary(span: Span, message: Option<String>) -> Label {
        Label {
            span,
            message,
            primary: true,
        }
    }

    pub fn secondary(span: Span, message: Option<String>) -> Label {
        Label {
            span,
            message,
            primary: false,
        }
    }
}

impl Diagnostic {
    /// An error of `stage` with no position.
    pub fn error(stage: Stage, message: impl Into<String>) -> Diagnostic {
        Diagnostic {
            stage,
            severity: Severity::Error,
            message: message.into(),
            source: None,
            labels: vec![],
            notes: vec![],
        }
    }

    #[must_use]
    pub fn with_source(self, source: Option<PathBuf>) -> Diagnostic {
        Diagnostic { source, ..self }
    }

    #[must_use]
    pub fn with_label(mut self, label: Label) -> Diagnostic {
        self.labels.push(label);
        self
    }

    #[must_use]
    pub fn with_note(mut self, note: impl Into<String>) -> Diagnostic {
        self.notes.push(note.into());
        self
    }

    /// The primary label, if the compiler knows where the problem is.
    pub fn primary_label(&self) -> Option<&Label> {
        self.labels.iter().find(|label| label.primary)
    }

    /// A parse error in `text`, which is the source named `source`.
    pub(crate) fn from_parse(err: &ParseErr, text: &str, source: Option<&Path>) -> Diagnostic {
        let mut diagnostic =
            Diagnostic::error(Stage::Parse, err.msg.clone()).with_source(owned(source));
        diagnostic
            .labels
            .push(match Span::from_position(text, err.pos) {
                Some(span) => Label::primary(span, None),
                None => Label::primary(
                    Span::end_of(text),
                    Some(String::from("the source ends here")),
                ),
            });

        for cause in labelled_causes(err) {
            diagnostic.add_cause(cause, Some(text));
        }
        diagnostic
    }

    /// A type error in `text`, which is the source named `source`.
    ///
    /// `text` is [None] when it is not known which source the error is in.
    /// Its causes are private to the type checker, so only its own position becomes a label.
    pub(crate) fn from_type(
        err: &TypeErr,
        text: Option<&str>,
        source: Option<&Path>,
    ) -> Diagnostic {
        let mut diagnostic =
            Diagnostic::error(Stage::Type, err.msg.clone()).with_source(owned(source));
        diagnostic.add_primary(err.pos, text);
        diagnostic
    }

    /// An error of the Python backend in `text`, which is the source named `source`.
    pub(crate) fn from_unimplemented(
        err: &UnimplementedErr,
        text: &str,
        source: Option<&Path>,
    ) -> Diagnostic {
        let mut diagnostic =
            Diagnostic::error(Stage::Generate, err.msg.clone()).with_source(owned(source));
        diagnostic.add_primary(Some(err.position), Some(text));
        diagnostic
    }

    /// An error of the machine code backend in `text`, which is the source named `source`.
    pub(crate) fn from_backend(err: &BackendErr, text: &str, source: Option<&Path>) -> Diagnostic {
        let mut diagnostic =
            Diagnostic::error(Stage::Generate, err.msg.clone()).with_source(owned(source));
        diagnostic.add_primary(Some(err.position), Some(text));
        diagnostic
    }

    /// Label `position` as where the problem is, or say where it is in a note if the text is not known.
    fn add_primary(&mut self, position: Option<Position>, text: Option<&str>) {
        let Some(position) = position.filter(|position| position.start.line > 0) else {
            return;
        };
        match text.and_then(|text| Span::from_position(text, position)) {
            Some(span) => self.labels.push(Label::primary(span, None)),
            None => self.notes.push(format!("at {}", line_col(position.start))),
        }
    }

    /// Add a cause as a label, or as a note if it has no position of its own.
    fn add_cause(&mut self, cause: &Cause, text: Option<&str>) {
        let msg = cause.msg.clone();
        let Some(span) = text.and_then(|text| Span::from_position(text, cause.pos)) else {
            match cause.pos.start.line {
                0 => self.notes.push(msg),
                _ => self
                    .notes
                    .push(format!("{msg}, at {}", line_col(cause.pos.start))),
            }
            return;
        };

        match self.labels.iter_mut().find(|label| label.span == span) {
            Some(label) if label.message.is_none() => label.message = Some(msg),
            Some(_) => self.notes.push(msg),
            None => self.labels.push(Label::secondary(span, Some(msg))),
        }
    }

    /// Render as text, quoting the lines of `src` that the labels point at.
    ///
    /// `src` should be the text of the source that this diagnostic is about.
    pub fn render(&self, src: &str, style: RenderStyle) -> String {
        let paint = Paint(style);
        let lines = Lines::new(src);
        let shown = self.shown_lines(&lines);
        let gutter = shown.last().map_or(1, |line| digits(*line));
        let mut out = String::new();

        self.write_header(&mut out, paint, gutter, Some(&lines));
        self.write_source(&mut out, paint, gutter, &lines, &shown);
        self.write_notes(&mut out, paint, gutter);
        out
    }

    /// The header, location and notes, without any source.
    fn render_without_source(&self, style: RenderStyle) -> String {
        let paint = Paint(style);
        let gutter = self
            .primary_label()
            .map_or(1, |label| digits(label.span.start.line));
        let mut out = String::new();
        self.write_header(&mut out, paint, gutter, None);
        self.write_notes(&mut out, paint, gutter);
        out
    }

    fn write_header(&self, out: &mut String, paint: Paint, gutter: usize, lines: Option<&Lines>) {
        let (kind, colour) = match self.severity {
            Severity::Error => ("error", RED),
            Severity::Warning => ("warning", YELLOW),
        };
        let mut message = self.message.lines();
        let first = message.next().unwrap_or_default();
        let _ = writeln!(
            out,
            "{}{}: {}",
            paint.paint(kind, colour),
            paint.paint(&format!("[{}]", self.stage), colour),
            paint.paint(first, BOLD)
        );
        for line in message {
            let _ = writeln!(
                out,
                "{} {}",
                pad(kind.len() + self.stage.to_string().len() + 3),
                paint.paint(line, BOLD)
            );
        }

        let name = self
            .source
            .as_ref()
            .map_or(String::from("<input>"), |path| path.display().to_string());
        let location = match self.primary_label() {
            Some(label) => {
                let start = lines.map_or(label.span.start, |lines| lines.clamp(label.span.start));
                format!("{name}:{start}")
            }
            None => name,
        };
        let _ = writeln!(out, "{} {} {location}", pad(gutter), paint.gutter("──→"));
    }

    fn write_notes(&self, out: &mut String, paint: Paint, gutter: usize) {
        for note in &self.notes {
            let _ = writeln!(
                out,
                "{} {} {}",
                pad(gutter),
                paint.gutter("="),
                paint.paint("note:", BOLD) + " " + note
            );
        }
    }

    /// Write the `shown` lines of the source between two bars, marking where lines are skipped.
    fn write_source(
        &self,
        out: &mut String,
        paint: Paint,
        gutter: usize,
        lines: &Lines,
        shown: &[usize],
    ) {
        if shown.is_empty() {
            return;
        }

        let bar = paint.gutter("│");
        let _ = writeln!(out, "{} {bar}", pad(gutter));
        let mut previous: Option<usize> = None;
        for &line in shown {
            if previous.is_some_and(|previous| line > previous + 1) {
                let _ = writeln!(out, "{} {}", pad(gutter), paint.gutter("┆"));
            }
            self.write_line(out, paint, gutter, lines, line);
            previous = Some(line);
        }
        let _ = writeln!(out, "{} {bar}", pad(gutter));
    }

    /// Write line `line` of the source, followed by a row of markers for each label on it.
    fn write_line(
        &self,
        out: &mut String,
        paint: Paint,
        gutter: usize,
        lines: &Lines,
        line: usize,
    ) {
        let text = lines.line(line);
        let ranges: Vec<(&Label, usize, usize)> = self
            .labels
            .iter()
            .filter_map(|label| {
                label_on_line(lines, label, line).map(|(from, to)| (label, from, to))
            })
            .collect();

        let bar = paint.gutter("│");
        let number = paint.gutter(&format!("{line:>gutter$}"));
        let _ = writeln!(
            out,
            "{number} {bar} {}",
            highlight(text, &ranges, paint).trim_end()
        );

        let mut ranges = ranges;
        ranges.sort_by_key(|(label, from, _)| (!label.primary, *from));
        for (label, from, to) in ranges {
            let (marker, colour) = if label.primary {
                ('^', RED)
            } else {
                ('-', BLUE)
            };
            let offset = display_width(text, from);
            let width = (display_width(text, to) - offset).max(1);
            let markers: String = std::iter::repeat_n(marker, width).collect();
            let ends_here = line_range(lines, label).1 == line;
            let message = match &label.message {
                Some(message) if ends_here => format!(" {message}"),
                _ => String::new(),
            };
            let _ = writeln!(
                out,
                "{} {bar} {}{}",
                pad(gutter),
                pad(offset),
                paint.paint(&(markers + &message), colour)
            );
        }
    }

    /// The numbers of the lines to show: each labelled line, with context around it.
    fn shown_lines(&self, lines: &Lines) -> Vec<usize> {
        let mut shown = BTreeSet::new();
        for label in &self.labels {
            let (start, end) = line_range(lines, label);
            let labelled: Vec<usize> = if end - start + 1 > MAX_SPAN_LINES {
                vec![start, start + 1, end - 1, end]
            } else {
                (start..=end).collect()
            };

            let first = start.saturating_sub(CONTEXT_LINES).max(1);
            let last = (end + CONTEXT_LINES).min(lines.count());
            let before = (first..start).skip_while(|line| lines.line(*line).trim().is_empty());
            let after: Vec<usize> = (end + 1..=last).collect();
            let after_len = after
                .iter()
                .rposition(|line| !lines.line(*line).trim().is_empty())
                .map_or(0, |i| i + 1);

            shown.extend(before);
            shown.extend(labelled);
            shown.extend(after.into_iter().take(after_len));
        }
        shown.into_iter().collect()
    }
}

impl Display for Stage {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let name = match self {
            Stage::Parse => "parse",
            Stage::Type => "type",
            Stage::Generate => "generate",
            Stage::Io => "io",
        };
        write!(f, "{name}")
    }
}

impl Display for LineCol {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// The header and notes, without source: use [Diagnostic::render] to also see the code.
impl Display for Diagnostic {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.render_without_source(RenderStyle::Plain).trim_end()
        )
    }
}

impl std::error::Error for Diagnostic {}

/// A diagnostic together with its rendering, which is the shape JSON output takes.
#[cfg(feature = "serde")]
#[derive(serde::Serialize)]
pub struct RenderedDiagnostic<'a> {
    #[serde(flatten)]
    pub diagnostic: &'a Diagnostic,
    /// [Diagnostic::render] in [RenderStyle::Plain].
    pub rendered: String,
}

#[cfg(feature = "serde")]
impl<'a> RenderedDiagnostic<'a> {
    pub fn new(diagnostic: &'a Diagnostic, src: &str) -> RenderedDiagnostic<'a> {
        RenderedDiagnostic {
            diagnostic,
            rendered: diagnostic.render(src, RenderStyle::Plain),
        }
    }
}

/// The causes of `err` that become labels, innermost first, see [PARSE_CAUSE_LABELS].
///
/// The last cause is the file as a whole, which says nothing, so it is left out.
fn labelled_causes(err: &ParseErr) -> impl Iterator<Item = &Cause> {
    let inner = err.causes.len().saturating_sub(1);
    err.causes.iter().take(inner.min(PARSE_CAUSE_LABELS))
}

/// The first and last line that `label` covers.
///
/// A span that ends at the first column of a line does not reach into that line.
fn line_range(lines: &Lines, label: &Label) -> (usize, usize) {
    let (start, end) = (lines.clamp(label.span.start), lines.clamp(label.span.end));
    if end.line > start.line && end.column == 1 {
        (start.line, end.line - 1)
    } else {
        (start.line, end.line)
    }
}

/// The range of columns, `from` up to `to`, that `label` covers on line `line`.
fn label_on_line(lines: &Lines, label: &Label, line: usize) -> Option<(usize, usize)> {
    let (start, end) = (lines.clamp(label.span.start), lines.clamp(label.span.end));
    let (first, last) = line_range(lines, label);
    if line < first || line > last {
        return None;
    }
    let from = if line == start.line { start.column } else { 1 };
    let to = if line == end.line {
        end.column
    } else {
        lines.line(line).chars().count() + 1
    };
    Some((from, to.max(from + 1)))
}

/// `text` with tabs expanded, and in colour the parts that primary labels cover.
fn highlight(text: &str, ranges: &[(&Label, usize, usize)], paint: Paint) -> String {
    let mut out = String::new();
    let mut painting = false;
    for (index, c) in text.chars().enumerate() {
        let column = index + 1;
        let covered = ranges
            .iter()
            .any(|(label, from, to)| label.primary && (*from..*to).contains(&column));
        if covered != painting {
            out.push_str(if covered {
                paint.start(RED)
            } else {
                paint.reset()
            });
            painting = covered;
        }
        if c == '\t' {
            out.push_str(&pad(TAB_WIDTH));
        } else {
            out.push(c);
        }
    }
    if painting {
        out.push_str(paint.reset());
    }
    out
}

/// The width on screen of the characters of `text` before `column`, with tabs expanded.
fn display_width(text: &str, column: usize) -> usize {
    let before = text.chars().take(column.saturating_sub(1));
    let width: usize = before
        .clone()
        .map(|c| if c == '\t' { TAB_WIDTH } else { 1 })
        .sum();
    width + column.saturating_sub(1).saturating_sub(before.count())
}

fn owned(source: Option<&Path>) -> Option<PathBuf> {
    source.map(Path::to_path_buf)
}

fn line_col(caret: CaretPos) -> LineCol {
    LineCol::new(caret.line, caret.pos)
}

fn pad(width: usize) -> String {
    " ".repeat(width)
}

fn digits(number: usize) -> usize {
    number.max(1).ilog10() as usize + 1
}

/// The lines of a source, split on `\n` with any trailing `\r` dropped.
///
/// Made once per source, it turns many offsets into spans without reading the text from the start each time.
struct Lines<'a> {
    text: &'a str,
    /// The byte offset at which each line starts.
    starts: Vec<usize>,
}

impl<'a> Lines<'a> {
    fn new(text: &'a str) -> Lines<'a> {
        let starts = std::iter::once(0)
            .chain(text.match_indices('\n').map(|(index, _)| index + 1))
            .collect();
        Lines { text, starts }
    }

    fn count(&self) -> usize {
        self.starts.len()
    }

    /// Line `line`, counted from 1, without its line ending.
    fn line(&self, line: usize) -> &'a str {
        let start = self.starts[line - 1];
        let end = self
            .starts
            .get(line)
            .map_or(self.text.len(), |next| next - 1);
        let line = &self.text[start..end];
        line.strip_suffix('\r').unwrap_or(line)
    }

    /// The nearest place that exists in the text.
    fn clamp(&self, place: LineCol) -> LineCol {
        let line = place.line.clamp(1, self.count());
        let columns = self.line(line).chars().count() + 1;
        LineCol::new(line, place.column.clamp(1, columns))
    }

    /// The byte offset of a place, which must exist.
    fn byte_offset(&self, place: LineCol) -> usize {
        let line = self.line(place.line);
        let within = line
            .char_indices()
            .nth(place.column - 1)
            .map_or(line.len(), |(index, _)| index);
        self.starts[place.line - 1] + within
    }

    /// The byte offset of a place as the core gives it, whose column counts bytes from 1.
    fn core_offset(&self, caret: CaretPos) -> usize {
        let line = caret.line.clamp(1, self.count());
        let within = caret.pos.saturating_sub(1).min(self.line(line).len());
        self.starts[line - 1] + within
    }

    /// The span between two places as the core gives them, see [Lines::core_offset].
    fn core_span(&self, start: CaretPos, end: CaretPos) -> Span {
        self.span(self.core_offset(start), self.core_offset(end))
    }

    /// The span between two byte offsets, clamped to the text and its character boundaries.
    ///
    /// An empty or reversed span is widened to the next character, unless that ends the line.
    fn span(&self, start: usize, end: usize) -> Span {
        let text = self.text;
        let mut start = start.min(text.len());
        while !text.is_char_boundary(start) {
            start -= 1;
        }
        let mut end = end.clamp(start, text.len());
        while !text.is_char_boundary(end) {
            end += 1;
        }
        if end == start {
            let next = text[start..].chars().next();
            if let Some(c) = next.filter(|c| *c != '\n' && *c != '\r') {
                end += c.len_utf8();
            }
        }

        Span {
            start: self.place(start),
            end: self.place(end),
            byte_start: start,
            byte_end: end,
        }
    }

    /// The line and column of a byte offset at a character boundary.
    fn place(&self, offset: usize) -> LineCol {
        let line = self.starts.partition_point(|start| *start <= offset);
        let within = &self.text[self.starts[line - 1]..offset];
        LineCol::new(line, within.chars().count() + 1)
    }
}

const BOLD: &str = "\x1b[1m";
const RED: &str = "\x1b[1;31m";
const YELLOW: &str = "\x1b[1;33m";
const BLUE: &str = "\x1b[1;34m";
const RESET: &str = "\x1b[0m";

/// Colour, if the style asks for it.
#[derive(Clone, Copy)]
struct Paint(RenderStyle);

impl Paint {
    fn start(self, colour: &'static str) -> &'static str {
        match self.0 {
            RenderStyle::Plain => "",
            RenderStyle::Ansi => colour,
        }
    }

    fn reset(self) -> &'static str {
        self.start(RESET)
    }

    fn paint(self, text: &str, colour: &'static str) -> String {
        format!("{}{text}{}", self.start(colour), self.reset())
    }

    fn gutter(self, text: &str) -> String {
        self.paint(text, BLUE)
    }
}

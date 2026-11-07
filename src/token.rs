//! The tokens of a source, for syntax highlighting.

use crate::diagnostic::{Diagnostic, Lines, Span};
use crate::parse::lex::token::Token as LexToken;
use crate::parse::lex::tokenize;

/// What sort of token a [Token] is, which is what a highlighter colours by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum TokenKind {
    /// A reserved word, such as `def`, `let` or `if`.
    Keyword,
    /// A name, such as `x` or `Int`.
    Identifier,
    /// An integer, real or E-notation literal.
    Number,
    /// A string literal, including its quotes.
    String,
    /// A doc string.
    DocString,
    /// `True`, `False`, `None` and the like are identifiers to the lexer, so this is only `_`.
    Underscore,
    /// An operator, such as `+`, `:=` or `and`.
    Operator,
    /// Brackets, commas, colons, points and arrows.
    Punctuation,
    /// A comment, from `#` to the end of the line.
    ///
    /// The lexer drops comments, so these are found in the text between the other tokens.
    Comment,
}

/// One token of a source.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Token {
    pub kind: TokenKind,
    /// The name of the token, such as `def`, `assign` or `identifier`.
    pub name: String,
    pub span: Span,
}

/// Split `text` into tokens, in the order they appear.
///
/// Newlines are left out, and comments are added, so that every token is something a highlighter can colour.
pub fn tokens(text: &str) -> Result<Vec<Token>, Vec<Diagnostic>> {
    let lexed = tokenize(text).map_err(|err| vec![Diagnostic::from_lex(&err, text, None)])?;
    let lines = Lines::new(text);

    let mut tokens: Vec<Token> = lexed
        .iter()
        .filter_map(|lex| {
            let (kind, name) = classify(&lex.token)?;
            let start = lines.core_offset(lex.pos.start);
            let span = match &lex.token {
                // The lexer does not track where a string that spans lines ends, so find its closing quote.
                LexToken::Str(..) => lines.span(start, string_end(text, start, false)),
                LexToken::DocStr(..) => lines.span(start, string_end(text, start, true)),
                _ => lines.core_span(lex.pos.start, lex.pos.end),
            };
            Some(Token {
                kind,
                name: String::from(name),
                span,
            })
        })
        .collect();

    tokens.extend(comments(text, &lines, &tokens));
    tokens.sort_by_key(|token| token.span.byte_start);
    Ok(tokens)
}

/// The kind and name of a token, or [None] for a newline, which there is nothing to colour of.
fn classify(token: &LexToken) -> Option<(TokenKind, &'static str)> {
    use LexToken::*;
    use TokenKind::{Keyword as K, Operator as O, Punctuation as P};

    Some(match token {
        NL => return None,

        Id(_) => (TokenKind::Identifier, "identifier"),
        Real(_) => (TokenKind::Number, "real"),
        Int(_) => (TokenKind::Number, "int"),
        ENum(..) => (TokenKind::Number, "eNum"),
        Str(..) => (TokenKind::String, "string"),
        DocStr(_) => (TokenKind::DocString, "docString"),
        Underscore => (TokenKind::Underscore, "underscore"),

        From => (K, "from"),
        Trait => (K, "trait"),
        Class => (K, "class"),
        Pure => (K, "pure"),
        Total => (K, "total"),
        Meta => (K, "meta"),
        As => (K, "as"),
        Import => (K, "import"),
        Forward => (K, "forward"),
        Mut => (K, "mut"),
        Def => (K, "def"),
        Let => (K, "let"),
        While => (K, "while"),
        For => (K, "for"),
        In => (K, "in"),
        If => (K, "if"),
        Then => (K, "then"),
        Match => (K, "match"),
        Else => (K, "else"),
        Do => (K, "do"),
        Continue => (K, "continue"),
        Break => (K, "break"),
        Ret => (K, "return"),
        Using => (K, "using"),
        When => (K, "when"),
        Pass => (K, "pass"),
        Where => (K, "where"),
        End => (K, "end"),

        Assign => (O, "assign"),
        AddAssign => (O, "addAssign"),
        SubAssign => (O, "subAssign"),
        MulAssign => (O, "mulAssign"),
        DivAssign => (O, "divAssign"),
        PowAssign => (O, "powAssign"),
        Range => (O, "range"),
        RangeIncl => (O, "rangeIncl"),
        Add => (O, "add"),
        Sub => (O, "sub"),
        Mul => (O, "mul"),
        Div => (O, "div"),
        FDiv => (O, "fDiv"),
        Pow => (O, "pow"),
        Mod => (O, "mod"),
        Ge => (O, "ge"),
        Geq => (O, "geq"),
        Le => (O, "le"),
        Leq => (O, "leq"),
        Eq => (O, "eq"),
        Neq => (O, "neq"),
        And => (O, "and"),
        Or => (O, "or"),
        Not => (O, "not"),
        Raise => (O, "raise"),
        Question => (O, "question"),

        Point => (P, "point"),
        Comma => (P, "comma"),
        DoublePoint => (P, "colon"),
        BSlash => (P, "backslash"),
        LRBrack => (P, "lRBrack"),
        RRBrack => (P, "rRBrack"),
        LSBrack => (P, "lSBrack"),
        RSBrack => (P, "rSBrack"),
        LCBrack => (P, "lCBrack"),
        RCBrack => (P, "rCBrack"),
        Ver => (P, "ver"),
        To => (P, "to"),
        BTo => (P, "bTo"),
    })
}

/// The byte offset just after the closing quote of the string that opens at byte offset `start`, or the end of the text.
///
/// A doc string opens and closes with three quotes.
/// As in the lexer, a quote inside an interpolation, between `{` and `}`, does not close the string.
fn string_end(text: &str, start: usize, doc: bool) -> usize {
    let rest = &text[start..];
    let length = if doc {
        rest.get(3..)
            .and_then(|inner| inner.find("\"\"\""))
            .map(|end| end + 6)
    } else {
        let (mut escaped, mut depth) = (false, 0_usize);
        rest.char_indices().skip(1).find_map(|(offset, c)| {
            let closes = c == '"' && !escaped && depth == 0;
            match c {
                '{' if !escaped => depth += 1,
                '}' if !escaped => depth = depth.saturating_sub(1),
                _ => {}
            }
            escaped = c == '\\' && !escaped;
            closes.then_some(offset + 1)
        })
    };
    start + length.unwrap_or(rest.len())
}

/// The comments of `text`: a `#` that no token covers, up to the end of its line.
fn comments(text: &str, lines: &Lines, tokens: &[Token]) -> Vec<Token> {
    let covered = |offset: usize| {
        tokens
            .iter()
            .any(|token| (token.span.byte_start..token.span.byte_end).contains(&offset))
    };

    let mut comments = vec![];
    let mut line_start = 0;
    for line in text.split('\n') {
        let hash = line
            .char_indices()
            .find(|(offset, c)| *c == '#' && !covered(line_start + offset));
        if let Some((offset, _)) = hash {
            let end = line_start + line.trim_end_matches('\r').len();
            comments.push(Token {
                kind: TokenKind::Comment,
                name: String::from("comment"),
                span: lines.span(line_start + offset, end),
            });
        }
        line_start += line.len() + 1;
    }
    comments
}

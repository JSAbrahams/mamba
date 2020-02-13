use crate::common::position::CaretPos;
use crate::lex::token::Lex;
use crate::lex::token::Token;

#[derive(Clone, Debug)]
pub struct State {
    newlines:        Vec<Lex>,
    cur_indent:      i32,
    line_indent:     i32,
    token_this_line: bool,
    pub pos:         CaretPos
}

impl State {
    pub fn new() -> State {
        let pos = CaretPos::new(1, 1);
        State { newlines: vec![], cur_indent: 1, line_indent: 1, token_this_line: false, pos }
    }

    pub fn flush_indents(&mut self) -> Vec<Lex> {
        let amount = ((self.cur_indent) / 4) as usize;
        self.cur_indent = 1;
        vec![Lex::new(&self.pos, Token::Dedent); amount]
    }

    /// Change state depending on given [Token](lexer::token::Token) and return
    /// [TokenPos](lexer::token::TokenPos) with current line and position
    /// (1-indexed).
    ///
    /// Newline tokens are not immediately returned. Instead, they are returned
    /// in a batch once a non-newline token is encountered.
    /// This allows us to ensure that if we have multiple newlines followed by a
    /// dedent, that the remaining newlines are placed after the dedent.
    /// Therefore, dedents are placed as early as possible.
    pub fn token(&mut self, token: Token) -> Vec<Lex> {
        if token == Token::NL {
            self.newline();
            return vec![];
        }

        self.token_this_line = true;
        let mut res = self.newlines.pop().map_or(vec![], |nl| vec![nl]);
        res.append(&mut if self.line_indent >= self.cur_indent {
            let amount = ((self.line_indent - self.cur_indent) / 4) as usize;
            vec![Lex::new(&self.pos, Token::Indent); amount]
        } else {
            let amount = ((self.cur_indent - self.line_indent) / 4) as usize;
            vec![Lex::new(&self.pos, Token::Dedent); amount]
        });

        res.append(&mut self.newlines);
        res.push(Lex::new(&self.pos, token.clone()));

        // TODO streamline application logic for multiline strings
        self.cur_indent = self.line_indent;
        self.pos = self.pos.clone().offset_pos(token.clone().width());
        if let Token::Str(_str, _) = &token {
            self.pos = self.pos.clone().offset_line(_str.lines().count().clone() as i32 - 1);
        } else if let Token::DocStr(_str) = &token {
            self.pos = self.pos.clone().offset_line(_str.lines().count().clone() as i32 - 1);
        }

        res
    }

    fn newline(&mut self) {
        self.newlines.push(Lex::new(&self.pos, Token::NL));
        self.token_this_line = false;
        self.line_indent = 1;
        self.pos = self.pos.clone().newline();
    }

    pub fn space(&mut self) {
        self.pos = self.pos.clone().offset_pos(1);
        self.line_indent += if self.token_this_line { 0 } else { 1 };
    }
}

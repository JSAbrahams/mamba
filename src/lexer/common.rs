use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;

#[derive(Clone)]
pub struct State {
    newlines: Vec<TokenPos>,
    current_indent: i32,
    line_indent: i32,
    hit_token_this_line: bool,
    pub line: i32,
    pub pos: i32
}

impl State {
    pub fn new() -> State {
        State {
            newlines: Vec::new(),
            current_indent: 1,
            line_indent: 1,
            hit_token_this_line: false,
            line: 1,
            pos: 1
        }
    }

    pub fn flush_indents(&mut self) -> Vec<TokenPos> {
        let dedents = vec![
            TokenPos::new(self.line, self.pos, Token::Dedent);
            ((self.current_indent) / 4) as usize
        ];

        self.current_indent = 1;
        dedents
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
    pub fn token(&mut self, token: Token) -> Vec<TokenPos> {
        debug_assert_ne!(token, Token::Indent);
        debug_assert_ne!(token, Token::Dedent);
        if token == Token::NL {
            return self.newline();
        }

        self.hit_token_this_line = true;
        let mut res = match self.newlines.pop() {
            Some(nl_token_pos) => vec![nl_token_pos],
            None => vec![]
        };

        res.append(&mut if self.line_indent >= self.current_indent {
            vec![
                TokenPos::new(self.line, self.pos, Token::Indent);
                ((self.line_indent - self.current_indent) / 4) as usize
            ]
        } else {
            vec![
                TokenPos::new(self.line, self.pos, Token::Dedent);
                ((self.current_indent - self.line_indent) / 4) as usize
            ]
        });

        while let Some(nl_token_pos) = self.newlines.pop() {
            debug_assert_eq!(nl_token_pos.token, Token::NL);
            res.push(nl_token_pos);
        }

        res.push(TokenPos::new(self.line, self.pos, token.clone()));

        self.current_indent = self.line_indent;
        self.pos += match token {
            Token::Id(id) => id.len(),
            Token::Real(real) => real.len(),
            Token::Int(int) => int.len(),
            Token::Bool(true) => 4,
            Token::Bool(false) => 5,
            Token::Str(_str) => _str.len() + 2,
            Token::ENum(num, exp) => num.len() + exp.len() + 1,
            other => format!("{}", other).len()
        } as i32;

        res
    }

    fn newline(&mut self) -> Vec<TokenPos> {
        self.newlines.push(TokenPos::new(self.line, self.pos, Token::NL));
        self.hit_token_this_line = false;
        self.line_indent = 1;
        self.pos = 1;
        self.line += 1;
        return vec![];
    }

    pub fn space(&mut self) {
        self.pos += 1;
        if !self.hit_token_this_line {
            self.line_indent += 1;
        }
    }
}

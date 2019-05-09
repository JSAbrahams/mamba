use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;

#[derive(Clone)]
pub struct State {
    current_indent: i32,
    line_indent: i32,
    hit_token_this_line: bool,
    pub line: i32,
    pub pos: i32
}

impl State {
    pub fn new() -> State {
        State { current_indent: 1, line_indent: 1, hit_token_this_line: false, line: 1, pos: 1 }
    }

    pub fn token(&mut self, token: Token) -> Vec<TokenPos> {
        debug_assert_ne!(token, Token::Indent);
        debug_assert_ne!(token, Token::Dedent);

        if token == Token::NL {
            self.hit_token_this_line = false;
            self.line_indent = 1;
            self.pos = 1;
            self.line += 1;
            return vec![TokenPos { line: self.line, pos: self.pos, token: token.clone() }];
        }

        self.hit_token_this_line = true;
        self.pos += match token.clone() {
            Token::Id(id) => id.len(),
            Token::Real(real) => real.len(),
            Token::Int(int) => int.len(),
            Token::Bool(true) => 4,
            Token::Bool(false) => 5,
            Token::Str(_str) => _str.len() + 2,
            Token::ENum(num, exp) => num.len() + exp.len() + 1,
            other => format!("{}", other).len()
        } as i32;

        let mut res = if self.line_indent >= self.current_indent {
            vec![
                TokenPos { line: self.line, pos: self.pos, token: Token::Indent };
                ((self.line_indent - self.current_indent) / 4) as usize
            ]
        } else {
            vec![
                TokenPos { line: self.line, pos: self.pos, token: Token::Dedent };
                ((self.current_indent - self.line_indent) / 4) as usize
            ]
        };

        self.current_indent = self.line_indent;
        res.push(TokenPos { line: self.line, pos: self.pos, token });
        res
    }

    pub fn space(&mut self) {
        self.pos += 1;
        if !self.hit_token_this_line {
            self.line_indent += 1;
        }
    }
}

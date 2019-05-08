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
        State { current_indent: 0, line_indent: 0, hit_token_this_line: false, line: 0, pos: 0 }
    }

    pub fn token(&mut self, token: &Token) -> Vec<TokenPos> {
        debug_assert_ne!(*token, Token::Indent);
        debug_assert_ne!(*token, Token::Dedent);

        if *token == Token::NL {
            self.pos = 1;
            self.line += 1;
            return vec![];
        }

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

        self.hit_token_this_line = true;
        if self.line_indent >= self.current_indent {
            vec![
                TokenPos { line: self.line, pos: self.pos, token: Token::Indent };
                ((self.line_indent - self.current_indent) % 4) as usize
            ]
        } else {
            vec![
                TokenPos { line: self.line, pos: self.pos, token: Token::Dedent };
                ((self.current_indent - self.line_indent) % 4) as usize
            ]
        }
    }

    pub fn space(&mut self) {
        self.pos += 1;
        if !self.hit_token_this_line {
            self.line_indent += 1;
        }
    }
}

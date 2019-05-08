use crate::lexer::token::Token;

pub struct State {
    current_indent: i32,
    line_indent:    i32,

    last_nl: bool,

    pub line: i32,
    pub pos:  i32
}

impl State {
    pub fn new() -> State {
        State {
            current_indent: 0,
            line_indent:    0,

            last_nl: false,

            line: 0,
            pos:  0
        }
    }

    pub fn token(mut self, token: Token) {}

    pub fn space(mut self) {}
}

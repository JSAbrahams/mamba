use crate::lexer::pass::Pass;
use crate::lexer::token::{Lex, Token};

pub struct DocString {
    front:  Option<Lex>,
    middle: Option<Lex>,
    back:   Option<Lex>
}

impl DocString {
    pub fn new() -> DocString { DocString { front: None, middle: None, back: None } }

    fn add(&mut self, lex: &Lex) {
        self.front = self.middle.clone();
        self.middle = self.back.clone();
        self.back = Some(lex.clone());
    }

    fn get(&mut self) -> Vec<Lex> {
        match (self.front.clone(), self.middle.clone(), self.back.clone()) {
            (Some(front), Some(middle), Some(back)) =>
                match (front.token, middle.token, back.token) {
                    (Token::Str(f_str, _), Token::Str(doc_str, _), Token::Str(b_str, _)) =>
                        if f_str.is_empty()
                            && b_str.is_empty()
                            && front.pos.end.pos == middle.pos.start.pos
                            && middle.pos.end.pos == back.pos.start.pos
                        {
                            self.front = None;
                            self.middle = None;
                            self.back = None;
                            return vec![Lex::new(&front.pos.start, Token::DocStr(doc_str))];
                        },
                    _ => {}
                },
            _ => {}
        };

        if let Some(lex) = &self.front.clone() {
            self.front = None;
            vec![lex.clone()]
        } else {
            vec![]
        }
    }

    fn flush(&mut self) -> Vec<Lex> {
        let mut remaining = vec![];
        if let Some(lex) = &self.front {
            remaining.push(lex.clone());
        }
        if let Some(lex) = &self.middle {
            remaining.push(lex.clone());
        }
        if let Some(lex) = &self.back {
            remaining.push(lex.clone());
        }
        remaining
    }
}

impl Pass for DocString {
    fn modify(&mut self, input: &[Lex]) -> Vec<Lex> {
        let mut out = vec![];
        for lex in input {
            self.add(lex);
            out.append(&mut self.get())
        }

        out.append(&mut self.flush());
        out
    }
}

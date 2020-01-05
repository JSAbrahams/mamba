use crate::lexer::pass::docstring::DocString;
use crate::lexer::token::Lex;

mod docstring;

pub fn pass(input: &[Lex]) -> Vec<Lex> {
    let passes: Vec<Box<dyn Pass>> = vec![Box::from(DocString::new())];

    let mut input = input.to_vec();
    for mut pass in passes {
        input = pass.modify(&input);
    }
    input
}

trait Pass {
    fn modify(&mut self, input: &[Lex]) -> Vec<Lex>;
}

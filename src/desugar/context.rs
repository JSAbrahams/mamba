pub struct State {
    pub tup:         usize,
    pub expect_expr: bool
}

impl State {
    pub fn new() -> State { State { tup: 1, expect_expr: false } }
}

pub struct Context {}

impl Context {
    pub fn new() -> Context { Context {} }
}

macro_rules! to_pos_unboxed {
    ($node:expr) => {{
        AST { pos: Position::default(), node: $node }
    }};
}

macro_rules! to_pos {
    ($node:expr) => {{
        Box::from(to_pos_unboxed!($node))
    }};
}

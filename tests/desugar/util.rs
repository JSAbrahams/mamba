macro_rules! to_pos_unboxed {
    ($node:expr) => {{
        AST {
            pos:  Position {
                start: EndPoint { line: 0, pos: 0 },
                end:   EndPoint { line: 0, pos: 0 }
            },
            node: $node
        }
    }};
}

macro_rules! to_pos {
    ($node:expr) => {{
        Box::from(to_pos_unboxed!($node))
    }};
}

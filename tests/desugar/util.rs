macro_rules! to_pos_unboxed {
    ($node:expr) => {{
        ASTNodePos { st_line: 0, st_pos: 0, en_line: 0, en_pos: 0, node: $node }
    }};
}

macro_rules! to_pos {
    ($node:expr) => {{
        Box::from(to_pos_unboxed!($node))
    }};
}

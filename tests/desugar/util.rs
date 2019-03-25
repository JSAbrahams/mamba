macro_rules! to_pos {
    ($node:expr) => {{
        Box::from(ASTNodePos { st_line: 0, st_pos: 0, en_line: 0, en_pos: 0, node: $node })
    }};
}

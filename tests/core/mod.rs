macro_rules! to_py {
    ($source:expr) => {{
        let tokens = tokenize(&$source).unwrap();
        let ast_nodes = parse(&tokens).unwrap();
        let desugared = desugar(&ast_nodes).unwrap();
        to_source(&desugared)
    }};
}

pub mod collection;
pub mod control_flow;
pub mod function;

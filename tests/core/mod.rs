macro_rules! to_py {
    ($source:expr) => {{
        let tokens = tokenize(&$source).unwrap();
        let ast_nodes = parse(&tokens).unwrap();
        let desugared = desugar(&ast_nodes);
        to_py_source(&desugared)
    }};
}

pub mod class;
pub mod collection;
pub mod compound;
pub mod control_flow;
pub mod function;

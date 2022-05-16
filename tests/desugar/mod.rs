macro_rules! to_py {
    ($source:expr) => {{
        let tokens = tokenize(&$source).unwrap();
        let ast_nodes = parse(&tokens).unwrap();
        let core = gen(&ast_nodes).unwrap();
        core.to_source()
    }};
}

pub mod collection;
pub mod control_flow;
pub mod function;

macro_rules! to_py {
    ($source:expr) => {{
        let ast = parse(&$source).unwrap();
        let core = gen(&ast).unwrap();
        core.to_source()
    }};
}

pub mod collection;
pub mod control_flow;
pub mod function;

macro_rules! to_py {
    ($source:expr) => {{
        let ast = parse(&$source).unwrap();
        let checked = check_all(&[(*ast, None, None)]).expect("Type checker should pass");
        let (ast, _, _) = checked.first().expect("Input as lost by checker");
        let core = gen(&ast).unwrap();
        core.to_source();
    }};
}

pub mod collection;
pub mod control_flow;
pub mod function;

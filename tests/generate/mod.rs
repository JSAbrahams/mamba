macro_rules! to_py {
    ($source:expr) => {{
        let ast = parse(&$source).unwrap();
        let checked = check_all(&[*ast]).expect("Type checker should pass");
        let ast_ty = checked.first().expect("Input as lost by checker");
        let core = gen(&ast_ty).unwrap();
        format!("{core}")
    }};
}

pub mod collection;
pub mod control_flow;
pub mod function;

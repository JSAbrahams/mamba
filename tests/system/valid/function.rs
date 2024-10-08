use crate::system::{test_directory, OutTestRet};

#[test]
fn callable_fun_arg() -> OutTestRet {
    test_directory(
        true,
        &["function"],
        &["function", "target"],
        "callable_fun_arg",
    )
}

#[test]
fn call_ast_verify() -> OutTestRet {
    test_directory(true, &["function"], &["function", "target"], "calls")
}

#[test]
fn definition_ast_verify() -> OutTestRet {
    test_directory(true, &["function"], &["function", "target"], "definition")
}

#[test]
fn function_with_defaults_ast_verify() -> OutTestRet {
    test_directory(
        true,
        &["function"],
        &["function", "target"],
        "function_with_defaults",
    )
}

#[test]
fn function_raise_super() -> OutTestRet {
    test_directory(
        true,
        &["function"],
        &["function", "target"],
        "function_raise_super",
    )
}

#[test]
fn match_function() -> OutTestRet {
    test_directory(
        true,
        &["function"],
        &["function", "target"],
        "match_function",
    )
}

#[test]
fn print_string() -> OutTestRet {
    test_directory(true, &["function"], &["function", "target"], "print_string")
}

#[test]
fn return_last_expression() -> OutTestRet {
    test_directory(
        true,
        &["function"],
        &["function", "target"],
        "return_last_expression",
    )
}

#[test]
fn ternary_function_call() -> OutTestRet {
    test_directory(
        true,
        &["function"],
        &["function", "target"],
        "ternary_function_call",
    )
}

use mamba::lexer::tokenize;
use mamba::parser::parse;
use mamba::type_checker::stage_1::Context;
use mamba::type_checker::type_node::Ty;

use crate::common::resource_content;

#[test]
fn interface_and_type_alias() -> Result<(), Vec<String>> {
    let source = resource_content(true, &["class"], "parent.mamba");
    let ast_node = *parse(&tokenize(&source).unwrap()).unwrap();

    let context = Context::new(&[ast_node])?;
    assert!(context.fields.is_empty());
    assert_eq!(context.classes.len(), 1);
    assert_eq!(context.interfaces.len(), 1);

    Ok(())
}

#[test]
fn interface() -> Result<(), Vec<String>> {
    let source = resource_content(true, &["class"], "parent.mamba");
    let ast_node = *parse(&tokenize(&source).unwrap()).unwrap();

    let context = Context::new(&[ast_node])?;
    assert!(context.fields.is_empty());
    assert_eq!(context.classes.len(), 1);
    assert_eq!(context.interfaces.len(), 1);
    let interface = &context.interfaces[0];
    assert_eq!(interface.functions.len(), 2);

    // def fun_a(self): () -> ()
    let function_1 = &interface.functions[0];
    assert_eq!(function_1.id, "fun_a");
    assert_eq!(function_1.ret.ty, Ty::Any);
    assert_eq!(function_1.args.len(), 1);
    let arg_1 = &function_1.args[0];
    assert_eq!(arg_1.id, "self");
    assert_eq!(arg_1.ty.clone().unwrap().ty, Ty::Custom { lit: String::from("MyType") });

    // def factorial(self,x: String): Int -> Int
    let function_2 = &interface.functions[1];
    assert_eq!(function_2.id, "factorial");
    let (out_args, out_out) = match &function_2.ret.ty {
        Ty::AnonFun { args, out } => (args, out),
        other => panic!("Expected anonymous function: {:?}", other)
    };
    assert_eq!(out_args.len(), 1);
    assert_eq!(out_args[0].ty, Ty::Int);
    assert_eq!(out_out.ty, Ty::Int);

    assert_eq!(function_2.args.len(), 2);
    let arg_1 = &function_2.args[0];
    assert_eq!(arg_1.id, "self");
    assert_eq!(arg_1.ty.clone().unwrap().ty, Ty::Custom { lit: String::from("MyType") });
    let arg_2 = &function_2.args[1];
    assert_eq!(arg_2.id, "x");
    assert_eq!(arg_2.ty.clone().unwrap().ty, Ty::String);

    Ok(())
}

use mamba::lexer::tokenize;
use mamba::parser::parse;
use mamba::type_checker::stage_1::Context;
use mamba::type_checker::type_node::Ty;

use crate::common::resource_content;

#[test]
fn context_definition_all_functions() -> Result<(), Vec<String>> {
    let source = resource_content(true, &["function"], "definition.mamba");
    let ast_node = *parse(&tokenize(&source).unwrap()).unwrap();

    let context = Context::new(&[ast_node])?;
    assert!(context.fields.is_empty());
    assert_eq!(context.classes.len(), 1);
    assert_eq!(context.classes[0].functions.len(), 10);

    assert!(context.interfaces.is_empty());
    assert_eq!(context.functions.len(), 8);

    Ok(())
}

#[test]
fn context_definition_higher_level_function() -> Result<(), Vec<String>> {
    let source = resource_content(true, &["function"], "definition.mamba");
    let ast_node = *parse(&tokenize(&source).unwrap()).unwrap();

    let context = Context::new(&[ast_node])?;
    let function = &context.functions[3];

    assert_eq!(function.id, String::from("fun_d"));

    assert_eq!(function.clone().args.len(), 1);
    let (args, out) = match function.args[0].ty.clone().unwrap().ty {
        Ty::AnonFun { args, out } => (args, out),
        other => panic!("Expected anonymous function: {:?}", other)
    };
    assert_eq!(args.len(), 2);
    assert_eq!(args[0].ty, Ty::Any);
    assert_eq!(args[1].ty, Ty::Any);
    assert_eq!(out.ty, Ty::Any);

    assert!(!function.private);
    assert!(!function.pure);
    assert!(function.raises.is_empty());
    assert_eq!(function.ret.ty, Ty::Any);

    Ok(())
}

#[test]
fn context_definition_override_addition() -> Result<(), Vec<String>> {
    let source = resource_content(true, &["function"], "definition.mamba");
    let ast_node = *parse(&tokenize(&source).unwrap()).unwrap();

    let context = Context::new(&[ast_node])?;
    let function = &context.classes[0].functions[0];

    assert_eq!(function.id, String::from("__add__"));
    assert_eq!(function.args.len(), 2);
    assert_eq!(function.args[0].ty.clone().unwrap().ty, Ty::Any);
    assert_eq!(function.args[1].ty.clone().unwrap().ty, Ty::Int);
    assert!(!function.private);
    assert!(!function.pure);
    assert!(function.raises.is_empty());
    assert_eq!(function.ret.ty, Ty::Any);

    Ok(())
}

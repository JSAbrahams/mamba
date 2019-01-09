use crate::lexer::Token;
use crate::parser::ASTNode;
use crate::parser::do_block::parse_do_block;
use crate::parser::function::parse_function_definition_body;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

pub fn parse_module(it: &mut Peekable<Iter<Token>>) -> Result<ASTNode, String> {
    match (parse_multiple(&Token::From, &parse_module_import, it),
           parse_class_name(it),
           parse_multiple(&Token::Fun, &parse_function_definition_body, it),
           parse_program_do(it)) {
        (Ok(_), Some(Ok(_)), Ok(_), Some(Ok(_))) => Err("Class cannot have a body.".to_string()),
        (Ok(imports), Some(Ok(class)), Ok(functions), None) =>
            Ok(ASTNode::ModClass(imports, wrap!(ASTNode::Id(class.to_string())), functions)),

        (Ok(imports), None, Ok(functions), Some(Ok(do_block))) =>
            Ok(ASTNode::ModProgram(imports, functions, wrap!(do_block))),
        (Ok(imports), None, Ok(functions), None) =>
            Ok(ASTNode::ModProgram(imports, functions, wrap!(ASTNode::Do(Vec::new())))),

        (Err(e), _, _, _) | (_, Some(Err(e)), _, _) | (_, _, Err(e), _) | (_, _, _, Some(Err(e))) =>
            Err(e)
    }
}

fn parse_class_name(it: &mut Peekable<Iter<Token>>) -> Option<Result<String, String>> {
    if let Some(&&Token::Class) = it.peek() {
        it.next();
        match (it.next(), it.next()) {
            (Some(Token::Id(name)), Some(Token::NL)) => Some(Ok(name.to_string())),
            (_, Some(Token::NL)) => Some(Err("Expected newline.".to_string())),
            (_, _) => Some(Err("Expected identifier".to_string()))
        }
    } else {
        None
    }
}

fn parse_multiple(token: &Token,
                  fun: &Fn(&mut Peekable<Iter<Token>>, i32) -> (Result<ASTNode, String>, i32),
                  it: &mut Peekable<Iter<Token>>) -> Result<Vec<ASTNode>, String> {
    skip_newlines(it);
    let mut elements = Vec::new();

    while let Some(&t) = it.peek() {
        if token != t { break; }
        match fun(it, 0) {
            (Ok(element), _) => elements.push(element),
            (Err(err), _) => return Err(err)
        }

        if it.peek().is_some() && it.next() != Some(&Token::NL) { break; }
        skip_newlines(it);
    }

    return Ok(elements);
}

fn skip_newlines(it: &mut Peekable<Iter<Token>>) {
    while let Some(&t) = it.peek() {
        match t {
            Token::NL => it.next(),
            _ => break
        };
    }
}

fn parse_module_import(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    if it.next() != Some(&Token::From) { return (Err("Expected 'from' keyword".to_string()), ind); }

    return match (it.next(), it.next()) {
        (Some(&Token::Id(ref m)), Some(&Token::UseAll)) =>
            (Ok(ASTNode::ImportModUseAll(wrap!(ASTNode::Id(m.to_string())))), ind),
        (Some(&Token::Id(ref m)), Some(&Token::Use)) => parse_module_use(m.to_string(), it, ind),
        (_, Some(&Token::UseAll)) | (_, Some(&Token::Use)) =>
            (Err("Expected identifier.".to_string()), ind),
        (_, _) => (Err("Expected `use` or 'useall'.".to_string()), ind)
    };
}

fn parse_module_use(id: String, it: &mut Peekable<Iter<Token>>, ind: i32)
                    -> (Result<ASTNode, String>, i32) {
    return match (it.next(), it.peek()) {
        (Some(Token::Id(prop)), Some(&Token::As)) => {
            it.next();
            if let Some(&Token::Id(ref other)) = it.next() {
                (Ok(ASTNode::ImportModUseAs(wrap!(ASTNode::Id(id)), wrap!(ASTNode::Id(prop.to_string())),
                                            wrap!(ASTNode::Id(other.to_string())))), ind)
            } else {
                (Err("Expected identifier.".to_string()), ind)
            }
        }
        (Some(Token::Id(prop)), _) =>
            (Ok(ASTNode::ImportModUse(wrap!(ASTNode::Id(id)), wrap!(ASTNode::Id(prop.to_string())))),
             ind),
        (_, _) => (Err("Expected identifier.".to_string()), ind)
    };
}

fn parse_program_do(it: &mut Peekable<Iter<Token>>) -> Option<Result<ASTNode, String>> {
    match parse_do_block(it, 0).0 {
        Ok(ASTNode::Do(expr_or_stmts)) => if expr_or_stmts.len() > 0 {
            Some(Ok(ASTNode::Do(expr_or_stmts)))
        } else { None }
        Ok(_) => None,
        err => Some(err)
    }
}

use crate::lexer::Token;
use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::do_block::parse_do_block;
use crate::parser::function::parse_function_definition_body;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

// module-import    ::= "from" id ( "use" id [ "as" id ] | "useall" )

// module           ::= type | util | class | script
// type             ::= { module-import newline } { newline }
//                     "type" id [ newline { newline }
//                     { ( function-def | definition | immutable-asssign ) newline { newline } } ]
// util             ::= { module-import newline } { newline }
//                      "util" id [ newline [ "isa" id [ { "," id } ] ] [ newline { newline }
//                      { ( immutable-assign | function-def-bod ) newline { newline } } ]
// class            ::= { module-import newline } { newline }
//                      "class" id [ "isa" id [ { "," id } ] ] [ newline { newline }
//                      { ( "util" ( function-def-bod | immutable-assign ) |
//                          "private" ( function-def-bod | assignment ) ) newline { newline } } ]
// script           ::= { module-import newline } { newline }
//                      { function-def newline { newline } }
//                      [ do-block ]

pub fn parse_module(it: &mut Peekable<Iter<TokenPos>>) -> ParseResult<ASTNode> {
    match (parse_multiple(&Token::From, &parse_module_import, it),
           parse_class_name(it),
           parse_multiple(&Token::Fun, &parse_function_definition_body, it),
           parse_script_do(it)) {
        (Ok(_), Some(Ok(_)), Ok(_), Some(Ok(_))) =>
            Err("Class cannot have a body.".to_string()),
        (Ok(imports), Some(Ok(class)), Ok(functions), None) =>
            Ok(ASTNode::ModClass(imports, wrap!(ASTNode::Id(class.to_string())), functions)),

        (Ok(imports), None, Ok(functions), Some(Ok(do_block))) =>
            Ok(ASTNode::ModScript(imports, functions, wrap!(do_block))),
        (Ok(imports), None, Ok(functions), None) =>
            Ok(ASTNode::ModScript(imports, functions, wrap!(ASTNode::Do(Vec::new())))),

        (Err(e), _, _, _) | (_, Some(Err(e)), _, _) | (_, _, Err(e), _) | (_, _, _, Some(Err(e))) =>
            Err(e)
    }
}

fn parse_class_name(it: &mut Peekable<Iter<TokenPos>>) -> Option<ParseResult<ASTNode>> {
    if let Some(&&TokenPos::Class) = it.peek() {
        it.next();
        match (it.next(), it.next()) {
            (Some(TokenPos::Id(name)), Some(TokenPos::NL)) => Some(Ok(name.to_string())),

            (Some(actual), Some(&TokenPos { line, pos, token: Token::NL })) =>
                Err(TokenErr { expected: Token::Id(String::new()), actual }),
            (Some(actual), _) => Err(TokenErr { expected: Token::Id(String::new()), actual }),
            None => Err(EOFErr { expected: Token::Id(String::new()) })
        }
    } else {
        None
    }
}

fn parse_multiple(expected: &Token,
                  fun: &Fn(&mut Peekable<Iter<TokenPos>>, i32) -> (ParseResult<ASTNode>, i32),
                  it: &mut Peekable<Iter<TokenPos>>) -> ParseResult<ASTNode> {
    skip_newlines(it);
    let mut elements = Vec::new();

    while let Some(&t) = it.peek() {
        match *t {
            Some(TokenPos { ref line, ref pos, token }) if token != expected => break
        }

        match fun(it, 0) {
            (Ok(element), _) => elements.push(element),
            (Err(err), _) => return Err(err)
        }

        if it.peek().is_some() && it.next() != Some(&TokenPos::NL) { break; }
        skip_newlines(it);
    }

    return Ok(elements);
}

fn skip_newlines(it: &mut Peekable<Iter<TokenPos>>) {
    while let Some(&t) = it.peek() {
        match t {
            TokenPos::NL => it.next(),
            _ => break
        };
    }
}

fn parse_module_import(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<ASTNode>, i32) {
    match it.next() {
        Some(actual @ TokenPos { ref line, ref pos, token }) if *token != Token::From =>
            return (Err(TokenErr { expected: Token::From, actual }), ind),
        None => return (Err(EOFErr { expected: Token::From }), ind)
    }

    return match (it.next(), it.next()) {
        (Some(&TokenPos::Id(ref m)), Some(&TokenPos::UseAll)) =>
            (Ok(ASTNode::ImportModUseAll(wrap!(ASTNode::Id(m.to_string())))), ind),
        (Some(&TokenPos::Id(ref m)), Some(&TokenPos::Use)) => parse_module_use(m.to_string(), it, ind),

        (Some(actual), Some(&TokenPos { line, pos, token: Token::Id(_) })) =>
            (Err(TokenErr { expected: Token::Use, actual }), ind),
        (Some(actual), _) => (Err(TokenErr { expected: Token::Id(String::new()), actual }), ind),
        None => return (Err(EOFErr { expected: Token::Id(String::new()) }), ind)
    };
}

// module-import    ::= "from" id ( "use" id [ "as" id ] | "useall" )
fn parse_module_use(id: String, it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                    -> (ParseResult<ASTNode>, i32) {
    return match it.next() {
        Some(TokenPos { line, pos, token: Token::Id(module) }) => match it.peek() {
            Some(&TokenPos { line, pos, token: Token::Use }) => match it.next() {
                Some(TokenPos { line, pos, token: Token::Id(useid) }) => match it.peek() {
                    Some(&TokenPos { line, pos, token: Token::As }) => match it.next() {
                        Some(TokenPos { line, pos, token: Token::Id(other) }) =>
                            (Ok(ASTNode::ImportModUseAs(wrap!(ASTNode::Id(module)),
                                                        wrap!(ASTNode::Id(useid.to_string())),
                                                        wrap!(ASTNode::Id(other.to_string())))), ind),
                        Some(actual) => (TokenErr { expected: Token::Id(String::new()), actual },
                                         ind),
                        None => (Err(EOFErr { expected: Token::Id(String::new()) }), ind)
                    }
                    _ => (Ok(ASTNode::ImportModUse(wrap!(ASTNode::Id(id)),
                                                   wrap!(ASTNode::Id(useid.to_string())))), ind)
                }

                Some(actual) => (Err(TokenErr { expected: Token::Id(String::new()), actual }), ind),
                None => (Err(EOFErr { expected: Token::Id(String::new()) }), ind)
            }

            Some(&TokenPos { line, pos, token: Token::UseAll }) =>
                (Ok(ASTNode::ImportModUseAll(wrap!(ASTNode::Id(id)))), ind),

            Some(actual) => (Err(TokenErr { expected: Token::Id(String::new()), actual }), ind),
            None => (Err(EOFErr { expected: Token::Id(String::new()) }), ind)
        }

        Some(actual) => (Err(TokenErr { expected: Token::Id(String::new()), actual }), ind),
        None => (Err(EOFErr { expected: Token::Id(String::new()) }), ind)
    };
}

fn parse_script_do(it: &mut Peekable<Iter<TokenPos>>) -> Option<ParseResult<ASTNode>> {
    match parse_do_block(it, 0).0 {
        Ok(ASTNode::Do(expr_or_stmts)) => if expr_or_stmts.len() > 0 {
            Some(Ok(ASTNode::Do(expr_or_stmts)))
        } else { None }
        Ok(_) => None,
        err => Some(err)
    }
}

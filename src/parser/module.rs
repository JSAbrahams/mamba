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
           parse_module_name(it),
           parse_multiple(&Token::Fun, &parse_function_definition_body, it),
           parse_script_do(it)) {
        (Ok(_), Some(Ok(_)), Ok(_), Some(Ok(_))) => Err(UtilBodyErr),
        (Ok(imports), Some(Ok(name)), Ok(functions), None) =>
            Ok(ASTNode::ModClass(get_or_err!(name), imports, functions)),

        (Ok(imports), None, Ok(functions), Some(Ok(do_block))) =>
            Ok(ASTNode::ModScript(imports, functions, get_or_err!(do_block))),
        (Ok(imports), None, Ok(functions), None) =>
            Ok(ASTNode::ModScript(imports, functions, get_or_err!(ASTNode::Do(Vec::new())))),

        (Err(e), _, _, _) | (_, Some(Err(e)), _, _) | (_, _, Err(e), _) | (_, _, _, Some(Err(e))) =>
            Err(e)
    }
}

fn parse_module_name(_it: &mut Peekable<Iter<TokenPos>>) -> Option<ParseResult<ASTNode>> {
    None
}

fn parse_multiple(expected: &Token,
                  fun: &Fn(&mut Peekable<Iter<TokenPos>>, i32) -> (ParseResult<ASTNode>, i32),
                  it: &mut Peekable<Iter<TokenPos>>) -> ParseResult<Vec<ASTNode>> {
    skip_newlines(it);
    let mut elements: Vec<ASTNode> = Vec::new();

    while let Some(&t) = it.peek() {
        if t.token != *expected { break; }

        match fun(it, 0) {
            (Ok(element), _) => elements.push(element),
            (Err(err), _) => return Err(err)
        }

        match it.next() {
            Some(&TokenPos { line: _, pos: _, token: Token::NL }) => skip_newlines(it),
            _ => break
        }
    }

    return Ok(elements);
}

fn skip_newlines(it: &mut Peekable<Iter<TokenPos>>) {
    while let Some(&t) = it.peek() {
        match t {
            TokenPos { line: _, pos: _, token: Token::NL } => it.next(),
            _ => break
        };
    }
}

fn parse_module_import(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<ASTNode>, i32) {
    check_next_is!(it, ind, Token::From);

    return match (it.next(), it.next()) {
        (Some(TokenPos { line: _, pos: _, token: Token::Id(m) }),
            Some(TokenPos { line: _, pos: _, token: Token::UseAll })) =>
            (Ok(ASTNode::ImportModUseAll(get_or_err!(ASTNode::Id(m.to_string())))), ind),
        (Some(TokenPos { line: _, pos: _, token: Token::Id(m) }),
            Some(TokenPos { line: _, pos: _, token: Token::Use })) =>
            parse_module_use(m.to_string(), it, ind),

        (Some(next), Some(&TokenPos { line: _, pos: _, token: Token::Id(_) })) =>
            (Err(TokenErr { expected: Token::Use, actual: next.clone() }), ind),
        (Some(next), _) =>
            (Err(TokenErr { expected: Token::Id(String::new()), actual: next.clone() }), ind),
        (None, _) => return (Err(EOFErr { expected: Token::Id(String::new()) }), ind)
    };
}

// module-import    ::= "from" id ( "use" id [ "as" id ] | "useall" )
fn parse_module_use(id: String, it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                    -> (ParseResult<ASTNode>, i32) {
    return match it.next() {
        Some(TokenPos { line: _, pos: _, token: Token::Id(module) }) => match it.peek() {
            Some(&TokenPos { line: _, pos: _, token: Token::Use }) => match it.next() {
                Some(TokenPos { line: _, pos: _, token: Token::Id(useid) }) => match it.peek() {
                    Some(&TokenPos { line: _, pos: _, token: Token::As }) => match it.next() {
                        Some(TokenPos { line: _, pos: _, token: Token::Id(other) }) =>
                            (Ok(ASTNode::ImportModUseAs(get_or_err!(ASTNode::Id(module.to_string())),
                                                        get_or_err!(ASTNode::Id(useid.to_string())),
                                                        get_or_err!(ASTNode::Id(other.to_string())))),
                             ind),
                        Some(next) => (Err(
                            TokenErr { expected: Token::Id(String::new()), actual: next.clone() }),
                                       ind),
                        None => (Err(EOFErr { expected: Token::Id(String::new()) }), ind)
                    }
                    _ => (Ok(ASTNode::ImportModUse(get_or_err!(ASTNode::Id(id)),
                                                   get_or_err!(ASTNode::Id(useid.to_string())))), ind)
                }

                Some(next) =>
                    (Err(TokenErr { expected: Token::Id(String::new()), actual: next.clone() }),
                     ind),
                None => (Err(EOFErr { expected: Token::Id(String::new()) }), ind)
            }

            Some(&TokenPos { line: _, pos: _, token: Token::UseAll }) =>
                (Ok(ASTNode::ImportModUseAll(get_or_err!(ASTNode::Id(id)))), ind),
            Some(&next) =>
                (Err(TokenErr { expected: Token::Id(String::new()), actual: next.clone() }), ind),
            None => (Err(EOFErr { expected: Token::Id(String::new()) }), ind)
        }

        Some(next) => (Err(TokenErr { expected: Token::Id(String::new()), actual: next.clone() }),
                       ind),
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

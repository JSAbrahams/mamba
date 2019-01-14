use crate::lexer::Token;
use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::block::parse_block;
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
//    Script(Vec<ASTNode>, Vec<ASTNode>, Box<ASTNode>),
    match parse_block(it, 0) {
        Ok((prog, _)) => Ok((ASTNode::Script(vec![ASTNode::Break], vec![ASTNode::Break],
                                             Box::new(prog)), 0)),
        err => err
    }
}

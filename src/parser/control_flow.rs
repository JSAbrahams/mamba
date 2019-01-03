use crate::lexer::Token;
use crate::parser::ASTNode;
use crate::parser::parse_expression;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

// control-flow-expression     ::= if-expression | when-expression
pub fn parse(it: &mut Peekable<Iter<Token>>, indent: i32) -> (Result<ASTNode, String>, i32) {
    return match it.peek() {
        Some(Token::If) => parse_if_expr(it, indent),
        Some(Token::When) => parse_when_expr(it, indent),

        Some(_) => panic!("Expected control flow statement, but other token."),
        None => panic!("Expected control flow statement, but end of file.")
    };
}

/* if-expression               ::= "if" expression "then"
 * ( newline indent do-block-expression | expression ) [ newline ] "else"
 * ( newline indent do - block-expression | expression )
 */
fn parse_if_expr(it: &mut Peekable<Iter<Token>>, indent: i32) -> (Result<ASTNode, String>, i32) {
    assert_eq!(it.next(), Some(&Token::If));
    match parse_expression(it, indent) {
        (Ok(cond), new_indent) => {
            if it.next() != Some(&Token::Then) {
                return (Err("'Then' keyword expected".to_string()), new_indent);
            }

            match parse_expression(it, new_indent) {
                (Ok(then), nnew_indent) => match it.peek() {
                    Some(Token::Else) => {
                        it.next();
                        match parse_expression(it, nnew_indent) {
                            (Ok(otherwise), nnnew_indent) => (Ok(ASTNode::IfElse(Box::new(cond), Box::new(then), Box::new(otherwise))), nnnew_indent),
                            err => err
                        }
                    }
                    _ => (Ok(ASTNode::If(Box::new(cond), Box::new(then))), nnew_indent)
                }
                err => err
            }
        }
        err => err
    }
}

/* when-expression             ::=
 * "when" expression newline { indent when-case } [ newline indent "else"
 * ( newline indent do-block-expression | expression ) ]
 * when-case                   ::=
 * "equals" expression "then" ( newline indent do-block-expression | expression )
 */
fn parse_when_expr(it: &mut Peekable<Iter<Token>>, indent: i32) -> (Result<ASTNode, String>, i32) {
    return (Err("Not implemented".to_string()), indent);
}

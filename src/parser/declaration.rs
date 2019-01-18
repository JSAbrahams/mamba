use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::ASTNode;
use crate::parser::ASTNodePos;
use crate::parser::end_pos;
use crate::parser::maybe_expr::parse_expression;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::start_pos;
use crate::parser::TPIterator;

pub fn parse_reassignment(pre: ASTNodePos, it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);

    check_next_is!(it, Token::Assign);
    let right: Box<ASTNodePos> = get_or_err!(it, parse_expression, "reassignment");

    return Ok(ASTNodePos {
        st_line,
        st_pos,
        en_line: right.en_line,
        en_pos: right.en_pos,
        node: ASTNode::Assign { left: Box::new(pre), right },
    });
}

pub fn parse_declaration(it: &mut TPIterator) -> ParseResult {
    return match match it.peek() {
        Some(TokenPos { token: Token::Def, .. }) => parse_immutable_declaration(it),
        Some(TokenPos { token: Token::Mut, .. }) => parse_mutable_declaration(it),

        Some(&next) => Err(CustomErr { expected: "declaration".to_string(), actual: next.clone() }),
        None => Err(CustomEOFErr { expected: "declaration".to_string() })
    } {
        Ok(declaration) => match it.peek() {
            Some(TokenPos { token: Token::Forward, .. }) => {
                let mut properties: Vec<ASTNodePos> = Vec::new();
                let mut en_line = None;
                let mut en_pos = None;

                while let Some(t) = it.peek() {
                    match *t {
                        TokenPos { token: Token::NL, .. } => break,
                        TokenPos { token: Token::Comma, .. } => {
                            it.next();
                            let property: ASTNodePos = get_or_err_direct!(it, parse_expression,
                                                                         "defer declaration");
                            en_line = property.en_line;
                            en_pos = property.en_pos;
                            properties.push(property);
                        }
                        next => return Err(TokenErr { expected: Token::Comma, actual: next.clone() })
                    };
                }

                Ok(ASTNodePos {
                    st_line: declaration.st_line,
                    st_pos: declaration.st_pos,
                    en_line,
                    en_pos,
                    node: ASTNode::Defer { declaration: Box::new(declaration), properties },
                })
            }
            _ => Ok(declaration)
        },
        err => err
    };
}

fn parse_mutable_declaration(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);

    check_next_is!(it, Token::Mut);
    let decl: Box<ASTNodePos> = get_or_err!(it, parse_immutable_declaration,
                                            "immutable declaration");

    return Ok(ASTNodePos {
        st_line,
        st_pos,
        en_line: decl.en_line,
        en_pos: decl.en_pos,
        node: ASTNode::Mut { decl },
    });
}

fn parse_immutable_declaration(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);

    let left: Box<ASTNodePos> = get_or_err!(it, parse_definition, "definition");
    check_next_is!(it, Token::Assign);
    let right: Box<ASTNodePos> = get_or_err!(it, parse_expression, "definition");

    return Ok(ASTNodePos {
        st_line,
        st_pos,
        en_line: right.en_line,
        en_pos: right.en_pos,
        node: ASTNode::Assign { left, right },
    });
}

fn parse_definition(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);

    check_next_is!(it, Token::Def);
    match it.next() {
        Some(TokenPos { token: Token::Id(_), .. }) => {
            let ast_id: Box<ASTNodePos> = get_or_err!(it, parse_id, "definition id");
            match it.peek() {
                Some(TokenPos { token: Token::DoublePoint, .. }) => match (it.next(), it.peek()) {
                    (_, Some(TokenPos { token: Token::Id(id), .. })) => {
                        let ast_type: Box<ASTNodePos> = get_or_err!(it, parse_id, "definition type");
                        Ok(ASTNodePos {
                            st_line,
                            st_pos,
                            en_line: ast_type.en_line,
                            en_pos: ast_type.en_pos,
                            node: ASTNode::DefType { id: ast_id, _type: ast_type },
                        })
                    }
                    (_, Some(&next)) => Err(TokenErr {
                        expected: Token::Id(String::new()),
                        actual: next.clone(),
                    }),
                    (_, None) => Err(EOFErr { expected: Token::Id(String::new()) })
                }
                _ => Ok(ASTNodePos {
                    st_line,
                    st_pos,
                    en_line: ast_id.en_line,
                    en_pos: ast_id.en_pos,
                    node: ASTNode::Def { id: ast_id },
                })
            }
        }
        Some(next) => Err(TokenErr { expected: Token::Id(String::new()), actual: next.clone() }),
        None => Err(EOFErr { expected: Token::Id(String::new()) })
    }
}

fn parse_id(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    let (en_line, en_pos) = end_pos(it);

    match it.next() {
        Some(TokenPos { token: Token::Id(id), .. }) => Ok(ASTNodePos {
            st_line,
            st_pos,
            en_line,
            en_pos,
            node: ASTNode::Id { id: id.to_string() },
        }),

        Some(next) => Err(TokenErr { expected: Token::Id(String::new()), actual: next.clone() }),
        None => Err(EOFErr { expected: Token::Id(String::new()) })
    }
}

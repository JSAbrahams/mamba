use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::_type::parse_id_and_type;
use crate::parser::ASTNode;
use crate::parser::ASTNodePos;
use crate::parser::end_pos;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::start_pos;
use crate::parser::TPIterator;

pub fn parse_init(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::Init);

    let args: Vec<ASTNodePos> =
        get_or_err_direct!(it, parse_constructor_args, "constructor arguments");

    let body: Option<Box<ASTNodePos>>;
    if let Some(TokenPos { token: Token::Assign, .. }) = it.peek() {
        it.next();
        body = Some(get_or_err!(it, parse_expr_or_stmt, "constructor body"));
    } else { body = None; }

    let (en_line, en_pos) = match &body {
        Some(b) => (b.en_line, b.en_pos),
        None => end_pos(it)
    };

    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Init { args, body } })
}

pub fn parse_constructor_args(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    check_next_is!(it, Token::LRBrack);
    let mut args: Vec<ASTNodePos> = Vec::new();

    while let Some(&t) = it.peek() {
        match t.token {
            Token::RRBrack => break,
            Token::Comma => continue,
            _ => args.push(get_or_err_direct!(it, parse_constructor_arg, "constructor arguments"))
        }
    }

    check_next_is!(it, Token::RRBrack);
    return Ok(args);
}

pub fn parse_constructor_arg(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);

    let vararg;
    if let Some(TokenPos { token: Token::Vararg, .. }) = it.peek() {
        it.next();
        vararg = true;
    } else { vararg = false; }

    let id_and_type = get_or_err!(it, parse_id_and_type, "constructor argument");

    return Ok(ASTNodePos {
        st_line,
        st_pos,
        en_line: id_and_type.en_line,
        en_pos: id_and_type.en_pos,
        node: ASTNode::InitArg { vararg, id_and_type },
    });
}

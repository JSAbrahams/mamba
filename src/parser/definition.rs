use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::_type::parse_id;
use crate::parser::_type::parse_id_and_type;
use crate::parser::_type::parse_id_maybe_type;
use crate::parser::_type::parse_type;
use crate::parser::ASTNode;
use crate::parser::ASTNodePos;
use crate::parser::end_pos;
use crate::parser::expression::parse_expression;
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
        node: ASTNode::ReAssign { left: Box::new(pre), right },
    });
}

pub fn parse_forward(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::Forward);

    let mut forwarded: Vec<ASTNodePos> = Vec::new();
    let mut en_line = st_line;
    let mut en_pos = st_pos;

    while let Some(t) = it.peek() {
        match *t {
            TokenPos { token: Token::NL, .. } => break,
            TokenPos { token: Token::Comma, .. } => {
                it.next();
                let def: ASTNodePos = get_or_err_direct!(it, parse_id, "forward");
                en_line = def.en_line;
                en_pos = def.en_pos;
                forwarded.push(def);
            }
            next => return Err(TokenErr { expected: Token::Comma, actual: next.clone() })
        };
    }

    return Ok(ASTNodePos {
        st_line,
        st_pos,
        en_line,
        en_pos,
        node: ASTNode::Forward { forwarded },
    });
}

pub fn parse_definition(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::Def);

    let private = it.peek().is_some() && it.peek().unwrap().token == Token::Private;
    if private { it.next(); }

    let definition: Box<ASTNodePos> = match it.peek() {
        Some(TokenPos { token: Token::Mut, .. }) => parse_variable_def(it),

        Some(TokenPos { token: Token::Add, .. }) => parse_operator_def(ASTNode::AddOp, it),
        Some(TokenPos { token: Token::Sub, .. }) => parse_operator_def(ASTNode::SubOp, it),
        Some(TokenPos { token: Token::Sqrt, .. }) => parse_operator_def(ASTNode::SqrtOp, it),
        Some(TokenPos { token: Token::Mul, .. }) => parse_operator_def(ASTNode::MulOp, it),
        Some(TokenPos { token: Token::Div, .. }) => parse_operator_def(ASTNode::DivOp, it),
        Some(TokenPos { token: Token::Pow, .. }) => parse_operator_def(ASTNode::PowOp, it),
        Some(TokenPos { token: Token::Mod, .. }) => parse_operator_def(ASTNode::ModOp, it),
        Some(TokenPos { token: Token::Eq, .. }) => parse_operator_def(ASTNode::EqOp, it),
        Some(TokenPos { token: Token::Ge, .. }) => parse_operator_def(ASTNode::GeOp, it),
        Some(TokenPos { token: Token::Le, .. }) => parse_operator_def(ASTNode::LeOp, it),

        _ => match get_or_err_direct!(it, parse_id_maybe_type, "definition id") {
            id @ ASTNode::IdAndType { .. } => parse_variable_def_id(id, false, it),
            id @ ASTNode::Id { .. } => match it.peek() {
                Some(TokenPos { token: Token::LRBrack, .. }) => parse_fun_def(id, it),
                Some(other) => parse_variable_def_id(id, false, it)
            }
        }
    };

    return Ok(ASTNodePos {
        st_line,
        st_pos,
        en_line: definition.en_line,
        en_pos: definition.en_pos,
        node: ASTNode::Def { private, definition },
    });
}

fn parse_fun_def(id: ASTNodePos, it: &mut TPIterator) -> ParseResult {
    check_next_is!(it, Token::LRBrack);
    let fun_args = get_or_err_direct!(it, parse_fun_args, "function arguments");
    check_next_is!(it, Token::RRBrack);

    check_next_is!(it, Token::DoublePoint);
    let ret_ty: Box<ASTNodePos> = get_or_err!(it, parse_type, "function return type");

    let body: Option<Box<ASTNodePos>>;
    if let Some(TokenPos { token: Token::To, .. }) = it.peek() {
        it.next();
        body = get_or_err!(it, parse_expression, "function body");
    } else { body = None }

    return Ok(ASTNodePos {
        st_line: id.st_line,
        st_pos: id.st_pos,
        en_line: if body.is_some() { body.unwrap().en_line } else { ret_ty.en_line },
        en_pos: if body.is_some() { body.unwrap().en_pos } else { ret_ty.en_pos },
        node: ASTNode::FunDef { id, fun_args, ret_ty, body },
    });
}

fn parse_fun_args(it: &mut TPIterator) -> ParseResult<Vec<ASTNode>> {
    let mut args = Vec::new();

    loop {
        match it.peek() {
            Some(TokenPos { token: Token::RRBrack, .. }) => break,
            Some(_) =>
                args.push(get_or_err_direct!(it, parse_id_and_type, "function argument"))
        }
    }

    return Ok(args);
}

fn parse_fun_arg(it: &mut TPIterator) -> ParseResult {
    unimplemented!()
}

fn parse_variable_def_id(id: ASTNodePos, mutable: bool, it: &mut TPIterator) -> ParseResult {
    let expression: Option<Box<ASTNodePos>>;
    if let Some(TokenPos { token: Token::Assign, .. }) = it.peek() {
        it.next();
        expression = get_or_err!(it, parse_expression, "definition expression");
    } else { expression = None }

    return Ok(ASTNodePos {
        st_line: id.st_line,
        st_pos: id.st_pos,
        en_line: if expression.is_some() { expression.unwrap().en_line } else { id.en_line },
        en_pos: if expression.is_some() { expression.unwrap().en_pos } else { id.en_pos },
        node: ASTNode::VariableDef { mutable, id_maybe_type: *id, expression },
    });
}

fn parse_variable_def(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);

    let mutable;
    if let Some(TokenPos { token: Token::Mut, .. }) = it.peek() {
        it.next();
        mutable = true;
    } else { mutable = false; }

    let id = get_or_err_direct!(it, parse_id_maybe_type, "variable id");
    return parse_variable_def_id(id, mutable, it);
}

fn parse_operator_def(op: ASTNode, it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);

    let arg;
    if let Some(TokenPos { token: Token::LRBrack, .. }) = it.peek() {
        it.next();
        if let Some(TokenPos { token: Token::RRBrack, .. }) = it.peek() {
            it.next();
            arg = Vec::new();
        } else { arg = vec![get_or_err!(it, parse_id_and_type, "operator overloaded argument")] }
    } else { arg = Vec::new() }

    let ret_ty;
    if let Some(TokenPos { token: Token::DoublePoint, .. }) = it.peek() {
        it.next();
        ret_ty = get_or_err!(it, parse_id, "operator overloaded return type");
    } else { ret_ty = ASTNode::_Self; }

    let body: Option<Box<ASTNodePos>>;
    if let Some(TokenPos { token: Token::To, .. }) = it.peek() {
        body = Some(get_or_err!(it, parse_expression, "operator overloaded body"))
    } else { body = None }

    return Ok(ASTNodePos {
        st_line,
        st_pos,
        en_line: if body.is_some() { body.unwrap().en_line } else { ret_ty.en_line },
        en_pos: if body.is_some() { body.unwrap().en_pos } else { ret_ty.en_pos },
        node: ASTNode::FunDef { id: *op, fun_args: arg, ret_ty, body: None },
    });
}

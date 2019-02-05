use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::_type::parse_conditions;
use crate::parser::_type::parse_generics;
use crate::parser::_type::parse_id;
use crate::parser::_type::parse_id_maybe_type;
use crate::parser::_type::parse_type;
use crate::parser::ASTNode;
use crate::parser::ASTNodePos;
use crate::parser::end_pos;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
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

pub fn parse_definition(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::Def);

    let private = it.peek().is_some() && it.peek().unwrap().token == Token::Private;
    if private { it.next(); }

    macro_rules! op{($node:ident) => {{
        let (en_line, en_pos) = end_pos(it);
        it.next();
        parse_fun_def(ASTNodePos{ st_line, st_pos, en_line, en_pos, node: ASTNode::$node }, it)
    }}};

    let definition: ParseResult = match it.peek() {
        Some(TokenPos { token: Token::Mut, .. }) => parse_variable_def(it),

        Some(TokenPos { token: Token::Add, .. }) => op!(AddOp),
        Some(TokenPos { token: Token::Sub, .. }) => op!(SubOp),
        Some(TokenPos { token: Token::Sqrt, .. }) => op!(SqrtOp),
        Some(TokenPos { token: Token::Mul, .. }) => op!(MulOp),
        Some(TokenPos { token: Token::Div, .. }) => op!(DivOp),
        Some(TokenPos { token: Token::Pow, .. }) => op!(PowOp),
        Some(TokenPos { token: Token::Mod, .. }) => op!(ModOp),
        Some(TokenPos { token: Token::Eq, .. }) => op!(EqOp),
        Some(TokenPos { token: Token::Ge, .. }) => op!(GeOp),
        Some(TokenPos { token: Token::Le, .. }) => op!(LeOp),

        _ => match get_or_err_direct!(it, parse_id_maybe_type, "definition id") {
            id @ ASTNodePos { node: ASTNode::TypeId { _type: Some(_), .. }, .. } |
            id @ ASTNodePos { node: ASTNode::TypeTup { .. }, .. } =>
                parse_variable_def_id(id, false, it),
            id @ ASTNodePos { node: ASTNode::TypeId { _type: None, .. }, .. } => match it.peek() {
                Some(TokenPos { token: Token::LRBrack, .. }) => parse_fun_def(id, it),
                Some(other) => parse_variable_def_id(id, false, it),
                None => Err(CustomEOFErr { expected: "id".to_string() })
            }
            other => return Err(InternalErr { message: String::from("couldn't parse def") })
        }
    };

    return match definition {
        Ok(definition) => Ok(ASTNodePos {
            st_line,
            st_pos,
            en_line: definition.en_line,
            en_pos: definition.en_pos,
            node: ASTNode::Def { private, definition: Box::from(definition) },
        }),
        err => err
    };
}

fn parse_fun_def(id: ASTNodePos, it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    let fun_args = get_or_err_direct!(it, parse_fun_args, "function arguments");

    let ret_ty: Option<Box<ASTNodePos>> = match it.peek() {
        Some(TokenPos { token: Token::DoublePoint, .. }) => {
            it.next();
            Some(get_or_err!(it, parse_type, "function return type"))
        }
        _ => None
    };

    let raises: Option<Vec<ASTNodePos>>;
    if let Some(TokenPos { token: Token::Raises, .. }) = it.peek() {
        it.next();
        raises = Some(get_or_err_direct!(it, parse_generics, "raises"));
    } else { raises = None }

    let body: Option<Box<ASTNodePos>>;
    if let Some(TokenPos { token: Token::To, .. }) = it.peek() {
        it.next();
        body = Some(get_or_err!(it, parse_expr_or_stmt, "function body"));
    } else { body = None }

    let (en_line, en_pos) = match (&ret_ty, &raises, &body) {
        (_, _, Some(b)) => (b.en_line, b.en_pos),
        (_, Some(b), _) if b.last().is_some() =>
            (b.last().unwrap().en_line, b.last().unwrap().en_pos),
        (Some(b), _, _) => (b.en_line, b.en_pos),
        _ => (id.en_line, id.en_pos)
    };

    let node = ASTNode::FunDef { id: Box::from(id), fun_args, ret_ty, raises, body };
    return Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node });
}

fn parse_fun_args(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    let mut args = Vec::new();
    let mut pos = 0;
    check_next_is!(it, Token::LRBrack);

    while let Some(&t) = it.peek() {
        match t.token {
            Token::RRBrack => break,
            _ => match parse_fun_arg(it, pos) {
                Ok(arg) => {
                    args.push(arg);
                    pos += 1;
                    if it.peek().is_some() && it.peek().unwrap().token == Token::Comma { it.next(); }
                }
                Err(err) => return Err(err)
            }
        }
    }

    check_next_is!(it, Token::RRBrack);
    return Ok(args);
}

fn parse_fun_arg(it: &mut TPIterator, pos: i32) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    let vararg;
    if let Some(TokenPos { token: Token::Vararg, .. }) = it.peek() {
        it.next();
        vararg = true;
    } else { vararg = false; }

    let id_maybe_type = get_or_err!(it, parse_id_maybe_type, format!("argument (pos {})", pos));

    let (en_line, en_pos) = end_pos(it);
    return Ok(ASTNodePos {
        st_line,
        st_pos,
        en_line,
        en_pos,
        node: ASTNode::FunArg { vararg, id_maybe_type },
    });
}

pub fn parse_forward(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::Forward);

    let mut forwarded: Vec<ASTNodePos> = Vec::new();
    while let Some(&t) = it.peek() {
        match t.token {
            Token::NL => break,
            _ => {
                forwarded.push(get_or_err_direct!(it, parse_id, "forward"));
                if it.peek().is_some() && it.peek().unwrap().token == Token::Comma { it.next(); }
            }
        };
    }

    return Ok(forwarded);
}

fn parse_variable_def_id(id: ASTNodePos, mutable: bool, it: &mut TPIterator) -> ParseResult {
    let expression: Option<Box<ASTNodePos>>;
    if let Some(TokenPos { token: Token::Assign, .. }) = it.peek() {
        it.next();
        expression = Some(get_or_err!(it, parse_expression, "definition expression"));
    } else { expression = None }

    let forward: Option<Vec<ASTNodePos>> = match it.peek() {
        Some(TokenPos { token: Token::Forward, .. }) =>
            Some(get_or_err_direct!(it, parse_forward, "definition raises")),
        _ => None
    };

    let (en_line, en_pos) = match &expression {
        Some(expr) => (expr.en_line, expr.en_pos),
        None => (id.en_line, id.en_pos)
    };

    return Ok(ASTNodePos {
        st_line: id.st_line,
        st_pos: id.st_pos,
        en_line,
        en_pos,
        node: ASTNode::VariableDef {
            mutable,
            id_maybe_type: Box::from(id),
            expression,
            forward,
        },
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

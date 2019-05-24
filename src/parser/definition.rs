use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::_type::parse_generics;
use crate::parser::_type::parse_id;
use crate::parser::_type::parse_id_maybe_type;
use crate::parser::_type::parse_type;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use crate::parser::collection::parse_collection;
use crate::parser::common::end_pos;
use crate::parser::common::start_pos;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::expression::parse_expression;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::TPIterator;

pub fn parse_definition(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::Def);

    let private = it.peek().is_some() && it.peek().unwrap().token == Token::Private;
    if private {
        it.next();
    }

    let pure = it.peek().is_some() && it.peek().unwrap().token == Token::Pure;
    if pure {
        it.next();
    }

    macro_rules! op {
        ($node:ident) => {{
            let (en_line, en_pos) = end_pos(it);
            let node_pos = ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::$node };
            it.next();
            parse_fun_def(node_pos, pure, it)
        }};
    };

    let definition = if pure {
        let id = get_or_err_direct!(it, parse_id_maybe_type, "definition id");
        parse_fun_def(id, pure, it)
    } else {
        match it.peek() {
            Some(TokenPos { token: Token::Mut, .. })
            | Some(TokenPos { token: Token::LRBrack, .. })
            | Some(TokenPos { token: Token::LCBrack, .. })
            | Some(TokenPos { token: Token::LSBrack, .. }) => parse_variable_def(it),

            Some(TokenPos { token: Token::Add, .. }) => op!(AddOp),
            Some(TokenPos { token: Token::Sub, .. }) => op!(SubOp),
            Some(TokenPos { token: Token::Sqrt, .. }) => op!(SqrtOp),
            Some(TokenPos { token: Token::Mul, .. }) => op!(MulOp),
            Some(TokenPos { token: Token::FDiv, .. }) => op!(FDivOp),
            Some(TokenPos { token: Token::Div, .. }) => op!(DivOp),
            Some(TokenPos { token: Token::Pow, .. }) => op!(PowOp),
            Some(TokenPos { token: Token::Mod, .. }) => op!(ModOp),
            Some(TokenPos { token: Token::Eq, .. }) => op!(EqOp),
            Some(TokenPos { token: Token::Ge, .. }) => op!(GeOp),
            Some(TokenPos { token: Token::Le, .. }) => op!(LeOp),

            _ => match get_or_err_direct!(it, parse_id_maybe_type, "definition id") {
                id @ ASTNodePos { node: ASTNode::IdType { _type: Some(_), .. }, .. }
                | id @ ASTNodePos { node: ASTNode::TypeTup { .. }, .. } =>
                    parse_variable_def_id(id, it),
                id @ ASTNodePos {
                    node: ASTNode::IdType { _type: None, mutable: false, .. },
                    ..
                } => match it.peek() {
                    Some(TokenPos { token: Token::LRBrack, .. }) => parse_fun_def(id, pure, it),
                    None | Some(_) => parse_variable_def_id(id, it)
                },
                _ => return Err(InternalErr { message: String::from("couldn't parse def") })
            }
        }
    };

    match definition {
        Ok(definition) => Ok(ASTNodePos {
            st_line,
            st_pos,
            en_line: definition.en_line,
            en_pos: definition.en_pos,
            node: ASTNode::Def { private, definition: Box::from(definition) }
        }),
        err => err
    }
}

fn parse_fun_def(id_type: ASTNodePos, pure: bool, it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    let fun_args = get_or_err_direct!(it, parse_fun_args, "function arguments");

    let id = match &id_type {
        ASTNodePos { node: ASTNode::IdType { id, mutable, _type }, .. } => match (mutable, _type) {
            (false, None) => id.clone(),
            (true, _) =>
                return Err(InternalErr {
                    message: String::from("Function definition cannot be mutable")
                }),
            (_, Some(_)) =>
                return Err(InternalErr {
                    message: String::from("Function definition given id type with some type.")
                }),
        },

        op @ ASTNodePos { node: ASTNode::AddOp, .. } => Box::from(op.clone()),
        op @ ASTNodePos { node: ASTNode::SubOp, .. } => Box::from(op.clone()),
        op @ ASTNodePos { node: ASTNode::SqrtOp, .. } => Box::from(op.clone()),
        op @ ASTNodePos { node: ASTNode::MulOp, .. } => Box::from(op.clone()),
        op @ ASTNodePos { node: ASTNode::DivOp, .. } => Box::from(op.clone()),
        op @ ASTNodePos { node: ASTNode::FDivOp, .. } => Box::from(op.clone()),
        op @ ASTNodePos { node: ASTNode::PowOp, .. } => Box::from(op.clone()),
        op @ ASTNodePos { node: ASTNode::ModOp, .. } => Box::from(op.clone()),
        op @ ASTNodePos { node: ASTNode::EqOp, .. } => Box::from(op.clone()),
        op @ ASTNodePos { node: ASTNode::GeOp, .. } => Box::from(op.clone()),
        op @ ASTNodePos { node: ASTNode::LeOp, .. } => Box::from(op.clone()),

        other =>
            return Err(InternalErr {
                message: format!("Function definition not given id or operator: {:?}", other)
            }),
    };

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
    } else {
        raises = None
    }

    let body: Option<Box<ASTNodePos>>;
    if let Some(TokenPos { token: Token::BTo, .. }) = it.peek() {
        it.next();
        body = Some(get_or_err!(it, parse_expr_or_stmt, "function body"));
    } else {
        body = None
    }

    let (en_line, en_pos) = match (&ret_ty, &raises, &body) {
        (_, _, Some(b)) => (b.en_line, b.en_pos),
        (_, Some(b), _) if b.last().is_some() =>
            (b.last().unwrap().en_line, b.last().unwrap().en_pos),
        (Some(b), ..) => (b.en_line, b.en_pos),
        _ => (id_type.en_line, id_type.en_pos)
    };

    let node = ASTNode::FunDef { id, pure, fun_args, ret_ty, raises, body };
    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
}

pub fn parse_fun_args(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
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
                    if it.peek().is_some() && it.peek().unwrap().token == Token::Comma {
                        it.next();
                    }
                }
                Err(err) => return Err(err)
            }
        }
    }

    check_next_is!(it, Token::RRBrack);
    Ok(args)
}

pub fn parse_fun_arg(it: &mut TPIterator, pos: i32) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    let vararg;
    if let Some(TokenPos { token: Token::Vararg, .. }) = it.peek() {
        it.next();
        vararg = true;
    } else {
        vararg = false;
    }

    let id_maybe_type = get_or_err!(it, parse_id_maybe_type, format!("argument (pos {})", pos));
    let default = match it.peek() {
        Some(TokenPos { token: Token::Assign, .. }) => {
            it.next();
            Some(get_or_err!(it, parse_expression, format!("argument default (pos {})", pos)))
        }
        _ => None
    };

    let (en_line, en_pos) = end_pos(it);
    let node = ASTNode::FunArg { vararg, id_maybe_type, default };
    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
}

pub fn parse_forward(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    check_next_is!(it, Token::Forward);

    let mut forwarded: Vec<ASTNodePos> = Vec::new();
    while let Some(&t) = it.peek() {
        match t.token {
            Token::NL => break,
            _ => {
                forwarded.push(get_or_err_direct!(it, parse_id, "forward"));
                if it.peek().is_some() && it.peek().unwrap().token == Token::Comma {
                    it.next();
                }
            }
        };
    }

    Ok(forwarded)
}

fn parse_variable_def_id(id: ASTNodePos, it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = (id.st_line, id.st_pos);
    let ofmut;
    if let Some(TokenPos { token: Token::OfMut, .. }) = it.peek() {
        it.next();
        ofmut = true;
    } else {
        ofmut = false
    }

    let expression: Option<Box<ASTNodePos>>;
    if let Some(TokenPos { token: Token::Assign, .. }) = it.peek() {
        it.next();
        expression = Some(get_or_err!(it, parse_expression, "definition expression"));
    } else {
        expression = None
    }

    let forward: Vec<ASTNodePos> = match it.peek() {
        Some(TokenPos { token: Token::Forward, .. }) =>
            get_or_err_direct!(it, parse_forward, "definition raises"),
        _ => vec![]
    };

    let (en_line, en_pos) = match &expression {
        Some(expr) => (expr.en_line, expr.en_pos),
        None => (id.en_line, id.en_pos)
    };

    let node = ASTNode::VariableDef { ofmut, id_maybe_type: Box::from(id), expression, forward };
    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
}

fn parse_variable_def(it: &mut TPIterator) -> ParseResult {
    let id = match it.peek() {
        Some(TokenPos { token: Token::LRBrack, .. })
        | Some(TokenPos { token: Token::LCBrack, .. })
        | Some(TokenPos { token: Token::LSBrack, .. }) =>
            get_or_err_direct!(it, parse_collection, "collection"),
        _ => get_or_err_direct!(it, parse_id_maybe_type, "variable id")
    };

    parse_variable_def_id(id, it)
}

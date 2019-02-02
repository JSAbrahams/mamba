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

    macro_rules! op{($node:ident) => {{
        let (en_line, en_pos) = end_pos(it);
        parse_operator_def(ASTNodePos{ st_line, st_pos, en_line, en_pos, node: ASTNode::$node }, it)
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
            id @ ASTNodePos { node: ASTNode::IdAndType { .. }, .. } =>
                parse_variable_def_id(id, false, it),
            id @ ASTNodePos { node: ASTNode::Id { .. }, .. } => match it.peek() {
                Some(TokenPos { token: Token::LRBrack, .. }) => parse_fun_def(id, it),
                Some(other) => parse_variable_def_id(id, false, it),
                None => Err(CustomEOFErr { expected: "id".to_string() })
            }
            other => panic!("{:?}", other)
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
    check_next_is!(it, Token::LRBrack);
    let fun_args = get_or_err_direct!(it, parse_fun_args, "function arguments");
    check_next_is!(it, Token::RRBrack);

    check_next_is!(it, Token::DoublePoint);
    let ret_ty: Box<ASTNodePos> = get_or_err!(it, parse_type, "function return type");

    let body: Option<Box<ASTNodePos>>;
    if let Some(TokenPos { token: Token::To, .. }) = it.peek() {
        it.next();
        body = Some(get_or_err!(it, parse_expression, "function body"));
    } else { body = None }

    let (en_line, en_pos) = match &body {
        Some(b) => (b.en_line, b.en_pos),
        None => (ret_ty.en_line, ret_ty.en_pos)
    };

    let node = ASTNode::FunDef { id: Box::from(id), fun_args, ret_ty, body };
    return Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node });
}

fn parse_fun_args(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    let mut args = Vec::new();

    loop {
        match it.peek() {
            Some(TokenPos { token: Token::RRBrack, .. }) => break,
            Some(_) =>
                args.push(get_or_err_direct!(it, parse_id_and_type, "function argument")),
            None => return Err(EOFErr { expected: Token::RRBrack })
        }
    }

    return Ok(args);
}

fn parse_fun_arg(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    let vararg;
    if let Some(TokenPos { token: Token::Vararg, .. }) = it.peek() {
        it.next();
        vararg = true;
    } else { vararg = false; }

    let id_and_type = get_or_err!(it, parse_id_and_type, "function argument");

    let (en_line, en_pos) = end_pos(it);
    return Ok(ASTNodePos {
        st_line,
        st_pos,
        en_line,
        en_pos,
        node: ASTNode::FunArg { vararg, id_and_type },
    });
}

fn parse_variable_def_id(id: ASTNodePos, mutable: bool, it: &mut TPIterator) -> ParseResult {
    let expression: Option<Box<ASTNodePos>>;
    if let Some(TokenPos { token: Token::Assign, .. }) = it.peek() {
        it.next();
        expression = Some(get_or_err!(it, parse_expression, "definition expression"));
    } else { expression = None }

    let (en_line, en_pos) = match &expression {
        Some(expr) => (expr.en_line, expr.en_pos),
        None => (id.en_line, id.en_pos)
    };

    return Ok(ASTNodePos {
        st_line: id.st_line,
        st_pos: id.st_pos,
        en_line,
        en_pos,
        node: ASTNode::VariableDef { mutable, id_maybe_type: Box::from(id), expression },
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

fn parse_operator_def(op: ASTNodePos, it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);

    let args;
    if let Some(TokenPos { token: Token::LRBrack, .. }) = it.peek() {
        it.next();
        if let Some(TokenPos { token: Token::RRBrack, .. }) = it.peek() {
            it.next();
            args = Vec::new();
        } else {
            args = vec![get_or_err_direct!(it, parse_id_and_type, "operator overloaded argument")]
        }
    } else { args = Vec::new() }

    let ret_ty;
    let (en_line, en_pos) = end_pos(it);
    if let Some(TokenPos { token: Token::DoublePoint, .. }) = it.peek() {
        it.next();
        ret_ty = get_or_err!(it, parse_id, "operator overloaded return type");
    } else {
        ret_ty = Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::_Self })
    }

    let body: Option<Box<ASTNodePos>>;
    if let Some(TokenPos { token: Token::To, .. }) = it.peek() {
        body = Some(get_or_err!(it, parse_expression, "operator overloaded body"))
    } else { body = None }

    let (en_line, en_pos) = match &body {
        Some(b) => (b.en_line, b.en_pos),
        None => (ret_ty.en_line, ret_ty.en_pos)
    };

    return Ok(ASTNodePos {
        st_line,
        st_pos,
        en_line,
        en_pos,
        node: ASTNode::FunDef { id: Box::from(op), fun_args: args, ret_ty, body },
    });
}

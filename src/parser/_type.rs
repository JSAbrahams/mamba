use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use crate::parser::expression::parse_expression;
use crate::parser::iterator::TPIterator;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;

pub fn parse_id(it: &mut TPIterator) -> ParseResult {
    it.next(&|token| match token {
        Some(TokenPos { token: Token::_Self, .. }) => Ok(ASTNode::_Self),
        Some(TokenPos { token: Token::Init, .. }) => Ok(ASTNode::Init),
        Some(TokenPos { token: Token::Id(id), .. }) => Ok(ASTNode::Id { lit: id.to_string() }),
        Some(next) => Err(CustomErr { expected: String::from("id"), actual: next.clone() }),
        None => Err(EOFErr { expected: Token::Id(String::new()) })
    })
}

pub fn parse_generics(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    it.eat(Token::LSBrack);

    let mut generics: Vec<ASTNodePos> = Vec::new();
    it.while_some_and_not(Token::RSBrack, &|| {
        generics.push(*it.parse(&parse_generic, "generic")?);
        it.eat_if(Token::Comma)
    });

    it.eat(Token::RSBrack);
    Ok(generics)
}

fn parse_generic(it: &mut TPIterator) -> ParseResult {
    let id = it.parse(&parse_id, "generic id")?;
    let isa = it.parse_or_none(Token::IsA, &parse_id, "generic isa")?;

    let (st_line, st_pos) = it.start_pos()?;
    let (en_line, en_pos) = match isa.as_ref() {
        Some(ast) => (ast.en_line, ast.en_pos),
        None => (id.en_line, id.en_pos)
    };
    let node = ASTNode::Generic { id, isa };
    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
}

pub fn parse_type(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;

    let _type: ASTNodePos = match it.peek() {
        Some(TokenPos { token: Token::Id(_), .. }) => {
            let id: Box<ASTNodePos> = it.parse(parse_id, "type");
            let generics: Vec<ASTNodePos> = match it.peek() {
                Some(TokenPos { token: Token::LSBrack, .. }) =>
                    it.parse(parse_generics, "type generic"),
                _ => vec![]
            };

            let (en_line, en_pos) = match generics.last() {
                Some(generic) => (generic.en_line, generic.en_pos),
                None => (id.en_line, id.en_pos)
            };

            let node = ASTNode::Type { id, generics };
            ASTNodePos { st_line, st_pos, en_line, en_pos, node }
        }
        _ => it.parse(parse_type_tuple, "type")
    };

    match it.peek() {
        Some(TokenPos { token: Token::To, .. }) => {
            it.next();
            let right: Box<ASTNodePos> = it.parse(parse_type, "type");
            Ok(ASTNodePos {
                st_line,
                st_pos,
                en_line: right.en_line,
                en_pos: right.en_pos,
                node: ASTNode::TypeFun { _type: Box::from(_type), body: right }
            })
        }
        _ => Ok(_type)
    }
}

pub fn parse_conditions(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    it.eat(Token::When);
    match it.peek() {
        Some(TokenPos { token: Token::NL, .. }) => {
            it.next();
        }
        _ => return Ok(vec![it.parse(parse_condition, "single condition")])
    }

    it.eat(Token::Indent);
    let mut conditions = Vec::new();
    while let Some(&t) = it.peek() {
        match t.token {
            Token::Dedent => break,
            Token::NL => {
                it.next();
            }
            _ => conditions.push(it.parse(parse_condition, "condition"))
        }
    }

    if it.peek().is_some() {
        it.eat(Token::Dedent);
    }
    Ok(conditions)
}

fn parse_condition(it: &mut TPIterator) -> ParseResult {
    let condition: Box<ASTNodePos> = it.parse(parse_expression, "condition");
    let _else: Option<Box<ASTNodePos>> = match it.peek() {
        Some(TokenPos { token: Token::Else, .. }) =>
            Some(it.parse(parse_expression, "condition else")),
        _ => None
    };

    let (en_line, en_pos) = match &_else {
        Some(ast_pos) => (ast_pos.en_line, ast_pos.en_pos),
        None => (condition.en_line, condition.en_pos)
    };

    let (st_line, st_pos) = (condition.st_line, condition.st_pos);
    let node = ASTNode::Condition { cond: condition, _else };
    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
}

pub fn parse_type_tuple(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    it.eat(Token::LRBrack);

    let mut types: Vec<ASTNodePos> = Vec::new();
    let mut en_line = st_line;
    let mut en_pos = st_pos;

    if it.peek().is_some() && it.peek().unwrap().token != Token::RRBrack {
        let id = it.parse(parse_type, "type tuple");
        types.push(id);
    }

    while let Some(t) = it.peek() {
        match *t {
            TokenPos { token: Token::RRBrack, .. } => break,
            TokenPos { token: Token::Comma, .. } => {
                it.next();

                let _type: ASTNodePos = it.parse(parse_type, "type");
                en_line = _type.en_line;
                en_pos = _type.en_pos;
                types.push(_type);
            }
            next => return Err(TokenErr { expected: Token::Comma, actual: next.clone() })
        };
    }

    it.eat(Token::RRBrack);
    let node = ASTNode::TypeTup { types };
    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
}

pub fn parse_id_maybe_type(it: &mut TPIterator) -> ParseResult {
    let mutable;
    if it.peek().is_some() && it.peek().unwrap().token == Token::Mut {
        mutable = true;
        it.next();
    } else {
        mutable = false;
    }

    let id: Box<ASTNodePos> = it.parse(parse_id, "id maybe type");

    let (en_line, en_pos, _type) = match it.peek() {
        Some(TokenPos { token: Token::DoublePoint, .. }) => {
            it.next();
            let _type: Box<ASTNodePos> = it.parse(parse_type, "id type");
            (_type.en_line, _type.en_pos, Some(_type))
        }
        _ => (id.en_line, id.en_pos, None)
    };

    let (st_line, st_pos) = (id.st_line, id.st_pos);
    let node = ASTNode::IdType { id, mutable, _type };
    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
}

use crate::lexer::classify::Tokens;
use crate::parser::ast::{Function, Parser, ParserError, Result, Struct};

pub(crate) enum Item<'str> {
    Function(Function<'str>),
    Struct(Struct<'str>),
}

pub(super) struct Program<'str> {
    pub(super) items: Vec<Item<'str>>,
}

pub(crate) fn ast_generate<'str>(tokens: &'str [Tokens<'str>]) -> Result<'str, Vec<Item<'str>>> {
    let mut parser = Parser::new(tokens);

    let mut items = Vec::new();

    while let Some(token) = parser.peek().cloned() {
        let item = match token {
            Tokens::Fn => Item::Function(parser.parse_fn_definition()?),
            Tokens::Struct => Item::Struct(parser.parse_struct()?),
            _ => return Err(ParserError::ExpectedType),
        };
        items.push(item);
    }

    Ok(items)
}

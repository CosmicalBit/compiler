use crate::lexer::classify::Tokens;
use crate::parser::ast::{Parser, ParserError, Result, Statement, Type};

impl<'src> Parser<'src> {
    pub(crate) fn parse_type(&mut self) -> Result<Type<'src>> {
        match self.next() {
            Some(Tokens::Other(typename)) => Ok(Type::Named(typename)),
            _ => Err(ParserError::MissingType),
        }
    }
}

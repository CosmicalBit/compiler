use crate::lexer::classify::Tokens;
use crate::parser::ast::{Parser, ParserError, Result, Statement, Type};

impl<'src> Parser<'src> {
    fn parse_types(&mut self) -> Result<Type<'src>> {
        match self.next() {
            Some(Tokens::Other(typename)) => Ok(Type::Named(typename)),
            _ => Err(ParserError::MissingVarType),
        }
    }
}

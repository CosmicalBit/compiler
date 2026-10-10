use crate::{
    lexer::classify::Tokens,
    parser::ast::{Parser, ParserError, Result, Struct},
};

impl<'src> Parser<'src> {
    pub(super) fn parse_struct(&mut self) -> Result<Struct<'src>> {
        self.expect(Tokens::Struct, ParserError::MissingStructDefinition)?;

        let name = match self.next() {
            Some(Tokens::Other(name)) => *name,
            _ => return Err(ParserError::MissingStructName),
        };

        self.expect(Tokens::LeftBrace, ParserError::MissingLeftBrace)?;

        let fields = self.parse_named_types(&Tokens::RightBrace)?;

        Ok(Struct { name, fields })
    }
}

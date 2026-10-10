use crate::{
    lexer::classify::Tokens,
    parser::{
        ast::{Parser, ParserError, Result, Struct},
        generic_params,
    },
};

impl<'src> Parser<'src> {
    pub(super) fn parse_struct(&mut self) -> Result<'src, Struct<'src>> {
        self.expect(Tokens::Struct, ParserError::MissingStructDefinition)?;

        let name = match self.next() {
            Some(Tokens::Other(name)) => *name,
            _ => return Err(ParserError::MissingStructName),
        };

        let generic_params = self.parse_generic_params()?;

        self.expect(Tokens::LeftBrace, ParserError::MissingLeftBrace)?;

        let fields = self.parse_named_types(&Tokens::RightBrace)?;

        Ok(Struct {
            name,
            generic_parameters: generic_params,
            fields,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_generic_struct_fields() {
        let tokens = [
            Tokens::Struct,
            Tokens::Other("Pair"),
            Tokens::Less,
            Tokens::Other("T"),
            Tokens::Bigger,
            Tokens::LeftBrace,
            Tokens::Other("left"),
            Tokens::Colon,
            Tokens::Other("T"),
            Tokens::Comma,
            Tokens::Other("right"),
            Tokens::Colon,
            Tokens::Other("T"),
            Tokens::Comma,
            Tokens::RightBrace,
        ];
        let mut parser = Parser::new(&tokens);

        let structure = parser.parse_struct().unwrap();
        assert_eq!(structure.name, "Pair");
        assert_eq!(structure.generic_parameters, vec![generic_params::GenericParam::Type("T")]);
        assert_eq!(structure.fields.len(), 2);
        assert_eq!(structure.fields[0].name, "left");
        assert_eq!(structure.fields[1].name, "right");
        assert_eq!(parser.peek(), None);
    }

    #[test]
    fn reports_missing_struct_name() {
        let tokens = [Tokens::Struct, Tokens::LeftBrace];
        let mut parser = Parser::new(&tokens);
        assert_eq!(parser.parse_struct(), Err(ParserError::MissingStructName));
    }
}

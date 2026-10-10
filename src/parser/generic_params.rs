use crate::{
    lexer::classify::Tokens,
    parser::ast::{
        Parser,
        ParserError::{self, MissingCommaOrRightParen, UnexpectedTokenInGenerics},
        Result, Struct,
    },
};

#[derive(Debug, PartialEq,Eq)]
pub(crate) enum GenericParam<'src> {
    Type(&'src str),
    Lifetime(&'src str),
}

impl<'src> Parser<'src> {
    pub(crate) fn parse_generic_params(&mut self) -> Result<'src, Vec<GenericParam<'src>>> {
        let mut params = Vec::new();

        if !matches!(self.peek(), Some(Tokens::Less)) {
            return Ok(params);
        }
        //consume <
        self.next();

        loop {
            let param = match self.next() {
                Some(Tokens::Other(name)) => GenericParam::Type(name),
                Some(Tokens::Tick) => match self.next() {
                    Some(Tokens::Other(name)) => GenericParam::Lifetime(name),
                    _ => return Err(ParserError::MissingLifetimeName),
                },
                Some(Tokens::Bigger) => return Err(ParserError::EmptyGenericParams),
                Some(err) => return Err(ParserError::UnexpectedTokenInGenerics(*err)),
                None => return Err(ParserError::EOF),
            };

            params.push(param);

            match self.next() {
                Some(Tokens::Comma) => {
                    if matches!(self.peek(), Some(Tokens::Bigger)) {
                        self.next();
                        return Ok(params);
                    }
                }
                Some(Tokens::Bigger) => {
                    return Ok(params);
                }
                Some(token) => return Err(UnexpectedTokenInGenerics(*token)),
                _ => return Err(ParserError::EOF),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_type_and_lifetime_parameters_with_trailing_comma() {
        let tokens = [
            Tokens::Less,
            Tokens::Other("T"),
            Tokens::Comma,
            Tokens::Tick,
            Tokens::Other("a"),
            Tokens::Comma,
            Tokens::Bigger,
        ];
        let mut parser = Parser::new(&tokens);

        assert_eq!(
            parser.parse_generic_params(),
            Ok(vec![GenericParam::Type("T"), GenericParam::Lifetime("a")])
        );
        assert_eq!(parser.peek(), None);
    }

    #[test]
    fn reports_empty_and_unclosed_generic_parameters() {
        let empty = [Tokens::Less, Tokens::Bigger];
        let mut parser = Parser::new(&empty);
        assert_eq!(parser.parse_generic_params(), Err(ParserError::EmptyGenericParams));

        let unclosed = [Tokens::Less, Tokens::Other("T")];
        let mut parser = Parser::new(&unclosed);
        assert_eq!(parser.parse_generic_params(), Err(ParserError::EOF));
    }
}

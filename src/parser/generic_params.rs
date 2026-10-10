use crate::{
    lexer::classify::Tokens,
    parser::ast::{
        Parser,
        ParserError::{self, MissingCommaOrRightParen, UnexpectedTokenInGenerics},
        Result, Struct,
    },
};

pub(crate) enum GenericParam<'src, T, U> {
    Type(&'src str),
    Lifetime(&'src str),
}

impl<'src> Parser<'src> {
    pub(crate) fn parse_generic_params(&mut self) -> Result<Vec<GenericParam<'src>>> {
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

use crate::lexer::classify::Tokens;
use crate::parser::ast::{Expressions, Function, Parameter, Parser, ParserError, Result};

impl<'src> Parser<'src> {
    /// Parse the contents of a function's parentheses.
    pub(super) fn parse_named_types(&mut self, end: &Tokens<'src>) -> Result<Vec<Parameter<'src>>> {
        let mut parameters = Vec::new();

        if self.peek() == Some(end) {
            self.next();
            return Ok(parameters);
        }

        loop {
            let name = match self.next() {
                Some(Tokens::Other(name)) => *name,
                _ => return Err(ParserError::MissingParameterName),
            };
            match self.next() {
                Some(Tokens::Colon) => (),
                _ => return Err(ParserError::MissingParameterType),
            }
            let var_type = self.parse_type()?;
            parameters.push(Parameter { name, var_type });

            match self.next() {
                Some(Tokens::Comma) => {
                    if self.peek() == Some(end) {
                        self.next();
                        return Ok(parameters);
                    }
                    continue;
                }
                Some(token) if token == end => return Ok(parameters),
                _ => return Err(ParserError::MissingCommaOrRightParen),
            }
        }
    }

    //parses fucntion arguments aka for calls
    pub(super) fn parse_arguments(&mut self) -> Result<Vec<Expressions<'src>>> {
        let mut args = Vec::new();

        if matches!(self.peek(), Some(Tokens::RightParen)) {
            self.next();
            return Ok(args);
        }
        loop {
            let arg = self.parse_expression(0)?;

            args.push(arg);

            match self.next() {
                Some(Tokens::Comma) => continue,
                Some(Tokens::RightParen) => return Ok(args),
                _ => return Err(ParserError::MissingCommaOrRightParen),
            }
        }
    }
    //parses fn defenition
    pub(super) fn parse_fn_definition(&mut self) -> Result<Function<'src>> {
        self.expect(Tokens::Fn, ParserError::MissingFn)?;

        let name = match self.next() {
            Some(Tokens::Other(name)) => *name,
            _ => return Err(ParserError::MissingFnName),
        };
        match self.next() {
            Some(Tokens::LeftParen) => (),
            _ => return Err(ParserError::MissingLeftParen),
        }
        let parameters = self.parse_named_types(&Tokens::RightParen)?;
        match self.next() {
            Some(Tokens::Arrow) => (),
            _ => return Err(ParserError::MissingArrow),
        }
        let return_type = self.parse_type()?;
        if !matches!(self.peek(), Some(Tokens::LeftBrace)) {
            return Err(ParserError::MissingFunctionBody);
        }
        let body = self.parse_block()?;

        Ok(Function {
            name,
            parameters,
            return_type,
            body,
        })
    }
}

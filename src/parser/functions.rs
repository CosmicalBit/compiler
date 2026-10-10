use crate::lexer::classify::Tokens;
use crate::parser::ast::{Expressions, Function, Parameter, Parser, ParserError, Result};
use crate::parser::generic_params::GenericParam;
impl<'src> Parser<'src> {
    /// Parse the contents of a function's parentheses.
    pub(super) fn parse_named_types(&mut self, end: &Tokens<'src>) -> Result<'src, Vec<Parameter<'src>>> {
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
    pub(super) fn parse_arguments(&mut self) -> Result<'src, Vec<Expressions<'src>>> {
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
    pub(super) fn parse_fn_definition(&mut self) -> Result<'src, Function<'src>> {
        self.expect(Tokens::Fn, ParserError::MissingFn)?;

        let name = match self.next() {
            Some(Tokens::Other(name)) => *name,
            _ => return Err(ParserError::MissingFnName),
        };
        let generic_params = self.parse_generic_params()?;

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
            generic_parameter: generic_params,
            parameters,
            return_type,
            body,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::Type;

    #[test]
    fn parses_function_with_parameters_return_type_and_body() {
        let tokens = [
            Tokens::Fn,
            Tokens::Other("identity"),
            Tokens::LeftParen,
            Tokens::Other("value"),
            Tokens::Colon,
            Tokens::Other("i32"),
            Tokens::RightParen,
            Tokens::Arrow,
            Tokens::Other("i32"),
            Tokens::LeftBrace,
            Tokens::RightBrace,
        ];
        let mut parser = Parser::new(&tokens);

        let function = parser.parse_fn_definition().unwrap();
        assert_eq!(function.name, "identity");
        assert_eq!(function.parameters.len(), 1);
        assert_eq!(function.parameters[0].name, "value");
        assert_eq!(function.parameters[0].var_type, Type::Named("i32"));
        assert_eq!(function.return_type, Type::Named("i32"));
        assert_eq!(parser.peek(), None);
    }

    #[test]
    fn reports_missing_function_body_after_return_type() {
        let tokens = [
            Tokens::Fn,
            Tokens::Other("identity"),
            Tokens::LeftParen,
            Tokens::RightParen,
            Tokens::Arrow,
            Tokens::Other("i32"),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            parser.parse_fn_definition(),
            Err(ParserError::MissingFunctionBody)
        );
    }
}

use crate::lexer::classify::Tokens;
use crate::parser::ast::{Parser, ParserError, Result, Statement};

impl<'src> Parser<'src> {
    pub(super) fn parse_declaration(&mut self) -> Result<Statement<'src>> {
        self.expect(Tokens::Let, ParserError::MissingLetStatement)?;

        let vname = match self.next() {
            Some(Tokens::Other(name)) => name,
            _ => return Err(ParserError::MissingVarName),
        };

        match self.next() {
            Some(Tokens::Colon) => (),
            _ => todo!("type inference not added"),
        };

        let vartype = self.parse_type()?;
        self.expect(Tokens::Equals, ParserError::MissingEqualSign)?;

        let exp = self.parse_expression(0)?;

        // Consume the semicolon only after confirming it is present.
        self.expect(Tokens::SemiColon, ParserError::MissingSemicolon)?;

        Ok(Statement::Let {
            name: vname,
            value: exp,
            var_type: Some(vartype),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::Type;

    #[test]
    fn parses_let_statement() {
        let tokens = [
            Tokens::Let,
            Tokens::Other("answer"),
            Tokens::Colon,
            Tokens::Other("i64"),
            Tokens::Equals,
            Tokens::Integer(42),
            Tokens::SemiColon,
        ];
        let mut parser = Parser::new(&tokens);

        let statement = parser.parse_declaration().unwrap();
        assert!(matches!(
            statement,
            Statement::Let {
                name: "answer",
                value: crate::parser::ast::Expressions::Integer(42),
                var_type: Some(Type::Named("i64")),  
            }
        ));
    }

    #[test]
    fn declaration_requires_let_keyword() {
        let tokens = [Tokens::Other("answer")];
        let mut parser = Parser::new(&tokens);

        assert!(matches!(
            parser.parse_declaration(),
            Err(ParserError::MissingLetStatement)
        ));
    }


    #[test]
    fn declaration_requires_equal_sign() {
        let tokens = [
            Tokens::Let,
            Tokens::Other("answer"),
            Tokens::Colon,
            Tokens::Other("i64"),
            Tokens::Integer(42),
        ];
        let mut parser = Parser::new(&tokens);

        assert_eq!(
            parser.parse_declaration(),
            Err(ParserError::MissingEqualSign)
        );
        assert_eq!(parser.peek(), Some(&Tokens::Integer(42)));
    }

    #[test]
    fn missing_declaration_semicolon_does_not_consume_next_token() {
        let tokens = [
            Tokens::Let,
            Tokens::Other("answer"),
            Tokens::Colon,
            Tokens::Other("i64"),
            Tokens::Equals,
            Tokens::Integer(42),
            Tokens::Other("next"),
        ];
        let mut parser = Parser::new(&tokens);

        assert_eq!(
            parser.parse_declaration(),
            Err(ParserError::MissingSemicolon)
        );
        assert_eq!(parser.peek(), Some(&Tokens::Other("next")));
    }
}

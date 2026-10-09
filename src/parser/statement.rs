use crate::lexer::classify::Tokens;
use crate::parser::ast::{Parser, ParserError, Result, Statement, Type};

impl<'src> Parser<'src> {
    pub(crate) fn parse_declaration(&mut self) -> Result<Statement<'src>> {
        self.expect(Tokens::Let, ParserError::MissingLetStatement)?;
        self.next();

        let vname = match self.next() {
            Some(Tokens::Other(name)) => name,
            _ => return Err(ParserError::MissingVarName),
        };

        match self.next() {
            Some(Tokens::Colon) => (),
            _ => todo!("type inference not added"),
        };

        let vartype = self.parse_type()?;
        match self.next() {
            Some(Tokens::Equals) => (),
            _ => return Err(ParserError::MissingEqualSign),
        }

        let exp = self.parse_expression(0)?;

        //consume semicolon
        self.next();

        Ok(Statement::Let {
            name: vname,
            value: exp,
            var_type: Some(vartype),
        })
    }
    pub(crate) fn parse_statement(&mut self) -> Result<Statement<'src>> {
        match self.peek() {
            Some(Tokens::Let) => self.parse_declaration(),
            _ => todo!("other statements"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let statement = parser.parse_statement().unwrap();
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
}

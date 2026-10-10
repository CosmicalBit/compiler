use crate::lexer::classify::Tokens;
use crate::parser::ast::{Expressions, Parser, ParserError, Result, Statement};

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Block<'src> {
    statements: Vec<Statement<'src>>,
    tail: Option<Box<Expressions<'src>>>,
}

impl<'src> Parser<'src> {
    pub(super) fn parse_block(&mut self) -> Result<'src, Block<'src>> {
        self.expect(Tokens::LeftBrace, ParserError::MissingLeftBrace)?;

        let mut statements = Vec::new();
        let mut tail = None;

        while !matches!(self.peek(), Some(Tokens::RightBrace) | None) {
            // Declarations remain statements.
            if matches!(self.peek(), Some(Tokens::Let)) {
                statements.push(self.parse_declaration()?);
                continue;
            }

            // Everything else starts as an expression.
            let expr = self.parse_expression(0)?;

            if matches!(self.peek(), Some(Tokens::SemiColon)) {
                // Replace ForwardSlash with your semicolon token.
                self.next();

                statements.push(Statement::Expression(expr));
            } else if matches!(self.peek(), Some(Tokens::RightBrace)) {
                tail = Some(Box::new(expr));
                break;
            } else {
                return Err(ParserError::MissingSemicolon);
            }
        }

        self.expect(Tokens::RightBrace, ParserError::MissingRightBrace)?;

        Ok(Block { statements, tail })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_block_with_statement() {
        let tokens = [
            Tokens::LeftBrace,
            Tokens::Let,
            Tokens::Other("answer"),
            Tokens::Colon,
            Tokens::Other("i64"),
            Tokens::Equals,
            Tokens::Integer(42),
            Tokens::SemiColon,
            Tokens::RightBrace,
        ];
        let mut parser = Parser::new(&tokens);

        let block = parser.parse_block().unwrap();
        assert_eq!(block.statements.len(), 1);
        assert!(matches!(
            block.statements[0],
            Statement::Let { name: "answer", .. }
        ));
    }

    #[test]
    fn reports_missing_closing_brace() {
        let tokens = [Tokens::LeftBrace];
        let mut parser = Parser::new(&tokens);

        assert!(matches!(
            parser.parse_block(),
            Err(ParserError::MissingRightBrace)
        ));
    }
}

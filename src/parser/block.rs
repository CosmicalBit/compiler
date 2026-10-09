use crate::lexer::classify::Tokens;
use crate::parser::ast::{Parser, ParserError, Result, Statement};

pub(crate) struct Block<'src> {
    statements: Vec<Statement<'src>>,
}

impl<'src> Parser<'src> {
    pub(crate) fn parse_block(&mut self) -> Result<Block<'src>> {
        self.expect(Tokens::LeftBrace, ParserError::MissingLeftBracket)?;
        self.next();

        let mut statements = Vec::new();

        while !matches!(self.peek(), Some(Tokens::RightBrace) | None) {
            statements.push(self.parse_statement()?);
        }

        match self.next() {
            Some(Tokens::RightBrace) => Ok(Block { statements }),
            _ => Err(ParserError::MissingRightBracket),
        }
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
        assert!(matches!(block.statements[0], Statement::Let { name: "answer", .. }));
    }

    #[test]
    fn reports_missing_closing_brace() {
        let tokens = [Tokens::LeftBrace];
        let mut parser = Parser::new(&tokens);

        assert!(matches!(
            parser.parse_block(),
            Err(ParserError::MissingRightBracket)
        ));
    }
}

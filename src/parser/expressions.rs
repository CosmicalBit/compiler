use crate::lexer::classify::Tokens;
use crate::parser::ast::{BinaryOp, Expressions, Parser, ParserError, Result};

impl<'src> Parser<'src> {
    fn parse_atom(&mut self) -> Result<Expressions<'src>> {
        match self.next() {
            Some(Tokens::Integer(value)) => Ok(Expressions::Integer(*value)),
            Some(Tokens::Other(name)) => Ok(Expressions::Identifier(name)),
            Some(Tokens::LeftParen) => {
                let expr = self.parse_expression(0)?;
                match self.next() {
                    Some(Tokens::RightParen) => Ok(expr),
                    _ => Err(ParserError::MissingRightParen),
                }
            }
            _ => Err(ParserError::ExpectedExpression),
        }
    }
    pub(crate) fn parse_expression(&mut self, min_bp: u8) -> Result<Expressions<'src>> {
        let mut lhs = self.parse_atom()?;

        loop {
            let (op, left_bp, right_bp) = match self.peek() {
                Some(Tokens::Plus) => (BinaryOp::Add, 1, 2),
                Some(Tokens::Minus) => (BinaryOp::Sub, 1, 2),
                Some(Tokens::Star) => (BinaryOp::Mul, 3, 4),
                Some(Tokens::FowardSlash) => (BinaryOp::Divide, 3, 4),
                _ => break,
            };
            if left_bp < min_bp {
                break;
            }

            self.next();
  
            let rhs = self.parse_expression(right_bp)?;

            lhs = Expressions::Binary {
                op,
                left: Box::new(lhs),
                right: Box::new(rhs),
            };
        }
        Ok(lhs)
    }
}

mod test {
    #[test]
    fn parses_operator_precedence_and_associativity() {
        use super::*;

        let expr = vec![
            Tokens::Integer(4),
            Tokens::Plus,
            Tokens::Integer(2),
            Tokens::FowardSlash,
            Tokens::Integer(2),
            Tokens::Minus,
            Tokens::Integer(40),
        ];
        let mut parser = Parser::new(&expr);
        let parsed = parser.parse_expression(0).unwrap();
        assert_eq!(
            parsed,
            Expressions::Binary {
                op: BinaryOp::Sub,
                left: Box::new(Expressions::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expressions::Integer(4)),
                    right: Box::new(Expressions::Binary {
                        op: BinaryOp::Divide,
                        left: Box::new(Expressions::Integer(2)),
                        right: Box::new(Expressions::Integer(2)),
                    }),
                }),
                right: Box::new(Expressions::Integer(40)),
            }
        );
    }

    #[test]
    fn parentheses_override_precedence_and_identifiers_parse() {
        use super::*;
        let expr = vec![
            Tokens::LeftParen,
            Tokens::Other("value"),
            Tokens::Plus,
            Tokens::Integer(3),
            Tokens::RightParen,
            Tokens::Star,
            Tokens::Integer(2),
        ];
        let mut parser = Parser::new(&expr);
        let parsed = parser.parse_expression(0).unwrap();
        assert_eq!(
            parsed,
            Expressions::Binary {
                op: BinaryOp::Mul,
                left: Box::new(Expressions::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expressions::Identifier("value")),
                    right: Box::new(Expressions::Integer(3)),
                }),
                right: Box::new(Expressions::Integer(2)),
            }
        );
    }

    #[test]
    fn reports_missing_closing_parenthesis() {
        use super::*;
        let expr = vec![Tokens::LeftParen, Tokens::Integer(1)];
        let mut parser = Parser::new(&expr);
        assert_eq!(
            parser.parse_expression(0),
            Err(ParserError::MissingRightParen)
        );
    }

    #[test]
    fn reports_missing_expression_after_operator() {
        use super::*;
        let expr = vec![Tokens::Integer(1), Tokens::Plus];
        let mut parser = Parser::new(&expr);
        assert_eq!(
            parser.parse_expression(0),
            Err(ParserError::ExpectedExpression)
        );
    }
}

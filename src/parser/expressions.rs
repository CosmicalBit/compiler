use crate::lexer::classify::Tokens;
use crate::parser::ast::{BinaryOp, Expressions, MatchArms, Parser, ParserError, Pattern, Result};
impl<'src> Parser<'src> {
    fn parse_atom(&mut self) -> Result<Expressions<'src>> {
        if matches!(self.peek(), Some(Tokens::If)) {
            return self.parse_if();
        }
        if matches!(self.peek(), Some(Tokens::Match)) {
            return self.parse_match();
        }

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
            Some(Tokens::If) => self.parse_if(),
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

    fn parse_if(&mut self) -> Result<Expressions<'src>> {
        self.expect(Tokens::If, ParserError::MissingIfStatement)?;
        self.next();

        let condition = self.parse_expression(0)?;
        let then_block = self.parse_block()?;

        let else_branch = if matches!(self.peek(), Some(Tokens::Else)) {
            self.next();

            if matches!(self.peek(), Some(Tokens::If)) {
                Some(Box::new(self.parse_if()?))
            } else {
                Some(Box::new(Expressions::Block(self.parse_block()?)))
            }
        } else {
            None
        };

        Ok(Expressions::If {
            condition: Box::new(condition),
            then_block,
            else_branch,
        })
    }
    fn parse_match(&mut self) -> Result<Expressions<'src>> {
        self.expect(Tokens::Match, ParserError::MissingMatchStatemt)?;
        self.next();
        let scruntime = self.parse_expression(0)?;

        self.expect(Tokens::LeftBracket, ParserError::MissingLeftBracket)?;
        self.next();

        let mut arms = Vec::new();
        while !matches!(self.peek(), Some(Tokens::RightBracket) | None) {
            let pattern = self.parse_pattern()?;
            self.expect(Tokens::FatArrow, ParserError::MissingFatArrow)?;
            self.next();

            let body = self.parse_expression(0)?;

            arms.push(MatchArms {
                body: Box::new(body),
                pattern,
            });

            if matches!(self.peek(), Some(Tokens::Comma)) {
                self.next();
            } else if !matches!(self.peek(), Some(Tokens::RightBracket)) {
                return Err(ParserError::ExpectedComma);
            }
        }
        self.expect(Tokens::RightBracket, ParserError::MissingRightBracket)?;
        Ok(Expressions::Match {
            scruntinne: Box::new(scruntime),
            arms,
        })
    }

    fn parse_pattern(&mut self) -> Result<Pattern<'src>> {
        match self.next() {
            Some(Tokens::Integer(value)) => Ok(Pattern::Integer(*value)),
            Some(Tokens::Other("_")) => Ok(Pattern::Wildcard),
            _ => Err(ParserError::ExpectedPattern),
        }
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

    #[test]
    fn parses_match_with_integer_and_wildcard_arms() {
        use super::*;
        let tokens = vec![
            Tokens::Match,
            Tokens::Other("value"),
            Tokens::LeftBracket,
            Tokens::Integer(1),
            Tokens::FatArrow,
            Tokens::Other("one"),
            Tokens::Comma,
            Tokens::Other("_"),
            Tokens::FatArrow,
            Tokens::Integer(0),
            Tokens::RightBracket,
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            parser.parse_expression(0),
            Ok(Expressions::Match {
                scruntinne: Box::new(Expressions::Identifier("value")),
                arms: vec![
                    MatchArms {
                        pattern: Pattern::Integer(1),
                        body: Box::new(Expressions::Identifier("one")),
                    },
                    MatchArms {
                        pattern: Pattern::Wildcard,
                        body: Box::new(Expressions::Integer(0)),
                    },
                ],
            })
        );
    }

    #[test]
    fn reports_missing_match_arm_separator() {
        use super::*;
        let tokens = vec![
            Tokens::Match,
            Tokens::Integer(2),
            Tokens::LeftBracket,
            Tokens::Integer(1),
            Tokens::FatArrow,
            Tokens::Integer(10),
            Tokens::Integer(2),
            Tokens::FatArrow,
            Tokens::Integer(20),
            Tokens::RightBracket,
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(parser.parse_expression(0), Err(ParserError::ExpectedComma));
    }

    #[test]
    fn reports_missing_match_arm_arrow() {
        use super::*;
        let tokens = vec![
            Tokens::Match,
            Tokens::Integer(2),
            Tokens::LeftBracket,
            Tokens::Integer(1),
            Tokens::Integer(10),
            Tokens::RightBracket,
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(parser.parse_expression(0), Err(ParserError::MissingFatArrow));
    }
}

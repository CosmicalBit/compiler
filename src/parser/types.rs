use crate::lexer::classify::Tokens;
use crate::parser::ast::{Parser, ParserError, Result, Type};

impl<'src> Parser<'src> {
    pub(super) fn parse_type(&mut self) -> Result<Type<'src>> {
        match self.next() {
            Some(Tokens::Other(typename)) => Ok(Type::Named(typename)),
            Some(Tokens::Ampersant) => {
                let mutable = matches!(self.peek(), Some(Tokens::Mut));

                if mutable {
                    self.next();
                }
                let inner = self.parse_type()?;

                Ok(Type::Reference {
                    mutable,
                    inner: Box::new(inner),
                })
            }
            Some(Tokens::Star) => {
                let mutable = matches!(self.peek(), Some(Tokens::Mut));

                if mutable {
                    self.next();
                }
                let inner = self.parse_type()?;

                Ok(Type::Pointer {
                    mutable,
                    inner: Box::new(inner),
                })
            }
            _ => Err(ParserError::MissingType),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_named_type() {
        let tokens = [Tokens::Other("u32")];
        let mut parser = Parser::new(&tokens);

        assert_eq!(parser.parse_type(), Ok(Type::Named("u32")));
        assert_eq!(parser.pos, 1);
    }

    #[test]
    fn parses_immutable_and_mutable_references() {
        let immutable = [Tokens::Ampersant, Tokens::Other("Item")];
        let mutable = [Tokens::Ampersant, Tokens::Mut, Tokens::Other("Item")];
        let mut parser = Parser::new(&immutable);
        assert_eq!(
            parser.parse_type(),
            Ok(Type::Reference {
                mutable: false,
                inner: Box::new(Type::Named("Item")),
            })
        );

        let mut parser = Parser::new(&mutable);
        assert_eq!(
            parser.parse_type(),
            Ok(Type::Reference {
                mutable: true,
                inner: Box::new(Type::Named("Item")),
            })
        );
    }

    #[test]
    fn parses_immutable_and_mutable_pointers() {
        let immutable = [Tokens::Star, Tokens::Other("Item")];
        let mutable = [Tokens::Star, Tokens::Mut, Tokens::Other("Item")];
        let mut parser = Parser::new(&immutable);
        assert_eq!(
            parser.parse_type(),
            Ok(Type::Pointer {
                mutable: false,
                inner: Box::new(Type::Named("Item")),
            })
        );

        let mut parser = Parser::new(&mutable);
        assert_eq!(
            parser.parse_type(),
            Ok(Type::Pointer {
                mutable: true,
                inner: Box::new(Type::Named("Item")),
            })
        );
    }

    #[test]
    fn parses_nested_type_modifiers() {
        let tokens = [
            Tokens::Ampersant,
            Tokens::Mut,
            Tokens::Star,
            Tokens::Other("Item"),
        ];
        let mut parser = Parser::new(&tokens);

        assert_eq!(
            parser.parse_type(),
            Ok(Type::Reference {
                mutable: true,
                inner: Box::new(Type::Pointer {
                    mutable: false,
                    inner: Box::new(Type::Named("Item")),
                }),
            })
        );
        assert_eq!(parser.pos, tokens.len());
    }

    #[test]
    fn reports_missing_type_after_modifier_or_at_end() {
        let empty = [];
        let incomplete_reference = [Tokens::Ampersant, Tokens::Mut];
        let incomplete_pointer = [Tokens::Star];

        for tokens in [&empty[..], &incomplete_reference, &incomplete_pointer] {
            let mut parser = Parser::new(tokens);
            assert_eq!(parser.parse_type(), Err(ParserError::MissingType));
        }
    }
}

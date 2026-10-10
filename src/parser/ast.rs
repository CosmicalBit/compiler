use crate::lexer::classify::Tokens;
use crate::parser::block::Block;

use crate::parser::generic_params::GenericParam;
pub(super) type Result<'src, T> = std::result::Result<T, ParserError<'src>>;

pub(crate) struct Parser<'src> {
    pub(crate) tokens: &'src [Tokens<'src>],
    pub(crate) pos: usize,
}

impl<'src> Parser<'src> {
    pub(crate) fn new(tokens: &'src [Tokens<'src>]) -> Self {
        Self { pos: 0, tokens }
    }
    pub(super) fn peek(&self) -> Option<&Tokens<'src>> {
        self.tokens.get(self.pos)
    }
    pub(super) fn next(&mut self) -> Option<&'src Tokens<'src>> {
        let token = self.tokens.get(self.pos)?;
        self.pos += 1;
        Some(token)
    }
    /// Checks for and consumes the expected token.
    pub(super) fn expect(
        &mut self,
        expected: Tokens<'src>,
        err: ParserError<'src>,
    ) -> Result<'src, ()> {
        if self.peek() == Some(&expected) {
            self.next();
            Ok(())
        } else {
            Err(err)
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Expressions<'src> {
    Integer(i64),
    Identifier(&'src str),
    Call {
        calle: Box<Expressions<'src>>,
        args: Vec<Expressions<'src>>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expressions<'src>>,
        right: Box<Expressions<'src>>,
    },
    If {
        condition: Box<Expressions<'src>>,
        then_block: Block<'src>,
        else_branch: Option<Box<Expressions<'src>>>,
    },
    FieldAccess {
        object: Box<Expressions<'src>>,
        field: &'src str,
    },

    Assignement {
        target: Box<Expressions<'src>>,
        value: Box<Expressions<'src>>,
    },

    Match {
        scruntinne: Box<Expressions<'src>>,
        arms: Vec<MatchArms<'src>>,
    },
    Cast {
        expr: Box<Expressions<'src>>,
        ty: Type<'src>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expressions<'src>>,
    },
    Block(Block<'src>),
}

#[derive(PartialEq, Eq, Debug)]
pub(super) enum UnaryOp {
    Deref,
    Neg,
    Not,
}
#[derive(PartialEq, Eq, Debug)]
pub(super) struct MatchArms<'src> {
    pub(super) pattern: Pattern<'src>,
    pub(super) body: Box<Expressions<'src>>,
}
#[derive(Debug, PartialEq, Eq)]
pub(super) enum BinaryOp {
    Add,
    Sub,
    Mul,
    Divide,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ParserError<'src> {
    EmptyGenericParams,
    EOF,
    MissingLessSymbol,
    MissingGreaterSymbol,
    ExpectedType,
    MissingLifetimeName,
    UnexpectedTokenInGenerics(Tokens<'src>),
    MissingStructName,
    MissingStructDefinition,
    ExpectedField,
    ExpectedComma,
    ExpectedPattern,
    MissingMatchStatemt,
    MissingFatArrow,
    MissingSemicolon,
    MissingIfStatement,
    MissingFnName,
    MissingFn,
    MissingParameterName,
    MissingParameterType,
    MissingCommaOrRightParen,
    MissingArrow,
    MissingFunctionBody,
    MissingLeftParen,
    MissingLeftBrace,
    MissingLeftBracket,
    MissingLetStatement,
    MissingRightBracket,
    MissingRightBrace,
    MissingEqualSign,
    MissingVarName,
    ExpectedExpression,
    MissingRightParen,
    ExpectedLetDeclaration,
    MissingType,
}
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Statement<'src> {
    Let {
        name: &'src str,
        value: Expressions<'src>,
        var_type: Option<Type<'src>>,
    },

    Expression(Expressions<'src>),
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Parameter<'src> {
    pub(super) name: &'src str,
    pub(super) var_type: Type<'src>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Function<'src> {
    pub(super) name: &'src str,
    pub(super) generic_parameter: Vec<GenericParam<'src>>,
    pub(super) parameters: Vec<Parameter<'src>>,
    pub(super) return_type: Type<'src>,
    pub(super) body: crate::parser::block::Block<'src>,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Type<'src> {
    Named(&'src str),
    Reference {
        mutable: bool,
        inner: Box<Type<'src>>,
    },
    Pointer {
        mutable: bool,
        inner: Box<Type<'src>>,
    },
}

#[derive(PartialEq, Eq, Debug)]
pub(super) enum Pattern<'src> {
    Wildcard,
    Integer(i64),
    Constant(Box<Expressions<'src>>),
}

#[derive(PartialEq, Eq, Debug)]
pub(crate) struct Struct<'src> {
    pub(super) generic_parameters: Vec<GenericParam<'src>>,
    pub(super) name: &'src str,
    pub(super) fields: Vec<Parameter<'src>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expect_consumes_matching_token_only() {
        let tokens = [Tokens::Let, Tokens::Other("next")];
        let mut parser = Parser::new(&tokens);

        assert_eq!(
            parser.expect(Tokens::Let, ParserError::MissingLetStatement),
            Ok(())
        );
        assert_eq!(parser.peek(), Some(&Tokens::Other("next")));
    }

    #[test]
    fn expect_leaves_unmatched_token_unconsumed() {
        let tokens = [Tokens::Other("next")];
        let mut parser = Parser::new(&tokens);

        assert_eq!(
            parser.expect(Tokens::Let, ParserError::MissingLetStatement),
            Err(ParserError::MissingLetStatement)
        );
        assert_eq!(parser.peek(), Some(&Tokens::Other("next")));
    }
}

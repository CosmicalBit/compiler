use crate::lexer::classify::Tokens;

pub(crate) type Result<T> = std::result::Result<T, ParserError>;

pub(crate) struct Parser<'src> {
    pub(crate) tokens: &'src [Tokens<'src>],
    pub(crate) pos: usize,
}
impl<'src> Parser<'src> {
    pub(crate) fn new(tokens: &'src [Tokens]) -> Self {
        Self { pos: 0, tokens }
    }
    pub(crate) fn peek(&self) -> Option<&Tokens<'src>> {
        self.tokens.get(self.pos)
    }
    pub(crate) fn next(&mut self) -> Option<&'src Tokens<'src>> {
        let token = self.tokens.get(self.pos)?;
        self.pos += 1;
        Some(token)
    }
    pub(crate) fn expect(&self, expected: Tokens<'src>, err: ParserError) -> Result<()> {
        if self.peek() == Some(&expected) {
            Ok(())
        } else {
            Err(err)
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Expressions<'src> {
    Integer(i64),
    Identifier(&'src str),
    Binary {
        op: BinaryOp,
        left: Box<Expressions<'src>>,
        right: Box<Expressions<'src>>,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum BinaryOp {
    Add,
    Sub,
    Mul,
    Divide,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ParserError {
    MissingLeftBracket,
    MissingLetStatement,
    MissingRightBracket,
    MissingEqualSign,
    MissingVarName,
    ExpectedExpression,
    MissingRightParen,
    ExpectedLetDeclaration,
    MissingVarType,
}
pub(crate) enum Statement<'src> {
    Let {
        name: &'src str,
        value: Expressions<'src>,
        var_type: Option<Type<'src>>,
    },
    Expression(Expressions<'src>),
}

pub(crate) enum Type<'src> {
    Named(&'src str),
}

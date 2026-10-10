use crate::lexer::classify::Tokens;
use crate::parser::block::Block;

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
    ///[`expect`] doesnt call self.next() aka doest consume token
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

    Match {
        scruntinne: Box<Expressions<'src>>,
        arms: Vec<MatchArms<'src>>,
    },
    Cast {
        expr: Box<Expressions<'src>>,
        ty: Type<'src>,
    },
    Block(Block<'src>),
}

#[derive(PartialEq, Eq, Debug)]
pub(crate) struct MatchArms<'src> {
    pub(crate) pattern: Pattern<'src>,
    pub(crate) body: Box<Expressions<'src>>,
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
pub(crate) enum Statement<'src> {
    Let {
        name: &'src str,
        value: Expressions<'src>,
        var_type: Option<Type<'src>>,
    },
    Expression(Expressions<'src>),
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Parameter<'src> {
    pub(crate) name: &'src str,
    pub(crate) var_type: Type<'src>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Function<'src> {
    pub(crate) name: &'src str,
    pub(crate) parameters: Vec<Parameter<'src>>,
    pub(crate) return_type: Type<'src>,
    pub(crate) body: crate::parser::block::Block<'src>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Type<'src> {
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
pub(crate) enum Pattern<'src> {
    Wildcard,
    Integer(i64),
    Constant(Box<Expressions<'src>>),
}

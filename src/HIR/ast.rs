//! Resolved, parser-independent representation of a program.
//!
//! Names are replaced with IDs during lowering. Integer literals remain `i64`
//! until type checking chooses their final numeric type.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct LocalId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ItemId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Identifies a field name across structs; equal names share an ID.
pub(crate) struct FieldId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct TypeParamId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct LifetimeId(pub u32);

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Program {
    pub(crate) items: Vec<Item>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Item {
    Function(Function),
    Struct(Struct),
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Function {
    pub(crate) id: ItemId,
    pub(crate) generics: Vec<GenericParam>,
    pub(crate) parameters: Vec<Parameter>,
    pub(crate) return_type: Type,
    pub(crate) body: Block,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Struct {
    pub(crate) id: ItemId,
    pub(crate) generics: Vec<GenericParam>,
    pub(crate) fields: Vec<Field>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum GenericParam {
    Type(TypeParamId),
    Lifetime(LifetimeId),
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Parameter {
    pub(crate) id: LocalId,
    pub(crate) ty: Type,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Field {
    pub(crate) id: FieldId,
    pub(crate) ty: Type,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Block {
    pub(crate) statements: Vec<Statement>,
    pub(crate) tail: Option<Box<Expression>>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Statement {
    Let {
        id: LocalId,
        ty: Option<Type>,
        value: Expression,
    },
    Expression(Expression),
}

#[derive(Debug, PartialEq)]
pub(crate) enum Expression {
    Integer(i32),
    Float(f64),
    Local(LocalId),
    Item(ItemId),
    Call {
        callee: Box<Expression>,
        generic_args: Vec<Type>,
        args: Vec<Expression>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expression>,
    },
    If {
        condition: Box<Expression>,
        then_block: Block,
        else_branch: Option<Box<Expression>>,
    },
    FieldAccess {
        object: Box<Expression>,
        field: FieldId,
    },
    Assignment {
        target: Box<Expression>,
        value: Box<Expression>,
    },
    Match {
        scrutinee: Box<Expression>,
        arms: Vec<MatchArm>,
    },
    Cast {
        expr: Box<Expression>,
        ty: Type,
    },
    Block(Block),
}

impl Eq for Expression {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BinaryOp {
    Add,
    Sub,
    Mul,
    Eq,
    Divide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UnaryOp {
    Deref,
    Neg,
    Not,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct MatchArm {
    pub(crate) pattern: Pattern,
    pub(crate) body: Expression,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Pattern {
    Wildcard,
    Integer(i64),
    Constant(Box<Expression>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Type {
    TypeParameter(TypeParamId),
    Primitive(PrimitiveType),
    Named { id: ItemId, args: Vec<Type> },
    Reference { mutable: bool, inner: Box<Type> },
    Pointer { mutable: bool, inner: Box<Type> },
}
impl Type {
    pub(crate) fn is_numerical(&self) -> bool {
        match self {
            Type::Primitive(PrimitiveType::Bool) => false,
            Type::Primitive(_) => true,
            _ => false,
        }
    }
    pub(crate) fn supports_wq(ty: &Type) -> bool {
        match ty {
            Type::Primitive(_) => true,
            Type::Named { args, .. } => {
                for arg in args {
                    if !Type::supports_wq(arg) {
                        return false;
                    }
                }
                true
            }
            Type::Reference { inner, .. } | Type::Pointer { inner, .. } => Type::supports_wq(inner),
            Type::TypeParameter(_) => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PrimitiveType {
    Bool,
    I8,
    I16,
    I32,
    I64,
    F64,
    ISize,
    U8,
    U16,
    U32,
    U64,
    USize,
}
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum HirError {
    DuplicatedName(String),
    UnknownName(String),
    UnknownType(String),
    UnknownField(String),
    InvalidGenericType(String),
    TooManyDefinitions,
}

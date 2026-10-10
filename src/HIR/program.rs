//! Lower the parser AST into a name-resolved HIR program.

use crate::HIR::ast::{self as hir, HirError};
use crate::parser::ast::{
    BinaryOp as AB, Expressions as AE, Pattern as AP, Statement as AS, Type as AT, UnaryOp as AU,
};
use crate::parser::block::Block as AstBlock;
use crate::parser::generic_params::GenericParam as AG;
use crate::parser::program::Item as AI;
use std::collections::HashMap;

type Result<T> = std::result::Result<T, HirError>;

pub(crate) struct Hir<'src> {
    items: Vec<AI<'src>>,
}

impl<'src> Hir<'src> {
    pub(crate) fn new(items: &mut Vec<AI<'src>>) -> Self {
        Self {
            items: std::mem::take(items),
        }
    }
    pub(crate) fn lower(&self) -> Result<hir::Program> {
        lower_program(&self.items)
    }
}

pub(crate) fn lower_program(items: &[AI<'_>]) -> Result<hir::Program> {
    let item_ids = collect_items(items)?;
    let fields = collect_fields(items)?;
    let ctx = Context {
        item_ids: &item_ids,
        fields: &fields,
    };
    let mut output = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        let item_id = hir::ItemId(to_id(index)?);
        output.push(match item {
            AI::Function(f) => {
                let (generics, types) = lower_generics(&f.generic_parameter)?;
                let mut locals = Locals::default();
                let parameters = f
                    .parameters
                    .iter()
                    .map(|p| {
                        let ty = ctx.lower_type(&p.var_type, &types)?;
                        let id = locals.declare(p.name)?;
                        Ok(hir::Parameter { id, ty })
                    })
                    .collect::<Result<Vec<_>>>()?;
                let return_type = ctx.lower_type(&f.return_type, &types)?;
                let body = ctx.lower_block(&f.body, &types, &mut locals)?;
                hir::Item::Function(hir::Function {
                    id: item_id,
                    generics,
                    parameters,
                    return_type,
                    body,
                })
            }
            AI::Struct(s) => {
                let (generics, types) = lower_generics(&s.generic_parameters)?;
                let mut seen = HashMap::new();
                let fields = s
                    .fields
                    .iter()
                    .map(|field| {
                        if seen.insert(field.name, ()).is_some() {
                            return Err(HirError::DuplicatedName(field.name.into()));
                        }
                        let ty = ctx.lower_type(&field.var_type, &types)?;
                        let id = *fields.get(field.name).expect("collected field");
                        Ok(hir::Field { id, ty })
                    })
                    .collect::<Result<Vec<_>>>()?;
                hir::Item::Struct(hir::Struct {
                    id: item_id,
                    generics,
                    fields,
                })
            }
        });
    }
    Ok(hir::Program { items: output })
}

fn to_id(index: usize) -> Result<u32> {
    u32::try_from(index).map_err(|_| HirError::TooManyDefinitions)
}

fn collect_items<'src>(items: &[AI<'src>]) -> Result<HashMap<&'src str, hir::ItemId>> {
    let mut names = HashMap::new();
    for (index, item) in items.iter().enumerate() {
        let name = match item {
            AI::Function(f) => f.name,
            AI::Struct(s) => s.name,
        };
        if names.insert(name, hir::ItemId(to_id(index)?)).is_some() {
            return Err(HirError::DuplicatedName(name.into()));
        }
    }
    Ok(names)
}

// A field ID represents a field name. Different structs can use the same name;
// the type checker will select the struct from the object's type.
fn collect_fields<'src>(items: &[AI<'src>]) -> Result<HashMap<&'src str, hir::FieldId>> {
    let mut fields = HashMap::new();
    for item in items {
        if let AI::Struct(s) = item {
            for field in &s.fields {
                if !fields.contains_key(field.name) {
                    let field_id = hir::FieldId(to_id(fields.len())?);
                    fields.insert(field.name, field_id);
                }
            }
        }
    }
    Ok(fields)
}

fn lower_generics<'src>(
    params: &[AG<'src>],
) -> Result<(Vec<hir::GenericParam>, HashMap<&'src str, hir::TypeParamId>)> {
    let mut output = Vec::with_capacity(params.len());
    let mut types = HashMap::new();
    let mut lifetimes = HashMap::new();
    for param in params {
        match param {
            AG::Type(name) => {
                let id = hir::TypeParamId(to_id(types.len())?);
                if types.insert(*name, id).is_some() {
                    return Err(HirError::DuplicatedName((*name).into()));
                }
                output.push(hir::GenericParam::Type(id));
            }
            AG::Lifetime(name) => {
                let id = hir::LifetimeId(to_id(lifetimes.len())?);
                if lifetimes.insert(*name, id).is_some() {
                    return Err(HirError::DuplicatedName((*name).into()));
                }
                output.push(hir::GenericParam::Lifetime(id));
            }
        }
    }
    Ok((output, types))
}

#[derive(Default)]
struct Locals<'src> {
    scopes: Vec<HashMap<&'src str, hir::LocalId>>,
    next: usize,
}

impl<'src> Locals<'src> {
    fn declare(&mut self, name: &'src str) -> Result<hir::LocalId> {
        if self.scopes.is_empty() {
            self.scopes.push(HashMap::new());
        }
        let scope = self.scopes.last_mut().unwrap();
        if scope.contains_key(name) {
            return Err(HirError::DuplicatedName(name.into()));
        }
        let id = hir::LocalId(to_id(self.next)?);
        self.next += 1;
        scope.insert(name, id);
        Ok(id)
    }
    fn lookup(&self, name: &str) -> Option<hir::LocalId> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
    }
}

struct Context<'a, 'src> {
    item_ids: &'a HashMap<&'src str, hir::ItemId>,
    fields: &'a HashMap<&'src str, hir::FieldId>,
}

impl Context<'_, '_> {
    fn lower_type(
        &self,
        ty: &AT<'_>,
        params: &HashMap<&str, hir::TypeParamId>,
    ) -> Result<hir::Type> {
        Ok(match ty {
            AT::Named(name) => {
                if let Some(p) = primitive(name) {
                    hir::Type::Primitive(p)
                } else if let Some(id) = params.get(name) {
                    hir::Type::Param(*id)
                } else if let Some(id) = self.item_ids.get(name) {
                    hir::Type::Named(*id)
                } else {
                    return Err(HirError::UnknownType((*name).into()));
                }
            }
            AT::Generic { typename, args } => {
                if primitive(typename).is_some() || params.contains_key(typename) {
                    return Err(HirError::InvalidGenericType((*typename).into()));
                }
                let item = self
                    .item_ids
                    .get(typename)
                    .ok_or_else(|| HirError::UnknownType((*typename).into()))?;
                hir::Type::Generic {
                    item: *item,
                    args: args
                        .iter()
                        .map(|arg| self.lower_type(arg, params))
                        .collect::<Result<_>>()?,
                }
            }
            AT::Reference { mutable, inner } => hir::Type::Reference {
                mutable: *mutable,
                inner: Box::new(self.lower_type(inner, params)?),
            },
            AT::Pointer { mutable, inner } => hir::Type::Pointer {
                mutable: *mutable,
                inner: Box::new(self.lower_type(inner, params)?),
            },
        })
    }

    fn lower_block<'src>(
        &self,
        block: &AstBlock<'src>,
        params: &HashMap<&str, hir::TypeParamId>,
        locals: &mut Locals<'src>,
    ) -> Result<hir::Block> {
        locals.scopes.push(HashMap::new());
        let result = (|| {
            let mut statements = Vec::with_capacity(block.statements.len());
            for statement in &block.statements {
                statements.push(match statement {
                    AS::Let {
                        name,
                        var_type,
                        value,
                    } => {
                        let ty = var_type
                            .as_ref()
                            .map(|ty| self.lower_type(ty, params))
                            .transpose()?;
                        let value = self.lower_expression(value, params, locals)?;
                        let id = locals.declare(name)?;
                        hir::Statement::Let { id, ty, value }
                    }
                    AS::Expression(expr) => {
                        hir::Statement::Expression(self.lower_expression(expr, params, locals)?)
                    }
                });
            }
            let tail = block
                .tail
                .as_ref()
                .map(|expr| self.lower_expression(expr, params, locals).map(Box::new))
                .transpose()?;
            Ok(hir::Block { statements, tail })
        })();
        locals.scopes.pop();
        result
    }

    fn lower_expression<'src>(
        &self,
        expr: &AE<'src>,
        params: &HashMap<&str, hir::TypeParamId>,
        locals: &mut Locals<'src>,
    ) -> Result<hir::Expression> {
        Ok(match expr {
            AE::Integer(value) => hir::Expression::Integer(*value),
            AE::Identifier(name) => {
                if let Some(id) = locals.lookup(name) {
                    hir::Expression::Local(id)
                } else if let Some(id) = self.item_ids.get(name) {
                    hir::Expression::Item(*id)
                } else {
                    return Err(HirError::UnknownName((*name).into()));
                }
            }
            AE::Call {
                calle,
                generic_args,
                args,
            } => hir::Expression::Call {
                callee: Box::new(self.lower_expression(calle, params, locals)?),
                generic_args: generic_args
                    .iter()
                    .map(|ty| self.lower_type(ty, params))
                    .collect::<Result<_>>()?,
                args: args
                    .iter()
                    .map(|arg| self.lower_expression(arg, params, locals))
                    .collect::<Result<_>>()?,
            },
            AE::Binary { op, left, right } => hir::Expression::Binary {
                op: match op {
                    AB::Add => hir::BinaryOp::Add,
                    AB::Sub => hir::BinaryOp::Sub,
                    AB::Mul => hir::BinaryOp::Mul,
                    AB::Divide => hir::BinaryOp::Divide,
                },
                left: Box::new(self.lower_expression(left, params, locals)?),
                right: Box::new(self.lower_expression(right, params, locals)?),
            },
            AE::Unary { op, expr } => hir::Expression::Unary {
                op: match op {
                    AU::Deref => hir::UnaryOp::Deref,
                    AU::Neg => hir::UnaryOp::Neg,
                    AU::Not => hir::UnaryOp::Not,
                },
                expr: Box::new(self.lower_expression(expr, params, locals)?),
            },
            AE::If {
                condition,
                then_block,
                else_branch,
            } => hir::Expression::If {
                condition: Box::new(self.lower_expression(condition, params, locals)?),
                then_block: self.lower_block(then_block, params, locals)?,
                else_branch: else_branch
                    .as_ref()
                    .map(|expr| self.lower_expression(expr, params, locals).map(Box::new))
                    .transpose()?,
            },
            AE::FieldAccess { object, field } => {
                let id = self
                    .fields
                    .get(field)
                    .ok_or_else(|| HirError::UnknownField((*field).into()))?;
                hir::Expression::FieldAccess {
                    object: Box::new(self.lower_expression(object, params, locals)?),
                    field: *id,
                }
            }
            AE::Assignement { target, value } => hir::Expression::Assignment {
                target: Box::new(self.lower_expression(target, params, locals)?),
                value: Box::new(self.lower_expression(value, params, locals)?),
            },
            AE::Match { scruntinne, arms } => hir::Expression::Match {
                scrutinee: Box::new(self.lower_expression(scruntinne, params, locals)?),
                arms: arms
                    .iter()
                    .map(|arm| {
                        Ok(hir::MatchArm {
                            pattern: match &arm.pattern {
                                AP::Wildcard => hir::Pattern::Wildcard,
                                AP::Integer(value) => hir::Pattern::Integer(*value),
                                AP::Constant(expr) => hir::Pattern::Constant(Box::new(
                                    self.lower_expression(expr, params, locals)?,
                                )),
                            },
                            body: self.lower_expression(&arm.body, params, locals)?,
                        })
                    })
                    .collect::<Result<_>>()?,
            },
            AE::Cast { expr, ty } => hir::Expression::Cast {
                expr: Box::new(self.lower_expression(expr, params, locals)?),
                ty: self.lower_type(ty, params)?,
            },
            AE::Block(block) => hir::Expression::Block(self.lower_block(block, params, locals)?),
        })
    }
}

fn primitive(name: &str) -> Option<hir::PrimitiveType> {
    Some(match name {
        "bool" => hir::PrimitiveType::Bool,
        "i8" => hir::PrimitiveType::I8,
        "i16" => hir::PrimitiveType::I16,
        "i32" => hir::PrimitiveType::I32,
        "i64" => hir::PrimitiveType::I64,
        "isize" => hir::PrimitiveType::ISize,
        "u8" => hir::PrimitiveType::U8,
        "u16" => hir::PrimitiveType::U16,
        "u32" => hir::PrimitiveType::U32,
        "u64" => hir::PrimitiveType::U64,
        "usize" => hir::PrimitiveType::USize,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::{Function, Parameter, Struct, Type};
    use crate::parser::block::Block;

    fn block(statements: Vec<AS<'static>>, tail: Option<AE<'static>>) -> Block<'static> {
        Block {
            statements,
            tail: tail.map(Box::new),
        }
    }

    fn function(name: &'static str, body: Block<'static>) -> AI<'static> {
        AI::Function(Function {
            name,
            generic_parameter: vec![],
            parameters: vec![],
            return_type: Type::Named("i64"),
            body,
        })
    }

    #[test]
    fn resolves_forward_items_and_local_scopes() {
        let items = vec![
            function(
                "first",
                block(
                    vec![AS::Let {
                        name: "x",
                        var_type: Some(Type::Named("i64")),
                        value: AE::Integer(1),
                    }],
                    Some(AE::Call {
                        calle: Box::new(AE::Identifier("second")),
                        generic_args: vec![],
                        args: vec![AE::Identifier("x")],
                    }),
                ),
            ),
            AI::Function(Function {
                name: "second",
                generic_parameter: vec![],
                parameters: vec![Parameter {
                    name: "arg",
                    var_type: Type::Named("i64"),
                }],
                return_type: Type::Named("i64"),
                body: block(vec![], Some(AE::Identifier("arg"))),
            }),
        ];
        let program = lower_program(&items).unwrap();
        let hir::Item::Function(first) = &program.items[0] else {
            panic!()
        };
        assert_eq!(
            first.body.statements[0],
            hir::Statement::Let {
                id: hir::LocalId(0),
                ty: Some(hir::Type::Primitive(hir::PrimitiveType::I64)),
                value: hir::Expression::Integer(1),
            }
        );
        assert_eq!(
            first.body.tail,
            Some(Box::new(hir::Expression::Call {
                callee: Box::new(hir::Expression::Item(hir::ItemId(1))),
                generic_args: vec![],
                args: vec![hir::Expression::Local(hir::LocalId(0))],
            }))
        );
    }

    #[test]
    fn resolves_generic_types_and_shared_field_names() {
        let items = vec![
            AI::Struct(Struct {
                name: "Box",
                generic_parameters: vec![AG::Type("T")],
                fields: vec![Parameter {
                    name: "value",
                    var_type: Type::Named("T"),
                }],
            }),
            AI::Struct(Struct {
                name: "Number",
                generic_parameters: vec![],
                fields: vec![Parameter {
                    name: "value",
                    var_type: Type::Named("i64"),
                }],
            }),
            AI::Function(Function {
                name: "read",
                generic_parameter: vec![],
                parameters: vec![Parameter {
                    name: "box",
                    var_type: Type::Generic {
                        typename: "Box",
                        args: vec![Type::Named("i64")],
                    },
                }],
                return_type: Type::Named("i64"),
                body: block(
                    vec![],
                    Some(AE::FieldAccess {
                        object: Box::new(AE::Identifier("box")),
                        field: "value",
                    }),
                ),
            }),
        ];
        let program = lower_program(&items).unwrap();
        let hir::Item::Struct(first) = &program.items[0] else {
            panic!()
        };
        let hir::Item::Struct(second) = &program.items[1] else {
            panic!()
        };
        assert_eq!(first.fields[0].id, second.fields[0].id);
        assert_eq!(first.fields[0].ty, hir::Type::Param(hir::TypeParamId(0)));
        let hir::Item::Function(read) = &program.items[2] else {
            panic!()
        };
        assert_eq!(
            read.parameters[0].ty,
            hir::Type::Generic {
                item: hir::ItemId(0),
                args: vec![hir::Type::Primitive(hir::PrimitiveType::I64)]
            }
        );
        assert_eq!(
            read.body.tail,
            Some(Box::new(hir::Expression::FieldAccess {
                object: Box::new(hir::Expression::Local(hir::LocalId(0))),
                field: hir::FieldId(0)
            }))
        );
    }

    #[test]
    fn rejects_duplicate_and_unresolved_names() {
        let duplicate = vec![
            function("same", block(vec![], None)),
            function("same", block(vec![], None)),
        ];
        assert_eq!(
            lower_program(&duplicate),
            Err(HirError::DuplicatedName("same".into()))
        );
        let unknown = vec![function(
            "f",
            block(vec![], Some(AE::Identifier("missing"))),
        )];
        assert_eq!(
            lower_program(&unknown),
            Err(HirError::UnknownName("missing".into()))
        );
    }

    #[test]
    fn nested_bindings_shadow_outer_bindings_without_escaping() {
        let items = vec![function(
            "f",
            block(
                vec![
                    AS::Let {
                        name: "x",
                        var_type: None,
                        value: AE::Integer(1),
                    },
                    AS::Expression(AE::Block(block(
                        vec![AS::Let {
                            name: "x",
                            var_type: None,
                            value: AE::Identifier("x"),
                        }],
                        Some(AE::Identifier("x")),
                    ))),
                ],
                Some(AE::Identifier("x")),
            ),
        )];
        let program = lower_program(&items).unwrap();
        let hir::Item::Function(f) = &program.items[0] else {
            panic!()
        };
        let hir::Statement::Expression(hir::Expression::Block(inner)) = &f.body.statements[1]
        else {
            panic!()
        };
        assert_eq!(
            inner.statements[0],
            hir::Statement::Let {
                id: hir::LocalId(1),
                ty: None,
                value: hir::Expression::Local(hir::LocalId(0)),
            }
        );
        assert_eq!(
            inner.tail,
            Some(Box::new(hir::Expression::Local(hir::LocalId(1))))
        );
        assert_eq!(
            f.body.tail,
            Some(Box::new(hir::Expression::Local(hir::LocalId(0))))
        );
    }
}

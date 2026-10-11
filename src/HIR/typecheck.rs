use std::collections::HashMap;

use crate::HIR::ast::{
    BinaryOp, Expression, GenericParam, Item, LocalId, PrimitiveType, Program, Type,
};

enum TypeError {
    InvalidBinaryOperands,
    NotAType,
    UnsupportedEq,
    TypeMismatch,
    UnknownLocal,
    UnsupportedExpression,
    WrongGenericCount,
}

type Result<T> = std::result::Result<T, TypeError>;

impl Program {
    fn supports_eq(&self, ty: &Type) -> bool {
        self.supports_eq_inner(ty, &mut Vec::new())
    }

    fn supports_eq_inner(&self, ty: &Type, visiting: &mut Vec<Type>) -> bool {
        match ty {
            Type::Primitive(_) => true,
            Type::Pointer { .. } => true,
            Type::Reference { inner, .. } => self.supports_eq_inner(inner, visiting),
            Type::TypeParameter(_) => false,
            Type::Named { id, args } => {
                if visiting.contains(ty) {
                    return true;
                }
                let Some(Item::Struct(definition)) = self.items.get(id.0 as usize) else {
                    return false;
                };

                let type_params: Vec<_> = definition.generics.iter().filter_map(|param| {
                    if let GenericParam::Type(id) = param { Some(*id) } else { None }
                }).collect();
                if type_params.len() != args.len() { return false; }
                visiting.push(ty.clone());
                let supports_fields = definition.fields.iter().all(|field| {
                    let field_ty = Self::substitute_type(&field.ty, &type_params, args);
                    self.supports_eq_inner(&field_ty, visiting)
                });
                visiting.pop();
                supports_fields
            }
        }
    }

    fn substitute_type(ty: &Type, params: &[crate::HIR::ast::TypeParamId], args: &[Type]) -> Type {
        match ty {
            Type::TypeParameter(id) => params.iter().position(|param| param == id)
                .and_then(|index| args.get(index)).cloned().unwrap_or_else(|| ty.clone()),
            Type::Named { id, args: nested } => Type::Named { id: *id, args: nested.iter().map(|arg| Self::substitute_type(arg, params, args)).collect() },
            Type::Reference { mutable, inner } => Type::Reference { mutable: *mutable, inner: Box::new(Self::substitute_type(inner, params, args)) },
            Type::Pointer { mutable, inner } => Type::Pointer { mutable: *mutable, inner: Box::new(Self::substitute_type(inner, params, args)) },
            _ => ty.clone(),
        }
    }
    fn check_declared_types(&self) -> Result<()> {
        for item in &self.items {
            match item {
                Item::Function(function) => {
                    let mut locals = HashMap::new();
                    for parameter in &function.parameters {
                        self.check_type(&parameter.ty, &function.generics)?;
                        locals.insert(parameter.id, parameter.ty.clone());
                    }

                    self.check_type(&function.return_type, &function.generics)?;
                    self.check_block(&function.body, &mut locals, Some(&function.return_type))?;
                }

                Item::Struct(s) => {
                    for field in &s.fields {
                        self.check_type(&field.ty, &s.generics)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn check_block(
        &self,
        block: &crate::HIR::ast::Block,
        locals: &mut HashMap<LocalId, Type>,
        expected_tail: Option<&Type>,
    ) -> Result<()> {
        for statement in &block.statements {
            match statement {
                crate::HIR::ast::Statement::Let { id, ty, value } => {
                    let inferred = self.infer_expr_expected(value, locals, ty.as_ref())?;
                    if ty.as_ref().is_some_and(|declared| declared != &inferred) {
                        return Err(TypeError::TypeMismatch);
                    }
                    locals.insert(*id, ty.clone().unwrap_or(inferred));
                }
                crate::HIR::ast::Statement::Expression(expr) => {
                    self.infer_expr(expr, locals)?;
                }
            }
        }
        if let Some(tail) = &block.tail {
            self.infer_expr_expected(tail, locals, expected_tail)?;
        }
        Ok(())
    }
    fn check_type(&self, ty: &Type, generics: &[GenericParam]) -> Result<()> {
        match ty {
            Type::TypeParameter(id) => {
                if generics.iter().any(
                    |param| matches!(param, GenericParam::Type(generic_id) if generic_id == id),
                ) {
                    Ok(())
                } else {
                    Err(TypeError::NotAType)
                }
            }
            Type::Named { id: item, args } => {
                let Some(Item::Struct(s)) = self.items.get(item.0 as usize) else {
                    return Err(TypeError::NotAType);
                };

                let expected = s
                    .generics
                    .iter()
                    .filter(|g| matches!(g, GenericParam::Type(_)))
                    .count();

                if args.len() != expected {
                    return Err(TypeError::WrongGenericCount);
                }

                for arg in args {
                    self.check_type(arg, generics)?;
                }

                Ok(())
            }
            Type::Primitive(_) => Ok(()),

            Type::Reference { inner, .. } => self.check_type(inner, generics),
            Type::Pointer { inner, .. } => self.check_type(inner, generics),
        }
    }
    fn infer_expr(&self, expr: &Expression, locals: &mut HashMap<LocalId, Type>) -> Result<Type> {
        self.infer_expr_expected(expr, locals, None)
    }

    fn infer_expr_expected(
        &self,
        expr: &Expression,
        locals: &mut HashMap<LocalId, Type>,
        expected: Option<&Type>,
    ) -> Result<Type> {
        match expr {
            Expression::Local(id) => locals.get(id).cloned().ok_or(TypeError::UnknownLocal),
            Expression::Integer(value) => {
                let ty = expected.cloned().unwrap_or(Type::Primitive(PrimitiveType::I32));
                if !is_integer_type(&ty) || !integer_fits(*value, &ty) {
                    return Err(TypeError::TypeMismatch);
                }
                Ok(ty)
            }
            Expression::Float(_) => {
                let ty = Type::Primitive(PrimitiveType::F64);
                if expected.is_some_and(|expected| expected != &ty) {
                    return Err(TypeError::TypeMismatch);
                }
                Ok(ty)
            }
            Expression::Binary { op, left, right } => {
                self.infer_binary(op, left, right, locals, expected)
            }
            Expression::Block(block) => {
                let mut inner_locals = locals.clone();
                self.check_block(block, &mut inner_locals, expected)?;
                block.tail.as_ref().map(|tail| self.infer_expr_expected(tail, &mut inner_locals, expected))
                    .ok_or(TypeError::UnsupportedExpression)?
            }
            _ => Err(TypeError::UnsupportedExpression),
        }
    }
    fn infer_binary(
        &self,
        op: &BinaryOp,
        left: &Expression,
        right: &Expression,
        locals: &mut HashMap<LocalId, Type>,
        expected: Option<&Type>,
    ) -> Result<Type> {
        let result_is_eq = matches!(op, BinaryOp::Eq);
        let mut operand_expected = expected.filter(|ty| !result_is_eq && ty.is_numerical()).cloned();
        if operand_expected.is_none() {
            let left_is_literal = matches!(left, Expression::Integer(_));
            let right_is_literal = matches!(right, Expression::Integer(_));
            if left_is_literal && !right_is_literal {
                operand_expected = Some(self.infer_expr(right, locals)?);
            } else if right_is_literal && !left_is_literal {
                operand_expected = Some(self.infer_expr(left, locals)?);
            }
        }
        let left_ty = self.infer_expr_expected(left, locals, operand_expected.as_ref())?;
        let right_ty = self.infer_expr_expected(right, locals, operand_expected.as_ref())?;

        if left_ty != right_ty {
            return Err(TypeError::TypeMismatch);
        }

        let answer = match op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Divide => {
                if left_ty.is_numerical() {
                    Ok(left_ty)
                } else {
                    Err(TypeError::InvalidBinaryOperands)
                }
            }
            BinaryOp::Eq => {
                if !self.supports_eq(&left_ty) {
                    return Err(TypeError::UnsupportedEq);
                }
                Ok(Type::Primitive(PrimitiveType::Bool))
            }
        }?;

        Ok(answer)
    }
}

fn is_integer_type(ty: &Type) -> bool {
    matches!(ty, Type::Primitive(
        PrimitiveType::I8 | PrimitiveType::I16 | PrimitiveType::I32 | PrimitiveType::I64 |
        PrimitiveType::ISize | PrimitiveType::U8 | PrimitiveType::U16 | PrimitiveType::U32 |
        PrimitiveType::U64 | PrimitiveType::USize
    ))
}

fn integer_fits(value: i64, ty: &Type) -> bool {
    match ty {
        Type::Primitive(PrimitiveType::I8) => i8::try_from(value).is_ok(),
        Type::Primitive(PrimitiveType::I16) => i16::try_from(value).is_ok(),
        Type::Primitive(PrimitiveType::I32) => i32::try_from(value).is_ok(),
        Type::Primitive(PrimitiveType::I64 | PrimitiveType::ISize) => true,
        Type::Primitive(PrimitiveType::U8) => u8::try_from(value).is_ok(),
        Type::Primitive(PrimitiveType::U16) => u16::try_from(value).is_ok(),
        Type::Primitive(PrimitiveType::U32) => u32::try_from(value).is_ok(),
        Type::Primitive(PrimitiveType::U64 | PrimitiveType::USize) => value >= 0,
        _ => false,
    }
}

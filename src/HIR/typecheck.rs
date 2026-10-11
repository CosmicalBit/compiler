use std::collections::HashMap;

use crate::HIR::ast::{
    BinaryOp, Expression, GenericParam, Item, LocalId, PrimitiveType, Program, Type,
};

enum TypeError {
    InvalidBinaryOperands,
    NotAType,
    UnssupportedEq,
    TypeMissmatch,
    UnkownLocal,
    WrongGenericCount,
}

type Result<T> = std::result::Result<T, TypeError>;

impl Program {
    fn check_declared_types(&self) -> Result<()> {
        for item in &self.items {
            match item {
                Item::Function(function) => {
                    for parameter in &function.parameters {
                        self.check_type(&parameter.ty, &function.generics)?;
                    }

                    self.check_type(&function.return_type, &function.generics)?;
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
        match expr {
            Expression::Local(id) => locals.get(id).cloned().ok_or(TypeError::UnkownLocal),
            Expression::Integer(_) => Ok(Type::Primitive(PrimitiveType::I32)),
            Expression::Float(_) => Ok(Type::Primitive(PrimitiveType::F64)),
            Expression::Binary { op, left, right } => self.infer_binary(op, left, right, locals),
            _ => Err(TypeError::NotAType),
        }
    }
    fn infer_binary(
        &self,
        op: &BinaryOp,
        left: &Expression,
        right: &Expression,
        locals: &mut HashMap<LocalId, Type>,
    ) -> Result<Type> {
        let left_ty = self.infer_expr(left, locals)?;
        let right_ty = self.infer_expr(right, locals)?;

        if left_ty != right_ty {
            return Err(TypeError::TypeMissmatch);
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
                if !Type::supports_wq(&left_ty) {
                    return Err(TypeError::UnssupportedEq);
                }
                Ok(Type::Primitive(PrimitiveType::Bool))
            }
        }?;

        Ok(answer)
    }
}

use super::{
    Addition, BitwiseAnd, BitwiseOR, BitwiseXOR, Cast, Comparaison, Equation, FieldAccess, FnCall,
    ListAccess, LogicalAnd, LogicalOr, Product, Range, Shift, Substraction, TupleAccess,
    UnaryOperation,
};

use crate::ast::expressions::data::{Data, Primitive, Variable};
use crate::ast::expressions::{Atomic, Expression};
use crate::semantic::scope::static_types::{
    AddrType, NumberType, PrimitiveType, RangeType, StaticType,
};
use crate::vm::platform::Lib;
use crate::{p_num, resolve_metadata};

use crate::semantic::scope::scope::Scope;
use crate::semantic::scope::type_traits::{GetSubTypes, TypeChecking};
use crate::semantic::scope::user_type_impl::UserType;
use crate::semantic::{
    scope::type_traits::OperandMerging, CompatibleWith, Either, Resolve, SemanticError, TypeOf,
};
use crate::semantic::{ArcMutex, EType, Info};
use crate::vm::vm::Locatable;

impl Resolve for UnaryOperation {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match self {
            UnaryOperation::Minus { value, metadata } => {
                // let binding = Some(Either::Static(
                //     StaticType::Primitive(PrimitiveType::Number(NumberType::F64)).into(),
                // ));
                let _ = value.resolve(scope, context, &mut None)?;
                let value_type =
                    value.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;

                // let _ = <EType as OperandMerging>::can_substract(&value_type)?;
                match value_type {
                    Either::Static(value) => match value.as_ref() {
                        StaticType::Primitive(value) => match value {
                            PrimitiveType::Number(
                                NumberType::I128
                                | NumberType::I64
                                | NumberType::I32
                                | NumberType::I16
                                | NumberType::I8
                                | NumberType::F64,
                            ) => {}
                            _ => return Err(SemanticError::IncompatibleOperation),
                        },
                        _ => return Err(SemanticError::IncompatibleOperation),
                    },
                    Either::User(_) => return Err(SemanticError::IncompatibleOperation),
                }
                {
                    metadata.info =
                        Info::Resolved {
                            context: context.clone(),
                            signature: Some(value.type_of(&crate::arw_read!(
                                scope,
                                SemanticError::ConcurrencyError
                            )?)?),
                        };
                }
                Ok(())
            }
            UnaryOperation::Not { value, metadata } => {
                let _ = value.resolve(scope, context, &mut None)?;
                let value_type =
                    value.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
                let _ = <EType as OperandMerging>::can_negate(&value_type)?;
                {
                    metadata.info =
                        Info::Resolved {
                            context: context.clone(),
                            signature: Some(value.type_of(&crate::arw_read!(
                                scope,
                                SemanticError::ConcurrencyError
                            )?)?),
                        };
                }
                Ok(())
            }
        }
    }
}

impl Resolve for TupleAccess {
    type Output = ();
    type Context = Option<EType>;
    type Extra = Option<EType>;
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let _ = self.var.resolve(scope, context, extra)?;
        let var_type = self
            .var
            .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
        if !var_type.is_dotnum_indexable() {
            Err(SemanticError::ExpectedIndexable)
        } else {
            let Some(fields) = &var_type.get_fields() else {
                return Err(SemanticError::InvalidPattern);
            };
            if self.index >= fields.len() {
                Err(SemanticError::InvalidPattern)
            } else {
                let item_type = var_type
                    .get_nth(&self.index)
                    .ok_or(SemanticError::ExpectedIndexable)?;

                self.metadata.info = Info::Resolved {
                    context: Some(var_type),
                    signature: Some(item_type),
                };
                Ok(())
            }
        }
    }
}

impl Resolve for ListAccess {
    type Output = ();
    type Context = Option<EType>;
    type Extra = Option<EType>;
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let _ = self.var.resolve(scope, context, extra)?;
        let var_type = self
            .var
            .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
        if var_type.is_indexable() {
            let _ = self.index.resolve(scope, &Some(p_num!(U64)), extra)?;
            let index_type = self
                .index
                .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
            if index_type.is_u64() {
                let item_type = var_type.get_item().ok_or(SemanticError::ExpectedIterable)?;

                self.metadata.info = Info::Resolved {
                    context: Some(var_type),
                    signature: Some(item_type),
                };
                Ok(())
            } else {
                Err(SemanticError::ExpectedIndexable)
            }
        } else {
            Err(SemanticError::ExpectedIndexable)
        }
    }
}

impl Resolve for FieldAccess {
    type Output = ();
    type Context = Option<EType>;
    type Extra = Option<EType>;
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let _ = self.var.resolve(scope, context, extra)?;
        let mut var_type = self
            .var
            .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;

        match &var_type {
            Either::Static(value) => match value.as_ref() {
                StaticType::Address(AddrType(inner)) => var_type = inner.as_ref().clone(),
                _ => return Err(SemanticError::ExpectedStruct),
            },
            _ => {}
        }
        match &var_type {
            Either::Static(_) => return Err(SemanticError::ExpectedStruct),
            Either::User(value) => match value.as_ref() {
                UserType::Struct(value) => {
                    let Some(field_id) = self.field.as_ref().most_left_id() else {
                        return Err(SemanticError::UnknownField);
                    };
                    let Some((_, field_type)) = value.fields.iter().find(|(id, _)| *id == field_id)
                    else {
                        return Err(SemanticError::UnknownField);
                    };
                    let _ = self
                        .field
                        .resolve(scope, context, &mut Some(field_type.clone()))?;
                    let field_type = self
                        .field
                        .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;

                    self.metadata.info = Info::Resolved {
                        context: Some(var_type),
                        signature: Some(field_type),
                    };
                    Ok(())
                }
                _ => return Err(SemanticError::ExpectedStruct),
            },
        }
    }
}

impl Resolve for FnCall {
    type Output = ();
    type Context = Option<EType>;
    type Extra = Option<EType>;
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match self.fn_var.as_mut() {
            Expression::Atomic(Atomic::Data(Data::Variable(Variable { id, .. }))) => {
                let found = crate::arw_read!(scope, SemanticError::ConcurrencyError)?.find_var(id);
                if found.is_err() || self.lib.is_some() {
                    if let Some(mut api) = Lib::from(&self.lib, id) {
                        let _ = api.resolve(scope, context, &mut self.params)?;
                        *self.platform.as_ref().borrow_mut() = Some(api);
                        resolve_metadata!(self.metadata.info, self, scope, context);
                        return Ok(());
                    }
                }
            }
            _ => {}
        }

        let _ = self.fn_var.resolve(scope, context, extra)?;
        let fn_var_type = self
            .fn_var
            .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
        if !<EType as TypeChecking>::is_callable(&fn_var_type) {
            return Err(SemanticError::ExpectedCallable);
        }

        for (index, expr) in self.params.iter_mut().enumerate() {
            let param_context = <EType as GetSubTypes>::get_nth(&fn_var_type, &index);
            let _ = expr.resolve(scope, &param_context, &mut None)?;
        }

        let Some(fields) = <EType as GetSubTypes>::get_fields(&fn_var_type) else {
            return Err(SemanticError::ExpectedCallable);
        };
        if self.params.len() != fields.len() {
            return Err(SemanticError::IncorrectArguments);
        }
        for (index, (_, field_type)) in fields.iter().enumerate() {
            let _ = field_type.compatible_with(
                &self.params[index],
                &crate::arw_read!(scope, SemanticError::ConcurrencyError)?,
            )?;
        }
        resolve_metadata!(self.metadata.info, self, scope, context);
        Ok(())
    }
}

impl Resolve for Range {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let inner_context = match context {
            Some(context) => match context {
                Either::Static(context) => match context.as_ref() {
                    StaticType::Range(RangeType { num, .. }) => Some(Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(num.clone())).into(),
                    )),
                    _ => None,
                },
                _ => None,
            },
            None => None,
        };

        let _ = self.lower.resolve(scope, &inner_context, &mut None)?;
        let _ = self.upper.resolve(scope, &inner_context, &mut None)?;

        let _ = resolve_metadata!(self.metadata.info, self, scope, context);
        Ok(())
    }
}

impl Resolve for Product {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let (left, right) = match self {
            Product::Mult {
                left,
                right,
                metadata,
            } => (left, right),
            Product::Div {
                left,
                right,
                metadata,
            } => (left, right),
            Product::Mod {
                left,
                right,
                metadata,
            } => (left, right),
        };

        match (left.as_mut(), right.as_mut()) {
            (Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_)))), value) => {
                let _ = value.resolve(scope, context, &mut None)?;
                let right_type =
                    right.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
                let _ = left.resolve(scope, &Some(right_type), &mut None)?;
            }
            (value, Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_))))) => {
                let _ = value.resolve(scope, context, &mut None)?;
                let left_type =
                    left.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
                let _ = right.resolve(scope, &Some(left_type), &mut None)?;
            }
            _ => {
                let _ = left.resolve(scope, context, &mut None)?;
                let _ = right.resolve(scope, context, &mut None)?;
            }
        }

        let left_type = left.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
        let _ = <EType as OperandMerging>::can_product(&left_type)?;

        let right_type =
            right.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
        let _ = <EType as OperandMerging>::can_product(&right_type)?;

        // let _ = left_type.compatible_with(right.as_ref(), &block.borrow())?;
        let merge_type =
            self.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
        match self {
            Product::Mult {
                left,
                right,
                metadata,
            } => {
                metadata.info = Info::Resolved {
                    context: context.clone(),
                    signature: Some(merge_type),
                }
            }
            Product::Div {
                left,
                right,
                metadata,
            } => {
                metadata.info = Info::Resolved {
                    context: context.clone(),
                    signature: Some(merge_type),
                }
            }
            Product::Mod {
                left,
                right,
                metadata,
            } => {
                metadata.info = Info::Resolved {
                    context: context.clone(),
                    signature: Some(merge_type),
                }
            }
        }

        Ok(())
    }
}
impl Resolve for Addition {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match (self.left.as_mut(), self.right.as_mut()) {
            (Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_)))), value) => {
                let _ = value.resolve(scope, context, &mut None)?;
                let right_type = self
                    .right
                    .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
                let _ = self.left.resolve(scope, &Some(right_type), &mut None)?;
            }
            (value, Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_))))) => {
                let _ = value.resolve(scope, context, &mut None)?;
                let left_type = self
                    .left
                    .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
                let _ = self.right.resolve(scope, &Some(left_type), &mut None)?;
            }
            _ => {
                let _ = self.left.resolve(scope, context, &mut None)?;
                let _ = self.right.resolve(scope, context, &mut None)?;
            }
        }

        let left_type = self
            .left
            .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
        let _ = <EType as OperandMerging>::can_add(&left_type)?;

        let right_type = self
            .right
            .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
        let _ = <EType as OperandMerging>::can_add(&right_type)?;
        // let _ = left_type.compatible_with(self.right.as_ref(), &block.borrow())?;
        resolve_metadata!(self.metadata.info, self, scope, context);
        Ok(())
    }
}

impl Resolve for Substraction {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match (self.left.as_mut(), self.right.as_mut()) {
            (Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_)))), value) => {
                let _ = value.resolve(scope, context, &mut None)?;
                let right_type = self
                    .right
                    .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
                let _ = self.left.resolve(scope, &Some(right_type), &mut None)?;
            }
            (value, Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_))))) => {
                let _ = value.resolve(scope, context, &mut None)?;
                let left_type = self
                    .left
                    .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
                let _ = self.right.resolve(scope, &Some(left_type), &mut None)?;
            }
            _ => {
                let _ = self.left.resolve(scope, context, &mut None)?;
                let _ = self.right.resolve(scope, context, &mut None)?;
            }
        }

        let left_type = self
            .left
            .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
        let _ = <EType as OperandMerging>::can_substract(&left_type)?;

        let right_type = self
            .right
            .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
        let _ = <EType as OperandMerging>::can_substract(&right_type)?;

        // let _ = left_type.compatible_with(self.right.as_ref(), &block.borrow())?;
        resolve_metadata!(self.metadata.info, self, scope, context);
        Ok(())
    }
}

impl Resolve for Shift {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let (left, right, metadata) = match self {
            Shift::Left {
                left,
                right,
                metadata,
            } => (left, right, metadata),
            Shift::Right {
                left,
                right,
                metadata,
            } => (left, right, metadata),
        };

        match (left.as_mut(), right.as_mut()) {
            (Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_)))), value) => {
                let _ = value.resolve(scope, context, &mut None)?;
                let right_type =
                    right.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
                let _ = left.resolve(scope, &Some(right_type), &mut None)?;
            }
            (value, Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_))))) => {
                let _ = value.resolve(scope, context, &mut None)?;
                let left_type =
                    left.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
                let _ = right.resolve(scope, &Some(left_type), &mut None)?;
            }
            _ => {
                let _ = left.resolve(scope, context, &mut None)?;
                let _ = right.resolve(scope, context, &mut None)?;
            }
        }
        let left_type = left.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
        let _ = <EType as OperandMerging>::can_shift(&left_type)?;

        let right_type =
            right.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
        let _ = <EType as OperandMerging>::can_shift(&right_type)?;

        // let _ = left_type.compatible_with(right.as_ref(), &block.borrow())?;
        let merge_type =
            self.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
        match self {
            Shift::Left {
                left,
                right,
                metadata,
            } => {
                metadata.info = Info::Resolved {
                    context: context.clone(),
                    signature: Some(merge_type),
                }
            }
            Shift::Right {
                left,
                right,
                metadata,
            } => {
                metadata.info = Info::Resolved {
                    context: context.clone(),
                    signature: Some(merge_type),
                }
            }
        }
        Ok(())
    }
}
impl Resolve for BitwiseAnd {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match (self.left.as_mut(), self.right.as_mut()) {
            (Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_)))), value) => {
                let _ = value.resolve(scope, context, &mut None)?;
                let right_type = self
                    .right
                    .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
                let _ = self.left.resolve(scope, &Some(right_type), &mut None)?;
            }
            (value, Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_))))) => {
                let _ = value.resolve(scope, context, &mut None)?;
                let left_type = self
                    .left
                    .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
                let _ = self.right.resolve(scope, &Some(left_type), &mut None)?;
            }
            _ => {
                let _ = self.left.resolve(scope, context, &mut None)?;
                let _ = self.right.resolve(scope, context, &mut None)?;
            }
        }
        let left_type = self
            .left
            .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
        let _ = <EType as OperandMerging>::can_bitwise_and(&left_type)?;

        let right_type = self
            .right
            .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
        let _ = <EType as OperandMerging>::can_bitwise_and(&right_type)?;

        // let _ = left_type.compatible_with(self.right.as_ref(), &block.borrow())?;
        resolve_metadata!(self.metadata.info, self, scope, context);
        Ok(())
    }
}
impl Resolve for BitwiseXOR {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match (self.left.as_mut(), self.right.as_mut()) {
            (Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_)))), value) => {
                let _ = value.resolve(scope, context, &mut None)?;
                let right_type = self
                    .right
                    .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
                let _ = self.left.resolve(scope, &Some(right_type), &mut None)?;
            }
            (value, Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_))))) => {
                let _ = value.resolve(scope, context, &mut None)?;
                let left_type = self
                    .left
                    .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
                let _ = self.right.resolve(scope, &Some(left_type), &mut None)?;
            }
            _ => {
                let _ = self.left.resolve(scope, context, &mut None)?;
                let _ = self.right.resolve(scope, context, &mut None)?;
            }
        }

        let left_type = self
            .left
            .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
        let _ = <EType as OperandMerging>::can_bitwise_xor(&left_type)?;

        let right_type = self
            .right
            .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
        let _ = <EType as OperandMerging>::can_bitwise_xor(&right_type)?;

        // let _ = left_type.compatible_with(self.right.as_ref(), &block.borrow())?;
        resolve_metadata!(self.metadata.info, self, scope, context);
        Ok(())
    }
}
impl Resolve for BitwiseOR {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match (self.left.as_mut(), self.right.as_mut()) {
            (Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_)))), value) => {
                let _ = value.resolve(scope, context, &mut None)?;
                let right_type = self
                    .right
                    .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
                let _ = self.left.resolve(scope, &Some(right_type), &mut None)?;
            }
            (value, Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_))))) => {
                let _ = value.resolve(scope, context, &mut None)?;
                let left_type = self
                    .left
                    .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
                let _ = self.right.resolve(scope, &Some(left_type), &mut None)?;
            }
            _ => {
                let _ = self.left.resolve(scope, context, &mut None)?;
                let _ = self.right.resolve(scope, context, &mut None)?;
            }
        }

        let left_type = self
            .left
            .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
        let _ = <EType as OperandMerging>::can_bitwise_or(&left_type)?;

        let right_type = self
            .right
            .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
        let _ = <EType as OperandMerging>::can_bitwise_or(&right_type)?;

        // let _ = left_type.compatible_with(self.right.as_ref(), &block.borrow())?;
        resolve_metadata!(self.metadata.info, self, scope, context);
        Ok(())
    }
}

impl Resolve for Cast {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let _ = self.left.resolve(scope, context, &mut None)?;
        let _ = self.right.resolve(scope, &(), &mut ())?;

        if let Some(l_metadata) = self.left.metadata_mut() {
            let signature = l_metadata.signature();
            let new_context = self
                .right
                .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;

            l_metadata.info = Info::Resolved {
                context: Some(new_context),
                signature,
            };
        }
        resolve_metadata!(self.metadata.info, self, scope, context);
        Ok(())
    }
}

impl Resolve for Comparaison {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let (left, right) = match self {
            Comparaison::Less {
                left,
                right,
                metadata,
            } => (left, right),
            Comparaison::LessEqual {
                left,
                right,
                metadata,
            } => (left, right),
            Comparaison::Greater {
                left,
                right,
                metadata,
            } => (left, right),
            Comparaison::GreaterEqual {
                left,
                right,
                metadata,
            } => (left, right),
        };

        match (left.as_mut(), right.as_mut()) {
            (Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_)))), value) => {
                let _ = value.resolve(scope, context, &mut None)?;
                let right_type =
                    right.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
                let _ = left.resolve(scope, &Some(right_type), &mut None)?;
            }
            (value, Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_))))) => {
                let _ = value.resolve(scope, context, &mut None)?;
                let left_type =
                    left.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
                let _ = right.resolve(scope, &Some(left_type), &mut None)?;
            }
            _ => {
                let _ = left.resolve(scope, context, &mut None)?;
                let _ = right.resolve(scope, context, &mut None)?;
            }
        }

        let left_type = left.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
        let _ = <EType as OperandMerging>::can_comparaison(&left_type)?;

        let right_type =
            right.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
        let _ = <EType as OperandMerging>::can_comparaison(&right_type)?;

        let _ = left_type.compatible_with(
            right.as_ref(),
            &crate::arw_read!(scope, SemanticError::ConcurrencyError)?,
        )?;

        let merge_type =
            self.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
        match self {
            Comparaison::Less {
                left,
                right,
                metadata,
            } => {
                metadata.info = Info::Resolved {
                    context: context.clone(),
                    signature: Some(merge_type),
                }
            }
            Comparaison::LessEqual {
                left,
                right,
                metadata,
            } => {
                metadata.info = Info::Resolved {
                    context: context.clone(),
                    signature: Some(merge_type),
                }
            }
            Comparaison::Greater {
                left,
                right,
                metadata,
            } => {
                metadata.info = Info::Resolved {
                    context: context.clone(),
                    signature: Some(merge_type),
                }
            }
            Comparaison::GreaterEqual {
                left,
                right,
                metadata,
            } => {
                metadata.info = Info::Resolved {
                    context: context.clone(),
                    signature: Some(merge_type),
                }
            }
        }
        Ok(())
    }
}

impl Resolve for Equation {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let (left, right) = match self {
            Equation::Equal {
                left,
                right,
                metadata,
            } => (left, right),
            Equation::NotEqual {
                left,
                right,
                metadata,
            } => (left, right),
        };

        match (left.as_mut(), right.as_mut()) {
            (Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_)))), value) => {
                let _ = value.resolve(scope, context, &mut None)?;
                let right_type =
                    right.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
                let _ = left.resolve(scope, &Some(right_type), &mut None)?;
            }
            (value, Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_))))) => {
                let _ = value.resolve(scope, context, &mut None)?;
                let left_type =
                    left.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
                let _ = right.resolve(scope, &Some(left_type), &mut None)?;
            }
            _ => {
                let _ = left.resolve(scope, context, &mut None)?;
                let _ = right.resolve(scope, context, &mut None)?;
            }
        }

        let left_type = left.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
        let _ = <EType as OperandMerging>::can_equate(&left_type)?;

        let right_type =
            right.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
        let _ = <EType as OperandMerging>::can_equate(&right_type)?;

        let _ = left_type.compatible_with(
            right.as_ref(),
            &crate::arw_read!(scope, SemanticError::ConcurrencyError)?,
        )?;

        let merge_type =
            self.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
        match self {
            Equation::Equal {
                left,
                right,
                metadata,
            } => {
                metadata.info = Info::Resolved {
                    context: context.clone(),
                    signature: Some(merge_type),
                }
            }
            Equation::NotEqual {
                left,
                right,
                metadata,
            } => {
                metadata.info = Info::Resolved {
                    context: context.clone(),
                    signature: Some(merge_type),
                }
            }
        }
        Ok(())
    }
}

impl Resolve for LogicalAnd {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match (self.left.as_mut(), self.right.as_mut()) {
            (Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_)))), value) => {
                let _ = value.resolve(scope, context, &mut None)?;
                let right_type = self
                    .right
                    .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
                let _ = self.left.resolve(scope, &Some(right_type), &mut None)?;
            }
            (value, Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_))))) => {
                let _ = value.resolve(scope, context, &mut None)?;
                let left_type = self
                    .left
                    .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
                let _ = self.right.resolve(scope, &Some(left_type), &mut None)?;
            }
            _ => {
                let _ = self.left.resolve(scope, context, &mut None)?;
                let _ = self.right.resolve(scope, context, &mut None)?;
            }
        }

        let left_type = self
            .left
            .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
        let _ = <EType as OperandMerging>::can_logical_and(&left_type)?;

        let right_type = self
            .right
            .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
        let _ = <EType as OperandMerging>::can_logical_and(&right_type)?;

        let _ = left_type.compatible_with(
            self.right.as_ref(),
            &crate::arw_read!(scope, SemanticError::ConcurrencyError)?,
        )?;
        resolve_metadata!(self.metadata.info, self, scope, context);
        Ok(())
    }
}
impl Resolve for LogicalOr {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match (self.left.as_mut(), self.right.as_mut()) {
            (Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_)))), value) => {
                let _ = value.resolve(scope, context, &mut None)?;
                let right_type = self
                    .right
                    .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
                let _ = self.left.resolve(scope, &Some(right_type), &mut None)?;
            }
            (value, Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_))))) => {
                let _ = value.resolve(scope, context, &mut None)?;
                let left_type = self
                    .left
                    .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
                let _ = self.right.resolve(scope, &Some(left_type), &mut None)?;
            }
            _ => {
                let _ = self.left.resolve(scope, context, &mut None)?;
                let _ = self.right.resolve(scope, context, &mut None)?;
            }
        }

        let left_type = self
            .left
            .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
        let _ = <EType as OperandMerging>::can_logical_or(&left_type)?;

        let right_type = self
            .right
            .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
        let _ = <EType as OperandMerging>::can_logical_or(&right_type)?;

        let _ = left_type.compatible_with(
            self.right.as_ref(),
            &crate::arw_read!(scope, SemanticError::ConcurrencyError)?,
        )?;
        resolve_metadata!(self.metadata.info, self, scope, context);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use crate::{
        ast::{
            expressions::{operation::operation_parse::TryParseOperation, Expression},
            TryParse,
        },
        e_static, p_num,
        semantic::scope::{
            scope::Scope,
            static_types::{FnType, NumberType, PrimitiveType, SliceType, StaticType, StringType},
            var_impl::{Var, VarState},
        },
    };

    use super::*;

    #[test]
    fn valid_high_ord_math() {
        let mut expr = Product::parse("10 * 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Product::parse("10.0 * 10.0".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Product::parse("10 * (10+10)".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Product::parse("(10+10) * 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Product::parse("10 * x".into()).unwrap().1;
        let scope = Scope::new();
        let _ = crate::arw_write!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .register_var(Var {
                state: Cell::default(),
                id: "x".to_string().into(),
                type_sig: p_num!(I64),
                is_declared: Cell::new(false),
            })
            .unwrap();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_high_ord_math() {
        let mut expr = Product::parse("10 * 'a'".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_err());

        let mut expr = Product::parse("'a' * 'a'".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_err());

        let mut expr = Product::parse("10 * x".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_err());
    }

    #[test]
    fn valid_low_ord_math() {
        let mut expr = Addition::parse("10 + 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Addition::parse("10 - 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Addition::parse("10 + (10*10)".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Addition::parse("10 + 10*10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Addition::parse("(10 * 10) + 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Addition::parse("10 * 10 + 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Product::parse("10 * 10 + 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Product::parse("10 * 10 + 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Addition::parse("10 + x".into()).unwrap().1;
        let scope = Scope::new();
        let _ = crate::arw_write!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .register_var(Var {
                state: Cell::default(),
                id: "x".to_string().into(),
                type_sig: p_num!(I64),
                is_declared: Cell::new(false),
            })
            .unwrap();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_low_ord_math() {
        let mut expr = Addition::parse("10 + 'a'".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_err());

        let mut expr = Addition::parse("'a' + 'a'".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_err());

        let mut expr = Addition::parse("10 + x".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_err());

        let mut expr = Addition::parse("10.0 + 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_err());
    }

    #[test]
    fn valid_shift() {
        let mut expr = Shift::parse("10 >> 1".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Shift::parse("10 << 1".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_shift() {
        let mut expr = Shift::parse("10 >> 'a'".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_err());

        let mut expr = Shift::parse("'a' >> 1".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_err());
    }

    #[test]
    fn valid_bitwise() {
        let mut expr = BitwiseAnd::parse("10 & 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = BitwiseOR::parse("10 | 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = BitwiseXOR::parse("10 ^ 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_bitwise() {
        let mut expr = BitwiseAnd::parse("10 & 'a'".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_err());

        let mut expr = BitwiseOR::parse("'a' | 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_err());

        let mut expr = BitwiseXOR::parse("10 ^ 'a'".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_err());
    }

    #[test]
    fn valid_cast() {
        let mut expr = Cast::parse("10 as f64".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_ok(), "{:?}", res);
        let expr_type = expr
            .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError).unwrap())
            .unwrap();
        assert_eq!(p_num!(F64), expr_type);

        let mut expr = Cast::parse("'a' as u64".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_ok(), "{:?}", res);
        let expr_type = expr
            .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError).unwrap())
            .unwrap();
        assert_eq!(p_num!(U64), expr_type);
    }

    #[test]
    fn valid_comparaison() {
        let mut expr = Expression::parse("10 < 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Expression::parse("'a' > 'b'".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_comparaison() {
        let mut expr = Expression::parse("10 > 'a'".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_err());
    }

    #[test]
    fn valid_equation() {
        let mut expr = Expression::parse("10 == 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Expression::parse("'a' != 'b'".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_equation() {
        let mut expr = Expression::parse("10 == 'a'".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_err());
    }

    #[test]
    fn valid_and_or() {
        let mut expr = Expression::parse("true and false".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Expression::parse("true or false".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Expression::parse("true and 2 > 3".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = Expression::parse("true and true and true".into())
            .unwrap()
            .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_and_or() {
        let mut expr = Expression::parse("true and 2".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_err());

        let mut expr = Expression::parse("1 or true".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_err());
    }

    #[test]
    fn valid_unary() {
        let mut expr = UnaryOperation::parse("!true".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = UnaryOperation::parse("! ( true and false )".into())
            .unwrap()
            .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = UnaryOperation::parse("-1".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = UnaryOperation::parse("-1.0".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = UnaryOperation::parse("- ( 10 + 10 )".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_unary() {
        let mut expr = UnaryOperation::parse("!10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut ());
        assert!(res.is_err());

        let mut expr = UnaryOperation::parse("- true".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut ());
        assert!(res.is_err());
    }

    #[test]
    fn valid_call() {
        let mut expr = FnCall::parse("f(10,20+20)".into())
            .expect("Parsing should have succeeded")
            .1;

        let scope = Scope::new();
        let _ = crate::arw_write!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .register_var(Var {
                state: Cell::default(),
                id: "f".to_string().into(),
                type_sig: Either::Static(
                    StaticType::StaticFn(FnType {
                        params: vec![p_num!(I64), p_num!(I64)],
                        ret: Box::new(p_num!(I64)),
                        scope_params_size: 24,
                    })
                    .into(),
                ),
                is_declared: Cell::new(false),
            })
            .unwrap();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_ok(), "{:?}", res);

        let ret_type = expr
            .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError).unwrap())
            .unwrap();
        assert_eq!(ret_type, p_num!(I64))
    }

    #[test]
    fn robustness_call() {
        let mut expr = FnCall::parse("f(10,20+20)".into())
            .expect("Parsing should have succeeded")
            .1;

        let scope = Scope::new();
        let _ = crate::arw_write!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .register_var(Var {
                state: Cell::default(),
                id: "f".to_string().into(),
                type_sig: p_num!(I64),
                is_declared: Cell::new(false),
            })
            .unwrap();
        let res = expr.resolve(&scope, &None, &mut None);
        assert!(res.is_err());
    }

    #[test]
    fn valid_call_lib() {
        let mut expr = FnCall::parse("io::print(true)".into())
            .expect("Parsing should have succeeded")
            .1;

        let scope = Scope::new();
        let _ = crate::arw_write!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .register_var(Var {
                state: Cell::default(),
                id: "print".to_string().into(),
                type_sig: p_num!(I64),
                is_declared: Cell::new(false),
            })
            .unwrap();
        let _ = expr
            .resolve(&scope, &None, &mut None)
            .expect("Resolution should have succeeded");
    }

    #[test]
    fn robustness_call_lib() {
        let mut expr = FnCall::parse("print(true)".into())
            .expect("Parsing should have succeeded")
            .1;

        let scope = Scope::new();
        let _ = crate::arw_write!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .register_var(Var {
                state: Cell::default(),
                id: "print".to_string().into(),
                type_sig: p_num!(I64),
                is_declared: Cell::new(false),
            })
            .unwrap();
        let _ = expr
            .resolve(&scope, &None, &mut None)
            .expect_err("Resolution should have failed");
    }
}

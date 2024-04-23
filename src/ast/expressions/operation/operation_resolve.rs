use super::{
    Addition, BitwiseAnd, BitwiseOR, BitwiseXOR, Cast, Comparaison, Equation, LogicalAnd,
    LogicalOr, Product, Range, Shift, Substraction, UnaryOperation,
};

use crate::ast::expressions::data::{Data, Primitive};
use crate::ast::expressions::{Atomic, Expression};
use crate::resolve_metadata;
use crate::semantic::scope::static_types::{
    ClosureType, FnType, NumberType, PrimitiveType, RangeType, StaticType,
};
use crate::semantic::scope::type_traits::GetSubTypes;
use crate::semantic::scope::user_type_impl::UserType;

use crate::semantic::scope::scope_impl::Scope;
use crate::semantic::{
    scope::type_traits::OperandMerging, CompatibleWith, Either, Resolve, SemanticError, TypeOf,
};
use crate::semantic::{EType, Info, MutRc};
use std::{cell::RefCell, rc::Rc};

impl Resolve for UnaryOperation {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match self {
            UnaryOperation::Minus { value, metadata } => {
                // let binding = Some(Either::Static(
                //     StaticType::Primitive(PrimitiveType::Number(NumberType::F64)).into(),
                // ));
                let _ = value.resolve(scope, context, extra)?;
                let value_type = value.type_of(&scope.borrow())?;

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
                    let mut borrowed_metadata = metadata
                        .info
                        .as_ref()
                        .try_borrow_mut()
                        .map_err(|_| SemanticError::Default)?;

                    *borrowed_metadata = Info::Resolved {
                        context: context.clone(),
                        signature: Some(self.type_of(&scope.borrow())?),
                    };
                }
                Ok(())
            }
            UnaryOperation::Not { value, metadata } => {
                let _ = value.resolve(scope, context, extra)?;
                let value_type = value.type_of(&scope.borrow())?;
                let _ = <EType as OperandMerging>::can_negate(&value_type)?;
                {
                    let mut borrowed_metadata = metadata
                        .info
                        .as_ref()
                        .try_borrow_mut()
                        .map_err(|_| SemanticError::Default)?;

                    *borrowed_metadata = Info::Resolved {
                        context: context.clone(),
                        signature: Some(self.type_of(&scope.borrow())?),
                    };
                }
                Ok(())
            }
        }
    }
}

impl Resolve for Range {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
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

        let _ = self.lower.resolve(scope, &inner_context, extra)?;
        let _ = self.upper.resolve(scope, &inner_context, extra)?;

        let _ = resolve_metadata!(self.metadata, self, scope, context);
        Ok(())
    }
}

impl Resolve for Product {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let (left, right, metadata) = match self {
            Product::Mult {
                left,
                right,
                metadata,
            } => (left, right, metadata),
            Product::Div {
                left,
                right,
                metadata,
            } => (left, right, metadata),
            Product::Mod {
                left,
                right,
                metadata,
            } => (left, right, metadata),
        };

        match (left.as_ref(), right.as_ref()) {
            (Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_)))), value) => {
                let _ = value.resolve(scope, context, extra)?;
                let right_type = right.type_of(&scope.borrow())?;
                let _ = left.resolve(scope, &Some(right_type), extra)?;
            }
            (value, Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_))))) => {
                let _ = value.resolve(scope, context, extra)?;
                let left_type = left.type_of(&scope.borrow())?;
                let _ = right.resolve(scope, &Some(left_type), extra)?;
            }
            _ => {
                let _ = left.resolve(scope, context, extra)?;
                let _ = right.resolve(scope, context, extra)?;
            }
        }

        let left_type = left.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging>::can_product(&left_type)?;

        let right_type = right.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging>::can_product(&right_type)?;

        // let _ = left_type.compatible_with(right.as_ref(), &block.borrow())?;
        resolve_metadata!(metadata, self, scope, context);
        Ok(())
    }
}
impl Resolve for Addition {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match (self.left.as_ref(), self.right.as_ref()) {
            (Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_)))), value) => {
                let _ = value.resolve(scope, context, extra)?;
                let right_type = self.right.type_of(&scope.borrow())?;
                let _ = self.left.resolve(scope, &Some(right_type), extra)?;
            }
            (value, Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_))))) => {
                let _ = value.resolve(scope, context, extra)?;
                let left_type = self.left.type_of(&scope.borrow())?;
                let _ = self.right.resolve(scope, &Some(left_type), extra)?;
            }
            _ => {
                let _ = self.left.resolve(scope, context, extra)?;
                let _ = self.right.resolve(scope, context, extra)?;
            }
        }

        let left_type = self.left.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging>::can_add(&left_type)?;

        let right_type = self.right.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging>::can_add(&right_type)?;

        // let _ = left_type.compatible_with(self.right.as_ref(), &block.borrow())?;
        resolve_metadata!(self.metadata, self, scope, context);
        Ok(())
    }
}

impl Resolve for Substraction {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match (self.left.as_ref(), self.right.as_ref()) {
            (Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_)))), value) => {
                let _ = value.resolve(scope, context, extra)?;
                let right_type = self.right.type_of(&scope.borrow())?;
                let _ = self.left.resolve(scope, &Some(right_type), extra)?;
            }
            (value, Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_))))) => {
                let _ = value.resolve(scope, context, extra)?;
                let left_type = self.left.type_of(&scope.borrow())?;
                let _ = self.right.resolve(scope, &Some(left_type), extra)?;
            }
            _ => {
                let _ = self.left.resolve(scope, context, extra)?;
                let _ = self.right.resolve(scope, context, extra)?;
            }
        }

        let left_type = self.left.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging>::can_substract(&left_type)?;

        let right_type = self.right.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging>::can_substract(&right_type)?;

        // let _ = left_type.compatible_with(self.right.as_ref(), &block.borrow())?;
        resolve_metadata!(self.metadata, self, scope, context);
        Ok(())
    }
}

impl Resolve for Shift {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
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

        match (left.as_ref(), right.as_ref()) {
            (Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_)))), value) => {
                let _ = value.resolve(scope, context, extra)?;
                let right_type = right.type_of(&scope.borrow())?;
                let _ = left.resolve(scope, &Some(right_type), extra)?;
            }
            (value, Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_))))) => {
                let _ = value.resolve(scope, context, extra)?;
                let left_type = left.type_of(&scope.borrow())?;
                let _ = right.resolve(scope, &Some(left_type), extra)?;
            }
            _ => {
                let _ = left.resolve(scope, context, extra)?;
                let _ = right.resolve(scope, context, extra)?;
            }
        }
        let left_type = left.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging>::can_shift(&left_type)?;

        let right_type = right.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging>::can_shift(&right_type)?;

        // let _ = left_type.compatible_with(right.as_ref(), &block.borrow())?;
        resolve_metadata!(metadata, self, scope, context);
        Ok(())
    }
}
impl Resolve for BitwiseAnd {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match (self.left.as_ref(), self.right.as_ref()) {
            (Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_)))), value) => {
                let _ = value.resolve(scope, context, extra)?;
                let right_type = self.right.type_of(&scope.borrow())?;
                let _ = self.left.resolve(scope, &Some(right_type), extra)?;
            }
            (value, Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_))))) => {
                let _ = value.resolve(scope, context, extra)?;
                let left_type = self.left.type_of(&scope.borrow())?;
                let _ = self.right.resolve(scope, &Some(left_type), extra)?;
            }
            _ => {
                let _ = self.left.resolve(scope, context, extra)?;
                let _ = self.right.resolve(scope, context, extra)?;
            }
        }
        let left_type = self.left.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging>::can_bitwise_and(&left_type)?;

        let right_type = self.right.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging>::can_bitwise_and(&right_type)?;

        // let _ = left_type.compatible_with(self.right.as_ref(), &block.borrow())?;
        resolve_metadata!(self.metadata, self, scope, context);
        Ok(())
    }
}
impl Resolve for BitwiseXOR {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match (self.left.as_ref(), self.right.as_ref()) {
            (Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_)))), value) => {
                let _ = value.resolve(scope, context, extra)?;
                let right_type = self.right.type_of(&scope.borrow())?;
                let _ = self.left.resolve(scope, &Some(right_type), extra)?;
            }
            (value, Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_))))) => {
                let _ = value.resolve(scope, context, extra)?;
                let left_type = self.left.type_of(&scope.borrow())?;
                let _ = self.right.resolve(scope, &Some(left_type), extra)?;
            }
            _ => {
                let _ = self.left.resolve(scope, context, extra)?;
                let _ = self.right.resolve(scope, context, extra)?;
            }
        }

        let left_type = self.left.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging>::can_bitwise_xor(&left_type)?;

        let right_type = self.right.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging>::can_bitwise_xor(&right_type)?;

        // let _ = left_type.compatible_with(self.right.as_ref(), &block.borrow())?;
        resolve_metadata!(self.metadata, self, scope, context);
        Ok(())
    }
}
impl Resolve for BitwiseOR {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match (self.left.as_ref(), self.right.as_ref()) {
            (Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_)))), value) => {
                let _ = value.resolve(scope, context, extra)?;
                let right_type = self.right.type_of(&scope.borrow())?;
                let _ = self.left.resolve(scope, &Some(right_type), extra)?;
            }
            (value, Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_))))) => {
                let _ = value.resolve(scope, context, extra)?;
                let left_type = self.left.type_of(&scope.borrow())?;
                let _ = self.right.resolve(scope, &Some(left_type), extra)?;
            }
            _ => {
                let _ = self.left.resolve(scope, context, extra)?;
                let _ = self.right.resolve(scope, context, extra)?;
            }
        }

        let left_type = self.left.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging>::can_bitwise_or(&left_type)?;

        let right_type = self.right.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging>::can_bitwise_or(&right_type)?;

        // let _ = left_type.compatible_with(self.right.as_ref(), &block.borrow())?;
        resolve_metadata!(self.metadata, self, scope, context);
        Ok(())
    }
}

impl Resolve for Cast {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let _ = self.left.resolve(scope, context, extra)?;
        let _ = self.right.resolve(scope, &(), extra)?;
        if let Some(l_metadata) = self.left.metadata() {
            let signature = l_metadata.signature();
            let new_context = self.right.type_of(&scope.borrow())?;
            let mut borrowed_metadata = l_metadata
                .info
                .as_ref()
                .try_borrow_mut()
                .map_err(|_| SemanticError::Default)?;

            *borrowed_metadata = Info::Resolved {
                context: Some(new_context),
                signature,
            };
        }
        resolve_metadata!(self.metadata, self, scope, context);
        Ok(())
    }
}

impl Resolve for Comparaison {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let (left, right, metadata) = match self {
            Comparaison::Less {
                left,
                right,
                metadata,
            } => (left, right, metadata),
            Comparaison::LessEqual {
                left,
                right,
                metadata,
            } => (left, right, metadata),
            Comparaison::Greater {
                left,
                right,
                metadata,
            } => (left, right, metadata),
            Comparaison::GreaterEqual {
                left,
                right,
                metadata,
            } => (left, right, metadata),
        };

        match (left.as_ref(), right.as_ref()) {
            (Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_)))), value) => {
                let _ = value.resolve(scope, context, extra)?;
                let right_type = right.type_of(&scope.borrow())?;
                let _ = left.resolve(scope, &Some(right_type), extra)?;
            }
            (value, Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_))))) => {
                let _ = value.resolve(scope, context, extra)?;
                let left_type = left.type_of(&scope.borrow())?;
                let _ = right.resolve(scope, &Some(left_type), extra)?;
            }
            _ => {
                let _ = left.resolve(scope, context, extra)?;
                let _ = right.resolve(scope, context, extra)?;
            }
        }

        let left_type = left.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging>::can_comparaison(&left_type)?;

        let right_type = right.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging>::can_comparaison(&right_type)?;

        let _ = left_type.compatible_with(right.as_ref(), &scope.borrow())?;
        resolve_metadata!(metadata, self, scope, context);
        Ok(())
    }
}

impl Resolve for Equation {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let (left, right, metadata) = match self {
            Equation::Equal {
                left,
                right,
                metadata,
            } => (left, right, metadata),
            Equation::NotEqual {
                left,
                right,
                metadata,
            } => (left, right, metadata),
        };

        match (left.as_ref(), right.as_ref()) {
            (Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_)))), value) => {
                let _ = value.resolve(scope, context, extra)?;
                let right_type = right.type_of(&scope.borrow())?;
                let _ = left.resolve(scope, &Some(right_type), extra)?;
            }
            (value, Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_))))) => {
                let _ = value.resolve(scope, context, extra)?;
                let left_type = left.type_of(&scope.borrow())?;
                let _ = right.resolve(scope, &Some(left_type), extra)?;
            }
            _ => {
                let _ = left.resolve(scope, context, extra)?;
                let _ = right.resolve(scope, context, extra)?;
            }
        }

        let left_type = left.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging>::can_equate(&left_type)?;

        let right_type = right.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging>::can_equate(&right_type)?;

        let _ = left_type.compatible_with(right.as_ref(), &scope.borrow())?;
        resolve_metadata!(metadata, self, scope, context);
        Ok(())
    }
}

impl Resolve for LogicalAnd {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match (self.left.as_ref(), self.right.as_ref()) {
            (Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_)))), value) => {
                let _ = value.resolve(scope, context, extra)?;
                let right_type = self.right.type_of(&scope.borrow())?;
                let _ = self.left.resolve(scope, &Some(right_type), extra)?;
            }
            (value, Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_))))) => {
                let _ = value.resolve(scope, context, extra)?;
                let left_type = self.left.type_of(&scope.borrow())?;
                let _ = self.right.resolve(scope, &Some(left_type), extra)?;
            }
            _ => {
                let _ = self.left.resolve(scope, context, extra)?;
                let _ = self.right.resolve(scope, context, extra)?;
            }
        }

        let left_type = self.left.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging>::can_logical_and(&left_type)?;

        let right_type = self.right.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging>::can_logical_and(&right_type)?;

        let _ = left_type.compatible_with(self.right.as_ref(), &scope.borrow())?;
        resolve_metadata!(self.metadata, self, scope, context);
        Ok(())
    }
}
impl Resolve for LogicalOr {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match (self.left.as_ref(), self.right.as_ref()) {
            (Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_)))), value) => {
                let _ = value.resolve(scope, context, extra)?;
                let right_type = self.right.type_of(&scope.borrow())?;
                let _ = self.left.resolve(scope, &Some(right_type), extra)?;
            }
            (value, Expression::Atomic(Atomic::Data(Data::Primitive(Primitive::Number(_))))) => {
                let _ = value.resolve(scope, context, extra)?;
                let left_type = self.left.type_of(&scope.borrow())?;
                let _ = self.right.resolve(scope, &Some(left_type), extra)?;
            }
            _ => {
                let _ = self.left.resolve(scope, context, extra)?;
                let _ = self.right.resolve(scope, context, extra)?;
            }
        }

        let left_type = self.left.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging>::can_logical_or(&left_type)?;

        let right_type = self.right.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging>::can_logical_or(&right_type)?;

        let _ = left_type.compatible_with(self.right.as_ref(), &scope.borrow())?;
        resolve_metadata!(self.metadata, self, scope, context);
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
            scope_impl::Scope,
            static_types::{FnType, NumberType, PrimitiveType, SliceType, StaticType, StringType},
            var_impl::{Var, VarState},
        },
    };

    use super::*;

    #[test]
    fn valid_high_ord_math() {
        let expr = Product::parse("10 * 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = Product::parse("10.0 * 10.0".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = Product::parse("10 * (10+10)".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = Product::parse("(10+10) * 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = Product::parse("10 * x".into()).unwrap().1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                state: Cell::default(),
                id: "x".into(),
                type_sig: p_num!(I64),
                is_declared: Cell::new(false),
            })
            .unwrap();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_high_ord_math() {
        let expr = Product::parse("10 * 'a'".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());

        let expr = Product::parse("'a' * 'a'".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());

        let expr = Product::parse("10 * x".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());
    }

    #[test]
    fn valid_low_ord_math() {
        let expr = Addition::parse("10 + 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = Addition::parse("10 - 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = Addition::parse("10 + (10*10)".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = Addition::parse("10 + 10*10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = Addition::parse("(10 * 10) + 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = Addition::parse("10 * 10 + 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = Product::parse("10 * 10 + 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = Product::parse("10 * 10 + 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = Addition::parse("10 + x".into()).unwrap().1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                state: Cell::default(),
                id: "x".into(),
                type_sig: p_num!(I64),
                is_declared: Cell::new(false),
            })
            .unwrap();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_low_ord_math() {
        let expr = Addition::parse("10 + 'a'".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());

        let expr = Addition::parse("'a' + 'a'".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());

        let expr = Addition::parse("10 + x".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());

        let expr = Addition::parse("10.0 + 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());
    }

    #[test]
    fn valid_shift() {
        let expr = Shift::parse("10 >> 1".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = Shift::parse("10 << 1".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_shift() {
        let expr = Shift::parse("10 >> 'a'".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());

        let expr = Shift::parse("'a' >> 1".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());
    }

    #[test]
    fn valid_bitwise() {
        let expr = BitwiseAnd::parse("10 & 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = BitwiseOR::parse("10 | 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = BitwiseXOR::parse("10 ^ 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_bitwise() {
        let expr = BitwiseAnd::parse("10 & 'a'".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());

        let expr = BitwiseOR::parse("'a' | 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());

        let expr = BitwiseXOR::parse("10 ^ 'a'".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());
    }

    #[test]
    fn valid_cast() {
        let expr = Cast::parse("10 as f64".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);
        let expr_type = expr.type_of(&scope.borrow()).unwrap();
        assert_eq!(p_num!(F64), expr_type);

        let expr = Cast::parse("'a' as u64".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);
        let expr_type = expr.type_of(&scope.borrow()).unwrap();
        assert_eq!(p_num!(U64), expr_type);

        let expr = Cast::parse("'a' as string".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);
        let expr_type = expr.type_of(&scope.borrow()).unwrap();
        assert_eq!(e_static!(StaticType::String(StringType())), expr_type);

        let expr = Cast::parse("['a'] as string".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);
        let expr_type = expr.type_of(&scope.borrow()).unwrap();
        assert_eq!(e_static!(StaticType::String(StringType())), expr_type);
    }

    #[test]
    fn valid_comparaison() {
        let expr = Expression::parse("10 < 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = Expression::parse("'a' > 'b'".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_comparaison() {
        let expr = Expression::parse("10 > 'a'".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());
    }

    #[test]
    fn valid_equation() {
        let expr = Expression::parse("10 == 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = Expression::parse("'a' != 'b'".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_equation() {
        let expr = Expression::parse("10 == 'a'".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());
    }

    #[test]
    fn valid_and_or() {
        let expr = Expression::parse("true and false".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = Expression::parse("true or false".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = Expression::parse("true and 2 > 3".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = Expression::parse("true and true and true".into())
            .unwrap()
            .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_and_or() {
        let expr = Expression::parse("true and 2".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());

        let expr = Expression::parse("1 or true".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());
    }

    #[test]
    fn valid_unary() {
        let expr = UnaryOperation::parse("!true".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = UnaryOperation::parse("! ( true and false )".into())
            .unwrap()
            .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = UnaryOperation::parse("-1".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = UnaryOperation::parse("-1.0".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = UnaryOperation::parse("- ( 10 + 10 )".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_unary() {
        let expr = UnaryOperation::parse("!10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());

        let expr = UnaryOperation::parse("- true".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());
    }
}

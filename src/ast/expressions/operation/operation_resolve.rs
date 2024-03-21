use super::{
    Addition, BitwiseAnd, BitwiseOR, BitwiseXOR, Cast, Comparaison, Equation, Inclusion,
    LogicalAnd, LogicalOr, Product, Shift, Substraction, UnaryOperation,
};

use crate::ast::expressions::Expression;
use crate::resolve_metadata;
use crate::semantic::scope::static_types::{NumberType, PrimitiveType, StaticType};
use crate::semantic::scope::type_traits::GetSubTypes;
use crate::semantic::scope::user_type_impl::UserType;

use crate::semantic::{
    scope::{type_traits::OperandMerging, ScopeApi},
    CompatibleWith, Either, Resolve, SemanticError, TypeOf,
};
use crate::semantic::{EType, Info, MutRc};
use std::{cell::RefCell, rc::Rc};

impl<Scope: ScopeApi> Resolve<Scope> for UnaryOperation<Scope> {
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
        Scope: ScopeApi,
    {
        match self {
            UnaryOperation::Minus { value, metadata } => {
                // let binding = Some(Either::Static(
                //     StaticType::Primitive(PrimitiveType::Number(NumberType::F64)).into(),
                // ));
                let _ = value.resolve(scope, context, extra)?;
                let value_type = value.type_of(&scope.borrow())?;

                // let _ = <EType as OperandMerging<Scope>>::can_substract(&value_type)?;
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
                let _ = <EType as OperandMerging<Scope>>::can_negate(&value_type)?;
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
impl<Scope: ScopeApi> Resolve<Scope> for Product<Scope> {
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
        Scope: ScopeApi,
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
        let _ = left.resolve(scope, context, extra)?;
        let left_type = left.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging<Scope>>::can_product(&left_type)?;

        let _ = right.resolve(scope, context, extra)?;
        let right_type = right.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging<Scope>>::can_product(&right_type)?;

        // let _ = left_type.compatible_with(right.as_ref(), &scope.borrow())?;
        resolve_metadata!(metadata, self, scope, context);
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Addition<Scope> {
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
        Scope: ScopeApi,
    {
        let _ = self.left.resolve(scope, context, extra)?;
        let left_type = self.left.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging<Scope>>::can_add(&left_type)?;

        let _ = self.right.resolve(scope, context, extra)?;
        let right_type = self.right.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging<Scope>>::can_add(&right_type)?;

        // let _ = left_type.compatible_with(self.right.as_ref(), &scope.borrow())?;
        resolve_metadata!(self.metadata, self, scope, context);
        Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for Substraction<Scope> {
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
        Scope: ScopeApi,
    {
        let _ = self.left.resolve(scope, context, extra)?;
        let left_type = self.left.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging<Scope>>::can_substract(&left_type)?;

        let _ = self.right.resolve(scope, context, extra)?;
        let right_type = self.right.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging<Scope>>::can_substract(&right_type)?;

        // let _ = left_type.compatible_with(self.right.as_ref(), &scope.borrow())?;
        resolve_metadata!(self.metadata, self, scope, context);
        Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for Shift<Scope> {
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
        Scope: ScopeApi,
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
        let _ = left.resolve(scope, context, extra)?;
        let left_type = left.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging<Scope>>::can_shift(&left_type)?;

        let _ = right.resolve(scope, context, extra)?;
        let right_type = right.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging<Scope>>::can_shift(&right_type)?;

        // let _ = left_type.compatible_with(right.as_ref(), &scope.borrow())?;
        resolve_metadata!(metadata, self, scope, context);
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for BitwiseAnd<Scope> {
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
        Scope: ScopeApi,
    {
        let _ = self.left.resolve(scope, context, extra)?;
        let left_type = self.left.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging<Scope>>::can_bitwise_and(&left_type)?;

        let _ = self.right.resolve(scope, context, extra)?;
        let right_type = self.right.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging<Scope>>::can_bitwise_and(&right_type)?;

        // let _ = left_type.compatible_with(self.right.as_ref(), &scope.borrow())?;
        resolve_metadata!(self.metadata, self, scope, context);
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for BitwiseXOR<Scope> {
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
        Scope: ScopeApi,
    {
        let _ = self.left.resolve(scope, context, extra)?;
        let left_type = self.left.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging<Scope>>::can_bitwise_xor(&left_type)?;

        let _ = self.right.resolve(scope, context, extra)?;
        let right_type = self.right.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging<Scope>>::can_bitwise_xor(&right_type)?;

        // let _ = left_type.compatible_with(self.right.as_ref(), &scope.borrow())?;
        resolve_metadata!(self.metadata, self, scope, context);
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for BitwiseOR<Scope> {
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
        Scope: ScopeApi,
    {
        let _ = self.left.resolve(scope, context, extra)?;
        let left_type = self.left.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging<Scope>>::can_bitwise_or(&left_type)?;

        let _ = self.right.resolve(scope, context, extra)?;
        let right_type = self.right.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging<Scope>>::can_bitwise_or(&right_type)?;

        // let _ = left_type.compatible_with(self.right.as_ref(), &scope.borrow())?;
        resolve_metadata!(self.metadata, self, scope, context);
        Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for Cast<Scope> {
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
        Scope: ScopeApi,
    {
        let _ = self.left.resolve(scope, context, extra)?;

        let _ = self.right.resolve(scope, &(), extra)?;
        resolve_metadata!(self.metadata, self, scope, context);
        Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for Comparaison<Scope> {
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
        Scope: ScopeApi,
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
        let _ = left.resolve(scope, context, extra)?;
        let left_type = left.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging<Scope>>::can_comparaison(&left_type)?;

        let _ = right.resolve(scope, context, extra)?;
        let right_type = right.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging<Scope>>::can_comparaison(&right_type)?;

        let _ = left_type.compatible_with(right.as_ref(), &scope.borrow())?;
        resolve_metadata!(metadata, self, scope, context);
        Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for Equation<Scope> {
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
        Scope: ScopeApi,
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
        let _ = left.resolve(scope, context, extra)?;
        let left_type = left.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging<Scope>>::can_equate(&left_type)?;

        let _ = right.resolve(scope, context, extra)?;
        let right_type = right.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging<Scope>>::can_equate(&right_type)?;

        let _ = left_type.compatible_with(right.as_ref(), &scope.borrow())?;
        resolve_metadata!(metadata, self, scope, context);
        Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for Inclusion<Scope> {
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
        Scope: ScopeApi,
    {
        let _ = self.left.resolve(scope, context, extra)?;
        let left_type = self.left.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging<Scope>>::can_include_left(&left_type)?;

        let _ = self.right.resolve(scope, context, extra)?;
        let right_type = self.right.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging<Scope>>::can_include_right(&right_type)?;

        let Some(right_item) = <EType as GetSubTypes>::get_item(&right_type) else {
            return Err(SemanticError::IncompatibleOperands);
        };

        let _ = left_type.compatible_with(&right_item, &scope.borrow())?;
        resolve_metadata!(self.metadata, self, scope, context);
        Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for LogicalAnd<Scope> {
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
        Scope: ScopeApi,
    {
        let _ = self.left.resolve(scope, context, extra)?;
        let left_type = self.left.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging<Scope>>::can_logical_and(&left_type)?;

        let _ = self.right.resolve(scope, context, extra)?;
        let right_type = self.right.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging<Scope>>::can_logical_and(&right_type)?;

        let _ = left_type.compatible_with(self.right.as_ref(), &scope.borrow())?;
        resolve_metadata!(self.metadata, self, scope, context);
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for LogicalOr<Scope> {
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
        Scope: ScopeApi,
    {
        let _ = self.left.resolve(scope, context, extra)?;
        let left_type = self.left.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging<Scope>>::can_logical_or(&left_type)?;

        let _ = self.right.resolve(scope, context, extra)?;
        let right_type = self.right.type_of(&scope.borrow())?;
        let _ = <EType as OperandMerging<Scope>>::can_logical_or(&right_type)?;

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
        semantic::scope::{
            scope_impl::Scope,
            static_types::{NumberType, PrimitiveType, SliceType, StaticType, StringType},
            var_impl::Var,
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
                type_sig: Either::Static(
                    StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                ),
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

        let expr = Product::parse("10 * 10.0".into()).unwrap().1;
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
                type_sig: Either::Static(
                    StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                ),
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
        assert_eq!(
            Either::Static(StaticType::Primitive(PrimitiveType::Number(NumberType::F64)).into()),
            expr_type
        );

        let expr = Cast::parse("'a' as u64".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);
        let expr_type = expr.type_of(&scope.borrow()).unwrap();
        assert_eq!(
            Either::Static(StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into()),
            expr_type
        );

        let expr = Cast::parse("'a' as string".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);
        let expr_type = expr.type_of(&scope.borrow()).unwrap();
        assert_eq!(
            Either::Static(StaticType::String(StringType()).into()),
            expr_type
        );

        let expr = Cast::parse("['a'] as string".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);
        let expr_type = expr.type_of(&scope.borrow()).unwrap();
        assert_eq!(
            Either::Static(StaticType::String(StringType()).into()),
            expr_type
        );

        // let expr = Cast::parse("\"hello\" as [10]char".into()).unwrap().1;
        // let scope = Scope::new();
        // let res = expr.resolve(&scope, &None, &());
        // assert!(res.is_ok(), "{:?}", res);
        // let expr_type = expr.type_of(&scope.borrow()).unwrap();
        // assert_eq!(
        //     Either::Static(
        //         StaticType::Slice(SliceType {
        //             size: 10,
        //             item_type: Box::new(Either::Static(
        //                 StaticType::Primitive(PrimitiveType::Char).into()
        //             ))
        //         })
        //         .into()
        //     ),
        //     expr_type
        // );
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
    fn valid_inclusion() {
        let expr = Expression::parse("10 in [10,2]".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = Expression::parse(r##"'o' in "Hello world""##.into())
            .unwrap()
            .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_inclusion() {
        let expr = Expression::parse("10 in 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());

        let expr = Expression::parse(r##"2 in "Hello world""##.into())
            .unwrap()
            .1;
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

        let expr = Expression::parse("10 in [10,2] and false".into())
            .unwrap()
            .1;
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

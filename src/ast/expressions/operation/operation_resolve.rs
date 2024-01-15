use super::{
    BitwiseAnd, BitwiseOR, BitwiseXOR, Cast, Comparaison, Equation, HighOrdMath, Inclusion,
    LogicalAnd, LogicalOr, LowOrdMath, Shift, UnaryOperation,
};
use crate::semantic::scope::type_traits::GetSubTypes;
use crate::semantic::{
    scope::{type_traits::OperandMerging, ScopeApi},
    CompatibleWith, EitherType, Resolve, SemanticError, TypeOf,
};
use std::{cell::RefCell, rc::Rc};

impl<Scope: ScopeApi> Resolve<Scope> for UnaryOperation<Scope> {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            UnaryOperation::Minus(value) => {
                let _ = value.resolve(scope, context, extra)?;
                let value_type = value.type_of(&scope.borrow())?;
                let _ = value_type.can_minus()?;
                Ok(())
            }
            UnaryOperation::Not(value) => {
                let _ = value.resolve(scope, context, extra)?;
                let value_type = value.type_of(&scope.borrow())?;
                let _ = value_type.can_negate()?;
                Ok(())
            }
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for HighOrdMath<Scope> {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let (left, right) = match self {
            HighOrdMath::Mult { left, right } => (left, right),
            HighOrdMath::Div { left, right } => (left, right),
            HighOrdMath::Mod { left, right } => (left, right),
        };
        let _ = left.resolve(scope, context, extra)?;
        let left_type = left.type_of(&scope.borrow())?;
        let _ = left_type.can_high_ord_math()?;

        let _ = right.resolve(scope, context, extra)?;
        let right_type = right.type_of(&scope.borrow())?;
        let _ = right_type.can_high_ord_math()?;

        let _ = left_type.compatible_with(right, &scope.borrow())?;
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for LowOrdMath<Scope> {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let (left, right) = match self {
            LowOrdMath::Add { left, right } => (left, right),
            LowOrdMath::Minus { left, right } => (left, right),
        };
        let _ = left.resolve(scope, context, extra)?;
        let left_type = left.type_of(&scope.borrow())?;
        let _ = left_type.can_low_ord_math()?;

        let _ = right.resolve(scope, context, extra)?;
        let right_type = right.type_of(&scope.borrow())?;
        let _ = right_type.can_low_ord_math()?;

        let _ = left_type.compatible_with(right, &scope.borrow())?;
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Shift<Scope> {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let (left, right) = match self {
            Shift::Left { left, right } => (left, right),
            Shift::Right { left, right } => (left, right),
        };
        let _ = left.resolve(scope, context, extra)?;
        let left_type = left.type_of(&scope.borrow())?;
        let _ = left_type.can_shift()?;

        let _ = right.resolve(scope, context, extra)?;
        let right_type = right.type_of(&scope.borrow())?;
        let _ = right_type.can_shift()?;

        let _ = left_type.compatible_with(right, &scope.borrow())?;
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for BitwiseAnd<Scope> {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.left.resolve(scope, context, extra)?;
        let left_type = self.left.type_of(&scope.borrow())?;
        let _ = left_type.can_bitwise_and()?;

        let _ = self.right.resolve(scope, context, extra)?;
        let right_type = self.right.type_of(&scope.borrow())?;
        let _ = right_type.can_bitwise_and()?;

        let _ = left_type.compatible_with(&self.right, &scope.borrow())?;
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for BitwiseXOR<Scope> {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.left.resolve(scope, context, extra)?;
        let left_type = self.left.type_of(&scope.borrow())?;
        let _ = left_type.can_bitwise_xor()?;

        let _ = self.right.resolve(scope, context, extra)?;
        let right_type = self.right.type_of(&scope.borrow())?;
        let _ = right_type.can_bitwise_xor()?;

        let _ = left_type.compatible_with(&self.right, &scope.borrow())?;
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for BitwiseOR<Scope> {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.left.resolve(scope, context, extra)?;
        let left_type = self.left.type_of(&scope.borrow())?;
        let _ = left_type.can_bitwise_or()?;

        let _ = self.right.resolve(scope, context, extra)?;
        let right_type = self.right.type_of(&scope.borrow())?;
        let _ = right_type.can_bitwise_or()?;

        let _ = left_type.compatible_with(&self.right, &scope.borrow())?;
        Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for Cast<Scope> {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.left.resolve(scope, context, extra)?;

        let _ = self.right.resolve(scope, &(), extra)?;

        Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for Comparaison<Scope> {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let (left, right) = match self {
            Comparaison::Less { left, right } => (left, right),
            Comparaison::LessEqual { left, right } => (left, right),
            Comparaison::Greater { left, right } => (left, right),
            Comparaison::GreaterEqual { left, right } => (left, right),
        };
        let _ = left.resolve(scope, context, extra)?;
        let left_type = left.type_of(&scope.borrow())?;
        let _ = left_type.can_comparaison()?;

        let _ = right.resolve(scope, context, extra)?;
        let right_type = right.type_of(&scope.borrow())?;
        let _ = right_type.can_comparaison()?;

        let _ = left_type.compatible_with(right, &scope.borrow())?;
        Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for Equation<Scope> {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let (left, right) = match self {
            Equation::Equal { left, right } => (left, right),
            Equation::NotEqual { left, right } => (left, right),
        };
        let _ = left.resolve(scope, context, extra)?;
        let left_type = left.type_of(&scope.borrow())?;
        let _ = left_type.can_equate()?;

        let _ = right.resolve(scope, context, extra)?;
        let right_type = right.type_of(&scope.borrow())?;
        let _ = right_type.can_equate()?;

        let _ = left_type.compatible_with(right, &scope.borrow())?;
        Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for Inclusion<Scope> {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.left.resolve(scope, context, extra)?;
        let left_type = self.left.type_of(&scope.borrow())?;
        let _ = left_type.can_include_left()?;

        let _ = self.right.resolve(scope, context, extra)?;
        let right_type = self.right.type_of(&scope.borrow())?;
        let _ = right_type.can_include_right()?;

        let Some(right_item) = <EitherType<
            <Scope as ScopeApi>::UserType,
            <Scope as ScopeApi>::StaticType,
        > as GetSubTypes<Scope>>::get_item(&right_type) else {
            return Err(SemanticError::IncompatibleOperands);
        };

        let _ = left_type.compatible_with(&right_item, &scope.borrow())?;

        Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for LogicalAnd<Scope> {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.left.resolve(scope, context, extra)?;
        let left_type = self.left.type_of(&scope.borrow())?;
        let _ = left_type.can_logical_and()?;

        let _ = self.right.resolve(scope, context, extra)?;
        let right_type = self.right.type_of(&scope.borrow())?;
        let _ = right_type.can_logical_and()?;

        let _ = left_type.compatible_with(&self.right, &scope.borrow())?;
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for LogicalOr<Scope> {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.left.resolve(scope, context, extra)?;
        let left_type = self.left.type_of(&scope.borrow())?;
        let _ = left_type.can_logical_or()?;

        let _ = self.right.resolve(scope, context, extra)?;
        let right_type = self.right.type_of(&scope.borrow())?;
        let _ = right_type.can_logical_or()?;

        let _ = left_type.compatible_with(&self.right, &scope.borrow())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{
            expressions::{operation::operation_parse::TryParseOperation, Expression},
            TryParse,
        },
        semantic::scope::{
            scope_impl::Scope,
            static_types::{PrimitiveType, SliceType, StaticType},
            var_impl::Var,
        },
    };

    use super::*;

    #[test]
    fn valid_high_ord_math() {
        let expr = HighOrdMath::parse("10 * 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let expr = HighOrdMath::parse("10.0 * 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let expr = HighOrdMath::parse("10 * 10.0".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let expr = LowOrdMath::parse("10 * (10+10)".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let expr = LowOrdMath::parse("(10+10) * 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let expr = HighOrdMath::parse("10 * x".into()).unwrap().1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                id: "x".into(),
                type_sig: EitherType::Static(StaticType::Primitive(PrimitiveType::Number).into()),
            })
            .unwrap();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok());
    }

    #[test]
    fn robustness_high_ord_math() {
        let expr = HighOrdMath::parse("10 * 'a'".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());

        let expr = HighOrdMath::parse("'a' * 'a'".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());

        let expr = HighOrdMath::parse("10 * x".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());
    }

    #[test]
    fn valid_low_ord_math() {
        let expr = LowOrdMath::parse("10 + 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let expr = LowOrdMath::parse("10 - 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let expr = LowOrdMath::parse("10 + (10*10)".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let expr = LowOrdMath::parse("10 + 10*10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let expr = LowOrdMath::parse("10.0 + 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let expr = LowOrdMath::parse("(10 * 10) + 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let expr = LowOrdMath::parse("10 * 10 + 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let expr = HighOrdMath::parse("10.0 * 10 + 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let expr = HighOrdMath::parse("10 * 10.0 + 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let expr = LowOrdMath::parse("10 + x".into()).unwrap().1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                id: "x".into(),
                type_sig: EitherType::Static(StaticType::Primitive(PrimitiveType::Number).into()),
            })
            .unwrap();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok());
    }

    #[test]
    fn robustness_low_ord_math() {
        let expr = LowOrdMath::parse("10 + 'a'".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());

        let expr = LowOrdMath::parse("'a' + 'a'".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());

        let expr = LowOrdMath::parse("10 + x".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());
    }

    #[test]
    fn valid_shift() {
        let expr = Shift::parse("10 >> 1".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let expr = Shift::parse("10 << 1".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok());
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
        assert!(res.is_ok());

        let expr = BitwiseOR::parse("10 | 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let expr = BitwiseXOR::parse("10 ^ 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok());
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
        let expr = Cast::parse("10 as float".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok());
        let expr_type = expr.type_of(&scope.borrow()).unwrap();
        assert_eq!(
            EitherType::Static(StaticType::Primitive(PrimitiveType::Float).into()),
            expr_type
        );

        let expr = Cast::parse("'a' as number".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok());
        let expr_type = expr.type_of(&scope.borrow()).unwrap();
        assert_eq!(
            EitherType::Static(StaticType::Primitive(PrimitiveType::Number).into()),
            expr_type
        );

        let expr = Cast::parse("'a' as string".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok());
        let expr_type = expr.type_of(&scope.borrow()).unwrap();
        assert_eq!(
            EitherType::Static(StaticType::Slice(SliceType::String).into()),
            expr_type
        );

        let expr = Cast::parse("['a'] as string".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok());
        let expr_type = expr.type_of(&scope.borrow()).unwrap();
        assert_eq!(
            EitherType::Static(StaticType::Slice(SliceType::String).into()),
            expr_type
        );

        let expr = Cast::parse("\"hello\" as [10]char".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok());
        let expr_type = expr.type_of(&scope.borrow()).unwrap();
        assert_eq!(
            EitherType::Static(
                StaticType::Slice(SliceType::List(
                    10,
                    Box::new(EitherType::Static(
                        StaticType::Primitive(PrimitiveType::Char).into()
                    ))
                ))
                .into()
            ),
            expr_type
        );
    }

    #[test]
    fn valid_comparaison() {
        let expr = Expression::parse("10 < 10".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let expr = Expression::parse("'a' > 'b'".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok());
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
        assert!(res.is_ok());

        let expr = Expression::parse("'a' != 'b'".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok());
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
        assert!(res.is_ok());

        let expr = Expression::parse(r##"'o' in "Hello world""##.into())
            .unwrap()
            .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok());
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
        assert!(res.is_ok());

        let expr = Expression::parse("true or false".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let expr = Expression::parse("10 in [10,2] and false".into())
            .unwrap()
            .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let expr = Expression::parse("true and 2 > 3".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let expr = Expression::parse("true and true and true".into())
            .unwrap()
            .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok());
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
        assert!(res.is_ok());

        let expr = UnaryOperation::parse("! ( true and false )".into())
            .unwrap()
            .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let expr = UnaryOperation::parse("-1".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let expr = UnaryOperation::parse("-1.0".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let expr = UnaryOperation::parse("- ( 10 + 10 )".into()).unwrap().1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok());
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

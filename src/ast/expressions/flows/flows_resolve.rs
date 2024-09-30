use super::{
    Cases, EnumCase, ExprFlow, IfExpr, MatchExpr, PrimitiveCase, StringCase, TryExpr, UnionCase,
    UnionPattern,
};
use crate::ast::expressions::Atomic;
use crate::ast::statements::block::BlockCommonApi;
use crate::ast::TryParse;
use crate::p_num;
use crate::semantic::scope::scope::ScopeState;
use crate::semantic::scope::static_types::{PrimitiveType, TupleType, POINTER_SIZE};
use crate::semantic::scope::user_types::{Enum, Struct, Union};
use crate::semantic::{
    scope::{static_types::StaticType, user_types::UserType},
    CompatibleWith, EType, Resolve, SemanticError, TypeOf,
};
use crate::semantic::{Desugar, Info, ResolveNumber, SizeOf};
use crate::vm::GenerateCode;
use std::collections::HashSet;
use std::fmt::Debug;

impl Resolve for ExprFlow {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match self {
            ExprFlow::If(value) => value.resolve::<E>(scope_manager, scope_id, context, extra),
            ExprFlow::Match(value) => value.resolve::<E>(scope_manager, scope_id, context, extra),
            ExprFlow::Try(value) => value.resolve::<E>(scope_manager, scope_id, context, extra),
            ExprFlow::SizeOf(value, metadata) => {
                let _ = value.resolve::<E>(scope_manager, scope_id, &(), extra);
                metadata.info = Info::Resolved {
                    context: context.clone(),
                    signature: Some(p_num!(U64)),
                };
                Ok(())
            }
        }
    }
}

impl ResolveNumber for ExprFlow {
    fn is_unresolved_number(&self) -> bool {
        match self {
            ExprFlow::If(if_expr) => if_expr.is_unresolved_number(),
            ExprFlow::Match(match_expr) => match_expr.is_unresolved_number(),
            ExprFlow::Try(try_expr) => try_expr.is_unresolved_number(),
            ExprFlow::SizeOf(_, metadata) => false,
        }
    }

    fn resolve_number(
        &mut self,
        to: crate::semantic::scope::static_types::NumberType,
    ) -> Result<(), SemanticError> {
        match self {
            ExprFlow::If(if_expr) => if_expr.resolve_number(to),
            ExprFlow::Match(match_expr) => match_expr.resolve_number(to),
            ExprFlow::Try(try_expr) => try_expr.resolve_number(to),
            ExprFlow::SizeOf(_, metadata) => Ok(()),
        }
    }
}

impl Desugar<Atomic> for ExprFlow {
    fn desugar<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<Atomic>, SemanticError> {
        match self {
            ExprFlow::If(value) => value.desugar::<E>(scope_manager, scope_id),
            ExprFlow::Match(value) => value.desugar::<E>(scope_manager, scope_id),
            ExprFlow::Try(value) => value.desugar::<E>(scope_manager, scope_id),
            ExprFlow::SizeOf(value, metadata) => Ok(None),
        }
    }
}

impl Resolve for IfExpr {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let _ = self
            .condition
            .resolve::<E>(scope_manager, scope_id, context, &mut None)?;
        // Check if condition is a boolean
        let EType::Static(StaticType::Primitive(PrimitiveType::Bool)) =
            self.condition.type_of(&scope_manager, scope_id)?
        else {
            return Err(SemanticError::ExpectedBoolean);
        };

        let _ = self
            .then_branch
            .resolve::<E>(scope_manager, scope_id, context, &mut ())?;
        let _ = self
            .else_branch
            .resolve::<E>(scope_manager, scope_id, context, &mut ())?;

        let then_branch_type = self.then_branch.type_of(&scope_manager, scope_id)?;
        let _ = then_branch_type.compatible_with(
            &self.else_branch.type_of(scope_manager, scope_id)?,
            &scope_manager,
            scope_id,
        )?;
        self.metadata.info = crate::semantic::Info::Resolved {
            context: context.clone(),
            signature: Some(self.type_of(scope_manager, scope_id)?),
        };
        Ok(())
    }
}

impl Desugar<Atomic> for IfExpr {
    fn desugar<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<Atomic>, SemanticError> {
        if let Some(output) = self.condition.desugar::<E>(scope_manager, scope_id)? {
            self.condition = output.into();
        }
        if let Some(output) = self.then_branch.desugar::<E>(scope_manager, scope_id)? {
            self.then_branch = output;
        }
        if let Some(output) = self.else_branch.desugar::<E>(scope_manager, scope_id)? {
            self.else_branch = output;
        }
        Ok(None)
    }
}

impl ResolveNumber for IfExpr {
    fn is_unresolved_number(&self) -> bool {
        self.condition.is_unresolved_number()
    }

    fn resolve_number(
        &mut self,
        to: crate::semantic::scope::static_types::NumberType,
    ) -> Result<(), SemanticError> {
        self.condition.resolve_number(to)
    }
}

impl Resolve for UnionPattern {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let UserType::Union(union_type @ Union { .. }) = scope_manager
            .find_type_by_name(&self.typename, scope_id)?
            .def
        else {
            return Err(SemanticError::IncompatibleTypes);
        };

        let union_size = union_type.size_of();
        let variants = union_type.variants;

        let Some((variant_value, (_, struct_type @ Struct { .. }))) = variants
            .iter()
            .enumerate()
            .find(|(i, (variant_name, variant_struct))| *variant_name == self.variant)
        else {
            return Err(SemanticError::CantInferType(format!(
                "of {}::{}",
                self.typename, self.variant
            )));
        };

        let struct_size = struct_type.size_of();

        let fields = &struct_type.fields;

        let _ = self.variant_value.insert(variant_value as u64);

        let _ = self.variant_padding.insert(
            union_size
                .checked_sub(struct_size + POINTER_SIZE)
                .unwrap_or(0),
        );

        if self.vars_names.len() != fields.len() {
            return Err(SemanticError::InvalidPattern);
        }

        let ids = self
            .vars_id
            .insert(Vec::with_capacity(self.vars_names.len()));

        let is_scope_iife = scope_id.is_some()
            && *scope_manager
                .scope_states
                .get(&scope_id.unwrap())
                .unwrap_or(&ScopeState::Inline)
                == ScopeState::IIFE;

        for (field_name, field_type) in fields.iter() {
            let id: u64;
            if is_scope_iife {
                // the block is an IIFE
                id = scope_manager.register_parameter(&field_name, field_type.clone(), scope_id)?;
            } else {
                id = scope_manager.register_var(&field_name, field_type.clone(), scope_id)?;
            }

            ids.push(id);
        }

        Ok(())
    }
}

impl<
        B: TryParse
            + Resolve<Context = Option<EType>, Extra = ()>
            + GenerateCode
            + BlockCommonApi
            + Clone
            + Debug
            + PartialEq,
    > Resolve for PrimitiveCase<B>
{
    type Output = ();
    type Context = Option<EType>;
    type Extra = Option<EType>;
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let inner_scope = self.block.init_from_parent(scope_manager, scope_id)?;

        for pattern in &mut self.patterns {
            let _ = pattern.resolve::<E>(scope_manager, Some(inner_scope), &extra, &mut ())?;
        }

        let _ = self
            .block
            .resolve::<E>(scope_manager, scope_id, context, &mut ())?;
        Ok(())
    }
}

impl<
        B: TryParse
            + Resolve<Context = Option<EType>, Extra = ()>
            + GenerateCode
            + BlockCommonApi
            + Clone
            + Debug
            + PartialEq,
    > Resolve for StringCase<B>
{
    type Output = ();
    type Context = Option<EType>;
    type Extra = Option<EType>;
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let inner_scope = self.block.init_from_parent(scope_manager, scope_id)?;

        for pattern in &mut self.patterns {
            let _ = pattern.resolve::<E>(scope_manager, Some(inner_scope), &extra, &mut ())?;
        }

        let _ = self
            .block
            .resolve::<E>(scope_manager, scope_id, context, &mut ())?;
        Ok(())
    }
}

impl<
        B: TryParse
            + Resolve<Context = Option<EType>, Extra = ()>
            + GenerateCode
            + BlockCommonApi
            + Clone
            + Debug
            + PartialEq,
    > Resolve for EnumCase<B>
{
    type Output = ();
    type Context = Option<EType>;
    type Extra = Option<EType>;
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let inner_scope = self.block.init_from_parent(scope_manager, scope_id)?;

        for (ref typename, ref name, value) in self.patterns.iter_mut() {
            let UserType::Enum(Enum { id, values }) =
                scope_manager.find_type_by_name(typename, scope_id)?.def
            else {
                return Err(SemanticError::IncompatibleTypes);
            };

            let Some((idx, _)) = values.iter().enumerate().find(|(idx, v)| *v == name) else {
                return Err(SemanticError::IncorrectVariant(typename.to_string()));
            };
            value.insert(idx as u64);
        }

        let _ = self
            .block
            .resolve::<E>(scope_manager, scope_id, context, &mut ())?;
        Ok(())
    }
}

impl<
        B: TryParse
            + Resolve<Context = Option<EType>, Extra = ()>
            + GenerateCode
            + BlockCommonApi
            + Clone
            + Debug
            + PartialEq,
    > Resolve for UnionCase<B>
{
    type Output = ();
    type Context = Option<EType>;
    type Extra = Option<EType>;
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let inner_scope = self.block.init_from_parent(scope_manager, scope_id)?;

        if scope_manager.is_scope_global(Some(inner_scope)) {
            scope_manager
                .scope_states
                .insert(inner_scope, ScopeState::IIFE);
        } else {
            scope_manager
                .scope_states
                .insert(inner_scope, ScopeState::Inline);
        }
        let _ = self
            .pattern
            .resolve::<E>(scope_manager, Some(inner_scope), &extra, &mut ())?;

        let _ = self
            .block
            .resolve::<E>(scope_manager, scope_id, context, &mut ())?;
        Ok(())
    }
}

impl Resolve for MatchExpr {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let _ = self
            .expr
            .resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
        let expr_type = self.expr.type_of(&scope_manager, scope_id)?;

        let should_be_exhaustive = self.else_branch.is_none();

        match &mut self.cases {
            super::Cases::Primitive { cases } => {
                let EType::Static(StaticType::Primitive(_)) = expr_type else {
                    return Err(SemanticError::IncompatibleTypes);
                };
                let mut current_case_type: Option<EType> = None;
                for case in cases {
                    let _ = case.resolve::<E>(
                        scope_manager,
                        scope_id,
                        context,
                        &mut Some(expr_type.clone()),
                    )?;
                    let case_type = case.block.type_of(scope_manager, scope_id)?;
                    if let Some(current_case_type) = &current_case_type {
                        let _ = current_case_type.compatible_with(
                            &case_type,
                            scope_manager,
                            scope_id,
                        )?;
                    }
                    let _ = current_case_type.insert(case_type);
                }
            }
            super::Cases::String { cases } => {
                match expr_type {
                    EType::Static(StaticType::StrSlice(_))
                    | EType::Static(StaticType::String(_)) => {}
                    _ => return Err(SemanticError::IncompatibleTypes),
                }

                let mut current_case_type: Option<EType> = None;
                for case in cases {
                    let _ = case.resolve::<E>(
                        scope_manager,
                        scope_id,
                        context,
                        &mut Some(expr_type.clone()),
                    )?;
                    let case_type = case.block.type_of(scope_manager, scope_id)?;
                    if let Some(current_case_type) = &current_case_type {
                        let _ = current_case_type.compatible_with(
                            &case_type,
                            scope_manager,
                            scope_id,
                        )?;
                    }
                    let _ = current_case_type.insert(case_type);
                }
            }
            super::Cases::Enum { cases } => {
                let EType::User { id, size } = expr_type else {
                    return Err(SemanticError::IncompatibleTypes);
                };
                let UserType::Enum(Enum { id, values }) =
                    scope_manager.find_type_by_id(id, scope_id)?.clone()
                else {
                    return Err(SemanticError::IncompatibleTypes);
                };

                let mut current_case_type: Option<EType> = None;
                for case in cases.iter_mut() {
                    let _ = case.resolve::<E>(
                        scope_manager,
                        scope_id,
                        context,
                        &mut Some(expr_type.clone()),
                    )?;
                    let case_type = case.block.type_of(scope_manager, scope_id)?;
                    if let Some(current_case_type) = &current_case_type {
                        let _ = current_case_type.compatible_with(
                            &case_type,
                            scope_manager,
                            scope_id,
                        )?;
                    }
                    let _ = current_case_type.insert(case_type);
                }

                if should_be_exhaustive {
                    let mut found_names = HashSet::new();
                    for case in cases {
                        for (_, name, _) in &case.patterns {
                            found_names.insert(name.clone());
                        }
                    }
                    if found_names.len() != values.len() {
                        let names: HashSet<String> = values.clone().into_iter().collect();
                        let difference: HashSet<_> =
                            names.difference(&found_names).cloned().collect();

                        return Err(SemanticError::ExhaustiveCases(difference));
                    }
                }
            }
            super::Cases::Union { cases } => {
                let EType::User { id, size } = expr_type else {
                    return Err(SemanticError::IncompatibleTypes);
                };
                let UserType::Union(Union { id, variants }) =
                    scope_manager.find_type_by_id(id, scope_id)?.clone()
                else {
                    return Err(SemanticError::IncompatibleTypes);
                };

                let mut current_case_type: Option<EType> = None;
                for case in cases.iter_mut() {
                    let _ = case.resolve::<E>(
                        scope_manager,
                        scope_id,
                        context,
                        &mut Some(expr_type.clone()),
                    )?;
                    let case_type = case.block.type_of(scope_manager, scope_id)?;
                    if let Some(current_case_type) = &current_case_type {
                        let _ = current_case_type.compatible_with(
                            &case_type,
                            scope_manager,
                            scope_id,
                        )?;
                    }
                    let _ = current_case_type.insert(case_type);
                }

                if should_be_exhaustive {
                    let mut found_names = HashSet::new();
                    for case in cases {
                        found_names.insert(case.pattern.variant.clone());
                    }
                    if found_names.len() != variants.len() {
                        let names: HashSet<String> =
                            variants.clone().into_iter().map(|v| v.0).collect();
                        let difference: HashSet<_> =
                            names.difference(&found_names).cloned().collect();

                        return Err(SemanticError::ExhaustiveCases(difference));
                    }
                }
            }
        }

        if let Some(block) = self.else_branch.as_mut() {
            let _ = block.resolve::<E>(scope_manager, scope_id, context, extra)?;
        }

        self.metadata.info = crate::semantic::Info::Resolved {
            context: context.clone(),
            signature: Some(self.type_of(scope_manager, scope_id)?),
        };
        Ok(())
    }
}

impl ResolveNumber for MatchExpr {
    fn is_unresolved_number(&self) -> bool {
        self.expr.is_unresolved_number()
    }

    fn resolve_number(
        &mut self,
        to: crate::semantic::scope::static_types::NumberType,
    ) -> Result<(), SemanticError> {
        self.expr.resolve_number(to)
    }
}

impl<
        B: TryParse
            + Resolve<Context = Option<EType>, Extra = ()>
            + GenerateCode
            + BlockCommonApi
            + Desugar<B>
            + Clone
            + Debug
            + PartialEq,
        C: TryParse
            + Resolve<Context = Option<EType>, Extra = ()>
            + GenerateCode
            + BlockCommonApi
            + Desugar<C>
            + Clone
            + Debug
            + PartialEq,
    > Desugar<Cases<B, C>> for Cases<B, C>
{
    fn desugar<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<Cases<B, C>>, SemanticError> {
        match self {
            Cases::Primitive { cases } => {
                for case in cases.iter_mut() {
                    if let Some(output) = case.block.desugar::<E>(scope_manager, scope_id)? {
                        case.block = output;
                    }
                }
            }
            Cases::String { cases } => {
                for case in cases.iter_mut() {
                    if let Some(output) = case.block.desugar::<E>(scope_manager, scope_id)? {
                        case.block = output;
                    }
                }
            }
            Cases::Enum { cases } => {
                for case in cases.iter_mut() {
                    if let Some(output) = case.block.desugar::<E>(scope_manager, scope_id)? {
                        case.block = output;
                    }
                }
            }
            Cases::Union { cases } => {
                for case in cases.iter_mut() {
                    if let Some(output) = case.block.desugar::<E>(scope_manager, scope_id)? {
                        case.block = output;
                    }
                }
            }
        }
        Ok(None)
    }
}

impl Desugar<Atomic> for MatchExpr {
    fn desugar<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<Atomic>, SemanticError> {
        let _ = self.cases.desugar::<E>(scope_manager, scope_id)?;
        if let Some(output) = self.expr.desugar::<E>(scope_manager, scope_id)? {
            self.expr = output.into();
        }
        Ok(None)
    }
}

impl Resolve for TryExpr {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        _extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let _ = self
            .try_branch
            .resolve::<E>(scope_manager, scope_id, context, &mut ())?;

        if let Some(block) = &mut self.else_branch {
            block.resolve::<E>(scope_manager, scope_id, context, &mut ())?;
        }

        let mut try_branch_type = self.try_branch.type_of(&scope_manager, scope_id)?;

        let else_branch_type = self
            .else_branch
            .as_ref()
            .map(|block| block.type_of(scope_manager, scope_id))
            .unwrap_or(Ok(EType::Static(StaticType::Unit)))?;

        if let EType::Static(StaticType::Tuple(TupleType(tuple_type))) = &mut try_branch_type {
            if let Some(EType::Static(StaticType::Error)) = tuple_type.last() {
                self.pop_last_err = true;
                tuple_type.pop();
            }
            if tuple_type.len() == 1 {
                try_branch_type = tuple_type[0].clone();
            }
        } else if let EType::Static(StaticType::Error) = try_branch_type {
            self.pop_last_err = true;
            try_branch_type = EType::Static(StaticType::Unit);
        } else if self.else_branch.is_none() && EType::Static(StaticType::Unit) != try_branch_type {
            return Err(SemanticError::IncompatibleTypes);
        }

        let _ = try_branch_type.compatible_with(&else_branch_type, &scope_manager, scope_id)?;

        self.metadata.info = Info::Resolved {
            context: context.clone(),
            signature: Some(try_branch_type),
        };

        Ok(())
    }
}

impl ResolveNumber for TryExpr {
    fn is_unresolved_number(&self) -> bool {
        false
    }

    fn resolve_number(
        &mut self,
        to: crate::semantic::scope::static_types::NumberType,
    ) -> Result<(), SemanticError> {
        Ok(())
    }
}

impl Desugar<Atomic> for TryExpr {
    fn desugar<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<Atomic>, SemanticError> {
        if let Some(output) = self.try_branch.desugar::<E>(scope_manager, scope_id)? {
            self.try_branch = output.into();
        }
        if let Some(else_block) = &mut self.else_branch {
            if let Some(output) = else_block.desugar::<E>(scope_manager, scope_id)? {
                *else_block = output.into();
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {

    use crate::ast::TryParse;

    use super::*;

    #[test]
    fn valid_if() {
        let mut expr = IfExpr::parse("if true then {10} else {20}".into())
            .expect("Parsing should have succeeded")
            .1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_if() {
        let mut expr = IfExpr::parse("if 10 then {10} else {20}".into())
            .expect("Parsing should have succeeded")
            .1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_err());

        let mut expr = IfExpr::parse("if true then {10} else {'a'}".into())
            .expect("Parsing should have succeeded")
            .1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_err());
    }

    #[test]
    fn valid_try() {
        let mut expr = TryExpr::parse("try {10} else {20}".into())
            .expect("Parsing should have succeeded")
            .1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn valid_try_no_else() {
        let mut expr = TryExpr::parse("try {Ok()}".into())
            .expect("Parsing should have succeeded")
            .1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res = expr.resolve::<crate::vm::external::test::NoopEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);
    }
    #[test]
    fn valid_try_tuple_err() {
        let mut expr = TryExpr::parse("try {(10,Err())} else {20}".into())
            .expect("Parsing should have succeeded")
            .1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let _ = expr
            .resolve::<crate::vm::external::test::NoopEngine>(
                &mut scope_manager,
                None,
                &None,
                &mut (),
            )
            .expect("Resolutionb should have succeeded");
    }

    #[test]
    fn valid_try_tuple_multi_err() {
        let mut expr = TryExpr::parse("try {(10,20,Err())} else {(20,30)}".into())
            .expect("Parsing should have succeeded")
            .1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let _ = expr
            .resolve::<crate::vm::external::test::NoopEngine>(
                &mut scope_manager,
                None,
                &None,
                &mut (),
            )
            .expect("Resolutionb should have succeeded");
    }
    #[test]
    fn robustness_try_tuple_err() {
        let mut expr = TryExpr::parse("try {(10,20,Err())} else {20}".into())
            .expect("Parsing should have succeeded")
            .1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let _ = expr
            .resolve::<crate::vm::external::test::NoopEngine>(
                &mut scope_manager,
                None,
                &None,
                &mut (),
            )
            .expect_err("Resolution shoud have failed");
    }
    #[test]
    fn robustness_try_tuple_err_no_else() {
        let mut expr = TryExpr::parse("try {(10,Err())}".into())
            .expect("Parsing should have succeeded")
            .1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let _ = expr
            .resolve::<crate::vm::external::test::NoopEngine>(
                &mut scope_manager,
                None,
                &None,
                &mut (),
            )
            .expect_err("Resolution shoud have failed");
    }
}

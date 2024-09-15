use super::{CallStat, Flow, IfStat, MatchStat, TryStat};
use crate::semantic::scope::scope::ScopeManager;
use crate::semantic::scope::static_types::PrimitiveType;
use crate::semantic::{
    scope::{
        static_types::StaticType,
        type_traits::TypeChecking,
        user_type_impl::{Enum, Union, UserType},
    },
    EType, Resolve, SemanticError, TypeOf,
};
use std::collections::{HashMap, HashSet};

impl Resolve for Flow {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
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
            Flow::If(value) => value.resolve::<G>(scope_manager, scope_id, context, extra),
            Flow::Match(value) => value.resolve::<G>(scope_manager, scope_id, context, extra),
            Flow::Try(value) => value.resolve::<G>(scope_manager, scope_id, context, extra),
            Flow::Call(value) => value.resolve::<G>(scope_manager, scope_id, &(), &mut ()),
        }
    }
}
impl Resolve for IfStat {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
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
            .resolve::<G>(scope_manager, scope_id, &None, &mut None)?;
        // check that condition is a boolean
        let condition_type = self.condition.type_of(&scope_manager, scope_id)?;
        if EType::Static(StaticType::Primitive(PrimitiveType::Bool)) != condition_type {
            return Err(SemanticError::ExpectedBoolean);
        }

        let _ = self
            .then_branch
            .resolve::<G>(scope_manager, scope_id, &context, &mut ())?;

        for (else_if_cond, else_if_scope) in &mut self.else_if_branches {
            let _ = else_if_cond.resolve::<G>(scope_manager, scope_id, &None, &mut None)?;
            let condition_type = else_if_cond.type_of(&scope_manager, scope_id)?;
            if EType::Static(StaticType::Primitive(PrimitiveType::Bool)) != condition_type {
                return Err(SemanticError::ExpectedBoolean);
            }
            let _ = else_if_scope.resolve::<G>(scope_manager, scope_id, &context, &mut ())?;
        }

        if let Some(else_branch) = &mut self.else_branch {
            let _ = else_branch.resolve::<G>(scope_manager, scope_id, &context, &mut ())?;
        }

        Ok(())
    }
}
impl Resolve for MatchStat {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
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
            .resolve::<G>(scope_manager, scope_id, &None, &mut None)?;
        let expr_type = self.expr.type_of(&scope_manager, scope_id)?;

        let should_be_exhaustive = self.else_branch.is_none();

        match &mut self.cases {
            super::Cases::Primitive { cases } => {
                let EType::Static(StaticType::Primitive(_)) = expr_type else {
                    return Err(SemanticError::IncompatibleTypes);
                };
                for case in cases {
                    let _ = case.resolve::<G>(
                        scope_manager,
                        scope_id,
                        context,
                        &mut Some(expr_type.clone()),
                    )?;
                }
            }
            super::Cases::String { cases } => {
                match expr_type {
                    EType::Static(StaticType::StrSlice(_))
                    | EType::Static(StaticType::String(_)) => {}
                    _ => return Err(SemanticError::IncompatibleTypes),
                }

                for case in cases {
                    let _ = case.resolve::<G>(
                        scope_manager,
                        scope_id,
                        context,
                        &mut Some(expr_type.clone()),
                    )?;
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

                for case in cases.iter_mut() {
                    let _ = case.resolve::<G>(
                        scope_manager,
                        scope_id,
                        context,
                        &mut Some(expr_type.clone()),
                    )?;
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

                for case in cases.iter_mut() {
                    let _ = case.resolve::<G>(
                        scope_manager,
                        scope_id,
                        context,
                        &mut Some(expr_type.clone()),
                    )?;
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
            let _ = block.resolve::<G>(scope_manager, scope_id, context, extra)?;
        }
        Ok(())
    }
}

impl Resolve for TryStat {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
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
            .resolve::<G>(scope_manager, scope_id, &context, &mut ())?;
        if let Some(else_branch) = &mut self.else_branch {
            let _ = else_branch.resolve::<G>(scope_manager, scope_id, &context, &mut ())?;
        }
        Ok(())
    }
}
impl Resolve for CallStat {
    type Output = ();
    type Context = ();
    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        _context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        self.call
            .resolve::<G>(scope_manager, scope_id, &None, &mut None)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::ast::TryParse;
    use crate::p_num;
    use crate::semantic::scope::scope::ScopeManager;

    #[test]
    fn valid_if() {
        let mut expr = IfStat::parse(
            r##"
            if true {
                let x = 1;
            }
        "##
            .into(),
        )
        .unwrap()
        .1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res =
            expr.resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);

        let mut expr = IfStat::parse(
            r##"
            if 1 as bool {
                let x = 1;
            }
        "##
            .into(),
        )
        .unwrap()
        .1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res =
            expr.resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn valid_if_else_if() {
        let mut expr = IfStat::parse(
            r##"
            if true {
                let x = 1;
            } else if false {
                let x = 1;
            }
        "##
            .into(),
        )
        .unwrap()
        .1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res =
            expr.resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn valid_if_else() {
        let mut expr = IfStat::parse(
            r##"
            if true {
                let x = 1;
            } else {
                let x = 1;
            }
        "##
            .into(),
        )
        .unwrap()
        .1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res =
            expr.resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn valid_match() {
        let mut expr = MatchStat::parse(
            r##"
            match x {
                case 20 => {
                    let x = 1;
                }
                case 30 => {
                    let x = 1;
                }
            }
        "##
            .into(),
        )
        .unwrap()
        .1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let _ = scope_manager.register_var("x", p_num!(I64), None).unwrap();
        let res =
            expr.resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn valid_try() {
        let mut expr = TryStat::parse(
            r##"
            try {
                let x = 1;
            }
        "##
            .into(),
        )
        .unwrap()
        .1;
        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res =
            expr.resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);
    }
}

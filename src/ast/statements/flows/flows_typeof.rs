use std::cell::Ref;

use super::{CallStat, Flow, IfStat, MatchStat, TryStat};
use crate::semantic::scope::type_traits::TypeChecking;
use crate::semantic::scope::BuildStaticType;
use crate::semantic::EType;
use crate::semantic::{
    scope::{static_types::StaticType, user_type_impl::UserType, ScopeApi},
    Either, MergeType, Resolve, SemanticError, TypeOf,
};

impl<Scope: ScopeApi> TypeOf<Scope> for Flow<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Flow::If(value) => value.type_of(&scope),
            Flow::Match(value) => value.type_of(&scope),
            Flow::Try(value) => value.type_of(&scope),
            Flow::Call(value) => value.type_of(&scope),
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for IfStat<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let is_generator = scope.state().is_generator;

        let mut main_type = self.then_branch.type_of(&scope)?;

        for (_, else_if_scope) in &self.else_if_branches {
            let else_if_type = else_if_scope.type_of(scope)?;
            if is_generator && else_if_type.is_unit() {
                continue;
            }
            main_type = main_type.merge(&else_if_type, scope)?;
        }
        match &self.else_branch {
            Some(else_branch) => {
                let else_type = else_branch.type_of(&scope)?;
                if is_generator && else_type.is_unit() {
                    return Ok(main_type);
                }
                main_type.merge(&else_type, scope)
            }
            None => {
                if is_generator {
                    return Ok(main_type);
                }

                Ok(Either::Static(
                    <StaticType as BuildStaticType<Scope>>::build_unit().into(),
                ))
            }
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for MatchStat<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let is_generator = scope.state().is_generator;

        let pattern_type = {
            if let Some(res) = self.patterns.first() {
                let mut res = res.scope.type_of(&scope)?;
                if self.patterns.len() > 1 {
                    for pattern in &self.patterns {
                        let pattern_type = pattern.scope.type_of(&scope)?;
                        if is_generator && pattern_type.is_unit() {
                            continue;
                        }
                        res = res.merge(&pattern_type, scope)?;
                    }
                }
                Some(res)
            } else {
                None
            }
        };
        let Some(pattern_type) = pattern_type else {
            return Err(SemanticError::CantInferType);
        };
        match &self.else_branch {
            Some(else_branch) => {
                let else_type = else_branch.type_of(&scope)?;
                if is_generator && else_type.is_unit() {
                    return Ok(pattern_type);
                }
                pattern_type.merge(&else_type, scope)
            }
            None => {
                if is_generator {
                    return Ok(pattern_type);
                }

                Ok(Either::Static(
                    <StaticType as BuildStaticType<Scope>>::build_unit().into(),
                ))
            }
        }
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for TryStat<Scope> {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let is_generator = scope.state().is_generator;
        let main_type = self.try_branch.type_of(&scope)?;
        match &self.else_branch {
            Some(else_branch) => {
                let else_type = else_branch.type_of(&scope)?;
                if is_generator && else_type.is_unit() {
                    return Ok(main_type);
                }
                main_type.merge(&else_type, scope)
            }
            None => {
                if is_generator {
                    return Ok(main_type);
                }

                Ok(Either::Static(
                    <StaticType as BuildStaticType<Scope>>::build_unit().into(),
                ))
            }
        }
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for CallStat<Scope> {
    fn type_of(&self, _scope: &Ref<Scope>) -> Result<crate::semantic::EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        Ok(Either::Static(
            <StaticType as BuildStaticType<Scope>>::build_unit().into(),
        ))
    }
}

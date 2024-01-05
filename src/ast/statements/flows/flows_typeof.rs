use std::cell::Ref;

use super::{CallStat, Flow, IfStat, MatchStat, PatternStat, TryStat};
use crate::semantic::scope::BuildStaticType;
use crate::semantic::{scope::ScopeApi, EitherType, MergeType, Resolve, SemanticError, TypeOf};

impl<Scope: ScopeApi> TypeOf<Scope> for Flow {
    fn type_of(
        &self,
        scope: &Ref<Scope>,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
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
impl<Scope: ScopeApi> TypeOf<Scope> for IfStat {
    fn type_of(
        &self,
        scope: &Ref<Scope>,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let main_type = self.main_branch.type_of(&scope)?;
        match &self.else_branch {
            Some(else_branch) => {
                let else_type = else_branch.type_of(&scope)?;
                main_type.merge(&else_type, scope)
            }
            None => Ok(EitherType::Static(Scope::StaticType::build_unit())),
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for MatchStat {
    fn type_of(
        &self,
        scope: &Ref<Scope>,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let pattern_type = {
            if let Some(res) = self.patterns.first() {
                let mut res = res.type_of(&scope)?;
                if self.patterns.len() > 1 {
                    for pattern in &self.patterns {
                        let pattern_type = pattern.type_of(&scope)?;
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
                pattern_type.merge(&else_type, scope)
            }
            None => Ok(EitherType::Static(Scope::StaticType::build_unit())),
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for PatternStat {
    fn type_of(
        &self,
        scope: &Ref<Scope>,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        self.scope.type_of(&scope)
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for TryStat {
    fn type_of(
        &self,
        scope: &Ref<Scope>,
    ) -> Result<EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let main_type = self.try_branch.type_of(&scope)?;
        match &self.else_branch {
            Some(else_branch) => {
                let else_type = else_branch.type_of(&scope)?;
                main_type.merge(&else_type, scope)
            }
            None => Ok(EitherType::Static(Scope::StaticType::build_unit())),
        }
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for CallStat {
    fn type_of(
        &self,
        _scope: &Ref<Scope>,
    ) -> Result<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        Ok(EitherType::Static(Scope::StaticType::build_unit()))
    }
}

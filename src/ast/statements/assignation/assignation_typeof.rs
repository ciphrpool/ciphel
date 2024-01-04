use super::{AssignValue, Assignation, Assignee};
use crate::semantic::scope::BuildStaticType;
use crate::semantic::EitherType;
use crate::semantic::{scope::ScopeApi, Resolve, SemanticError, TypeOf};

impl<Scope: ScopeApi> TypeOf<Scope> for Assignation {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        Ok(EitherType::Static(Scope::StaticType::build_unit()))
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for AssignValue {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            AssignValue::Scope(value) => value.type_of(scope),
            AssignValue::Expr(value) => value.type_of(scope),
        }
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for Assignee {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<crate::semantic::EitherType<Scope::UserType, Scope::StaticType>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            Assignee::Variable(value) => value.type_of(scope),
            Assignee::PtrAccess(value) => value.type_of(scope),
        }
    }
}

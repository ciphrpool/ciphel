use super::Error;
use crate::semantic::scope::BuildStaticType;
use crate::semantic::{scope::ScopeApi, EitherType, Resolve, SemanticError, TypeOf};

impl<Scope: ScopeApi> TypeOf<Scope> for Error {
    fn type_of(
        &self,
        scope: &Scope,
    ) -> Result<Option<EitherType<Scope::UserType, Scope::StaticType>>, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        let static_type: Scope::StaticType = Scope::StaticType::build_error(&self);
        let static_type = static_type.type_of(scope)?;
        Ok(static_type)
    }
}

use super::{
    AddrType, ClosureType, MapType, PrimitiveType, RangeType, SliceType, StrSliceType, StringType,
    TupleType, Type, Types, VecType,
};
use crate::semantic::scope::scope::ScopeManager;
use crate::semantic::{Resolve, SemanticError};

impl Resolve for Type {
    type Output = ();
    type Context = ();

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
            Type::Primitive(value) => value.resolve::<G>(scope_manager, scope_id, context, extra),
            Type::Slice(value) => value.resolve::<G>(scope_manager, scope_id, context, extra),
            Type::StrSlice(value) => value.resolve::<G>(scope_manager, scope_id, context, extra),
            Type::UserType(ref value) => {
                let _ = scope_manager.find_type_by_name(value, scope_id)?;
                Ok(())
            }
            Type::Vec(value) => value.resolve::<G>(scope_manager, scope_id, context, extra),
            Type::Closure(value) => value.resolve::<G>(scope_manager, scope_id, context, extra),
            Type::Tuple(value) => value.resolve::<G>(scope_manager, scope_id, context, extra),
            Type::Unit => Ok(()),
            Type::Any => Ok(()),
            Type::Address(value) => value.resolve::<G>(scope_manager, scope_id, context, extra),
            Type::Map(value) => value.resolve::<G>(scope_manager, scope_id, context, extra),
            Type::String(value) => value.resolve::<G>(scope_manager, scope_id, context, extra),
            Type::Range(value) => value.resolve::<G>(scope_manager, scope_id, context, extra),
            Type::Error => Ok(()),
        }
    }
}
impl Resolve for PrimitiveType {
    type Output = ();
    type Context = ();

    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        _scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        _context: &Self::Context,
        _extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        Ok(())
    }
}
impl Resolve for SliceType {
    type Output = ();
    type Context = ();

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
        self.item_type
            .resolve::<G>(scope_manager, scope_id, context, extra)
    }
}

impl Resolve for StrSliceType {
    type Output = ();
    type Context = ();

    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        _scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        _context: &Self::Context,
        _extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        Ok(())
    }
}

impl Resolve for StringType {
    type Output = ();
    type Context = ();

    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        _scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        _context: &Self::Context,
        _extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        Ok(())
    }
}

impl Resolve for VecType {
    type Output = ();
    type Context = ();

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
        self.0.resolve::<G>(scope_manager, scope_id, context, extra)
    }
}

impl Resolve for ClosureType {
    type Output = ();
    type Context = ();

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
        for param in &mut self.params {
            let _ = param.resolve::<G>(scope_manager, scope_id, context, extra)?;
        }
        self.ret
            .resolve::<G>(scope_manager, scope_id, context, extra)
    }
}

impl Resolve for Types {
    type Output = ();
    type Context = ();

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
        for item in self {
            let _ = item.resolve::<G>(scope_manager, scope_id, context, extra)?;
        }
        Ok(())
    }
}

impl Resolve for TupleType {
    type Output = ();
    type Context = ();

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
        self.0.resolve::<G>(scope_manager, scope_id, context, extra)
    }
}

impl Resolve for AddrType {
    type Output = ();
    type Context = ();

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
        self.0.resolve::<G>(scope_manager, scope_id, context, extra)
    }
}
impl Resolve for RangeType {
    type Output = ();
    type Context = ();

    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        _scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        _context: &Self::Context,
        _extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        Ok(())
    }
}

impl Resolve for MapType {
    type Output = ();
    type Context = ();

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
            .keys_type
            .resolve::<G>(scope_manager, scope_id, context, extra)?;
        self.values_type
            .resolve::<G>(scope_manager, scope_id, context, extra)
    }
}

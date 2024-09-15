use super::{ForLoop, Loop, WhileLoop};
use crate::ast::statements::block::BlockCommonApi;
use crate::ast::statements::loops::ForInit;
use crate::e_static;
use crate::semantic::scope::scope::{ScopeManager, ScopeState};
use crate::semantic::scope::static_types::{PrimitiveType, StaticType};
use crate::semantic::scope::type_traits::TypeChecking;
use crate::semantic::EType;
use crate::semantic::{Resolve, SemanticError, TypeOf};

impl Resolve for Loop {
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
            Loop::For(value) => value.resolve::<G>(scope_manager, scope_id, context, extra),
            Loop::While(value) => value.resolve::<G>(scope_manager, scope_id, context, extra),
            Loop::Loop(value) => {
                let inner_scope = value.init_from_parent(scope_manager, scope_id)?;
                scope_manager
                    .scope_states
                    .insert(inner_scope, ScopeState::Loop);

                let _ = value.resolve::<G>(scope_manager, scope_id, &context, &mut ())?;
                Ok(())
            }
        }
    }
}

impl Resolve for ForLoop {
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
        self.block
            .resolve::<G>(scope_manager, scope_id, context, &mut ())
    }
}

impl Resolve for WhileLoop {
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
            .condition
            .resolve::<G>(scope_manager, scope_id, &None, &mut None)?;
        // check that the condition is a boolean
        let condition_type = self.condition.type_of(&scope_manager, scope_id)?;
        if EType::Static(StaticType::Primitive(PrimitiveType::Bool)) != condition_type {
            return Err(SemanticError::ExpectedBoolean);
        }
        let inner_scope = self.block.init_from_parent(scope_manager, scope_id)?;
        scope_manager
            .scope_states
            .insert(inner_scope, ScopeState::Loop);
        let _ = self
            .block
            .resolve::<G>(scope_manager, scope_id, context, &mut ())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::{ast::TryParse, p_num, semantic::scope::scope};

    use super::*;

    #[test]
    fn valid_for_loop() {
        let mut expr_loop = ForLoop::parse(
            r##"
        for i in [1,2,3] {
            x = i;
        }
        "##
            .into(),
        )
        .unwrap()
        .1;
        let mut scope_manager = scope::ScopeManager::default();
        let _ = scope_manager.register_var("x", p_num!(I64), None).unwrap();
        let res = expr_loop.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);

        let mut expr_loop = ForLoop::parse(
            r##"
        for (i,j) in [(1,1),(2,2),(3,3)] {
            x = j;
        }
        "##
            .into(),
        )
        .unwrap()
        .1;
        let mut scope_manager = scope::ScopeManager::default();
        let _ = scope_manager.register_var("x", p_num!(I64), None).unwrap();
        let res = expr_loop.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn valid_for_loop_range() {
        let mut expr_loop = ForLoop::parse(
            r##"
        for i in 0..10 {
            x = i;
        }
        "##
            .into(),
        )
        .unwrap()
        .1;
        let mut scope_manager = scope::ScopeManager::default();
        let _ = scope_manager.register_var("x", p_num!(I64), None).unwrap();
        let res = expr_loop.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn valid_for_loop_range_u64() {
        let mut expr_loop = ForLoop::parse(
            r##"
        for i in 0u64..10u64 {
            x = i;
        }
        "##
            .into(),
        )
        .unwrap()
        .1;
        let mut scope_manager = scope::ScopeManager::default();
        let _ = scope_manager.register_var("x", p_num!(I64), None).unwrap();
        let res = expr_loop.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_for_loop() {
        let mut expr_loop = ForLoop::parse(
            r##"
        for i in y {
            x = i;
        }
        "##
            .into(),
        )
        .unwrap()
        .1;
        let mut scope_manager = scope::ScopeManager::default();
        let _ = scope_manager.register_var("x", p_num!(I64), None).unwrap();
        let res = expr_loop.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_err());

        let mut expr_loop = ForLoop::parse(
            r##"
        for i in y {
            x = i;
        }
        "##
            .into(),
        )
        .unwrap()
        .1;
        let mut scope_manager = scope::ScopeManager::default();
        let _ = scope_manager.register_var("x", p_num!(I64), None).unwrap();

        let _ = scope_manager.register_var("y", p_num!(I64), None).unwrap();
        let res = expr_loop.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_err());
    }

    #[test]
    fn valid_while_loop() {
        let mut expr_loop = WhileLoop::parse(
            r##"
        while x > 10 {
            x = x + 1;
        }
        "##
            .into(),
        )
        .unwrap()
        .1;
        let mut scope_manager = scope::ScopeManager::default();
        let _ = scope_manager.register_var("x", p_num!(I64), None).unwrap();
        let res = expr_loop.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_while() {
        let mut expr_loop = WhileLoop::parse(
            r##"
        while x {
            x = x + 1;
        }
        "##
            .into(),
        )
        .unwrap()
        .1;
        let mut scope_manager = scope::ScopeManager::default();
        let _ = scope_manager.register_var("x", p_num!(I64), None).unwrap();
        let res = expr_loop.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_err());
    }

    #[test]
    fn valid_loop() {
        let mut expr_loop = Loop::parse(
            r##"
        loop {
            x = x + 1;
        }
        "##
            .into(),
        )
        .unwrap()
        .1;
        let mut scope_manager = scope::ScopeManager::default();
        let _ = scope_manager.register_var("x", p_num!(I64), None).unwrap();
        let res = expr_loop.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);
    }
}

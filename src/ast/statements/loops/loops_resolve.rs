use super::{ForLoop, Loop, WhileLoop};
use crate::ast::statements::block::{Block, BlockCommonApi};
use crate::ast::statements::loops::ForInit;
use crate::ast::statements::Statement;
use crate::semantic::scope::scope::ScopeState;
use crate::semantic::scope::static_types::{PrimitiveType, StaticType};
use crate::semantic::{Desugar, EType};
use crate::semantic::{Resolve, SemanticError, TypeOf};

impl Resolve for Loop {
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
            Loop::For(value) => value.resolve::<E>(scope_manager, scope_id, context, extra),
            Loop::While(value) => value.resolve::<E>(scope_manager, scope_id, context, extra),
            Loop::Loop(value) => {
                let inner_scope = value.init_from_parent(scope_manager, scope_id)?;
                scope_manager
                    .scope_states
                    .insert(inner_scope, ScopeState::Loop);

                let _ = value.resolve::<E>(scope_manager, scope_id, &context, &mut ())?;
                Ok(())
            }
        }
    }
}

impl Desugar<Statement> for Loop {
    fn desugar<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<Statement>, SemanticError> {
        match self {
            Loop::For(for_loop) => for_loop.desugar::<E>(scope_manager, scope_id),
            Loop::While(while_loop) => while_loop.desugar::<E>(scope_manager, scope_id),
            Loop::Loop(block) => {
                let _: Option<Block> = block.desugar::<E>(scope_manager, scope_id)?;
                Ok(None)
            }
        }
    }
}

impl Desugar<Statement> for ForLoop {
    fn desugar<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<Statement>, SemanticError> {
        for index in self.indices.iter_mut() {
            match index {
                ForInit::Assignation(assignation) => {
                    if let Some(Statement::Assignation(output)) =
                        assignation.desugar::<E>(scope_manager, scope_id)?
                    {
                        *index = ForInit::Assignation(output);
                    }
                }
                ForInit::Declaration(declaration) => {
                    if let Some(Statement::Declaration(output)) =
                        declaration.desugar::<E>(scope_manager, scope_id)?
                    {
                        *index = ForInit::Declaration(output);
                    }
                }
            }
        }

        if let Some(output) = self.block.desugar::<E>(scope_manager, scope_id)? {
            self.block = Box::new(output);
        }

        if let Some(condition) = &mut self.condition {
            if let Some(output) = condition.desugar::<E>(scope_manager, scope_id)? {
                *condition = output;
            }
        }

        for increment in self.increments.iter_mut() {
            if let Some(Statement::Assignation(output)) =
                increment.desugar::<E>(scope_manager, scope_id)?
            {
                *increment = output;
            }
        }

        Ok(None)
    }
}
impl Desugar<Statement> for WhileLoop {
    fn desugar<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<Statement>, SemanticError> {
        let _: Option<Block> = self.block.desugar::<E>(scope_manager, scope_id)?;
        self.condition.desugar::<E>(scope_manager, scope_id)?;
        Ok(None)
    }
}
impl Resolve for ForLoop {
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
        for index in self.indices.iter_mut() {
            match index {
                ForInit::Assignation(assignation) => {
                    let _ = assignation.resolve::<E>(scope_manager, scope_id, context, extra)?;
                }
                ForInit::Declaration(declaration) => {
                    let _ = declaration.resolve::<E>(scope_manager, scope_id, context, extra)?;
                }
            }
        }

        if let Some(condition) = &mut self.condition {
            let _ = condition.resolve::<E>(
                scope_manager,
                scope_id,
                &Some(EType::Static(StaticType::Primitive(PrimitiveType::Bool))),
                &mut None,
            )?;
            if EType::Static(StaticType::Primitive(PrimitiveType::Bool))
                != condition.type_of(scope_manager, scope_id)?
            {
                return Err(SemanticError::IncompatibleTypes);
            }
        }

        for increment in self.increments.iter_mut() {
            let _ = increment.resolve::<E>(scope_manager, scope_id, context, extra)?;
        }

        let inner = self.block.init_from_parent(scope_manager, scope_id)?;
        scope_manager.scope_states.insert(inner, ScopeState::Loop);

        let _ = self
            .block
            .resolve::<E>(scope_manager, scope_id, context, extra)?;

        Ok(())
    }
}

impl Resolve for WhileLoop {
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
            .condition
            .resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
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
            .resolve::<E>(scope_manager, scope_id, context, &mut ())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::{ast::TryParse, p_num, semantic::scope::scope};

    use super::*;

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
        let res = expr_loop.resolve::<crate::vm::external::test::NoopGameEngine>(
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
        let res = expr_loop.resolve::<crate::vm::external::test::NoopGameEngine>(
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
        let res = expr_loop.resolve::<crate::vm::external::test::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);
    }
}

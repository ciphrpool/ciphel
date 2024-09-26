use std::sync::atomic::Ordering;

use super::{Block, ClosureBlock, ExprBlock, FunctionBlock, LambdaBlock};

use crate::ast::statements::{self, Statement};
use crate::semantic::scope::scope::{ScopeManager, ScopeState};
use crate::semantic::scope::static_types::StaticType;

use crate::semantic::{CompatibleWith, Desugar, EType, TypeOf};
use crate::semantic::{Resolve, SemanticError};

impl Resolve for Block {
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
        if self.scope.is_none() {
            let inner_scope = scope_manager.spawn(scope_id)?;
            self.scope = Some(inner_scope)
        }

        for instruction in &mut self.statements {
            let _ = instruction.resolve::<G>(scope_manager, self.scope, context, &mut ())?;
        }

        // let return_type = self.type_of(&scope_manager, scope_id)?;

        let return_types = scope_manager
            .scope_types
            .get(&self.scope.ok_or(SemanticError::NotResolvedYet)?)
            .cloned()
            .unwrap_or(vec![EType::Static(StaticType::Unit)]);

        let mut scope_type = EType::Static(StaticType::Unit);
        for return_type in return_types.into_iter() {
            if scope_type == EType::Static(StaticType::Unit)
                && return_type != EType::Static(StaticType::Unit)
            {
                scope_type = return_type;
                continue;
            }
            if scope_type != EType::Static(StaticType::Unit)
                && return_type != EType::Static(StaticType::Unit)
            {
                scope_type.compatible_with(&return_type, scope_manager, scope_id)?;
            }
        }

        self.metadata.info = crate::semantic::Info::Resolved {
            context: context.clone(),
            signature: Some(scope_type),
        };

        Ok(())
    }
}

impl Desugar<Statement> for Block {
    fn desugar<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<Statement>, SemanticError> {
        for statement in self.statements.iter_mut() {
            if let Some(output) = statement.desugar::<G>(scope_manager, scope_id)? {
                *statement = output;
            }
        }
        Ok(None)
    }
}

impl Desugar<Block> for Block {
    fn desugar<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<Block>, SemanticError> {
        for statement in self.statements.iter_mut() {
            if let Some(output) = statement.desugar::<G>(scope_manager, scope_id)? {
                *statement = output;
            }
        }
        Ok(None)
    }
}

impl Resolve for FunctionBlock {
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
        if self.scope.is_none() {
            let inner_scope = scope_manager.spawn_allocating(scope_id, true)?;
            self.scope = Some(inner_scope)
        }

        for instruction in &mut self.statements {
            let _ = instruction.resolve::<G>(scope_manager, self.scope, context, &mut ())?;
        }

        // let return_type = self.type_of(&scope_manager, scope_id)?;

        let return_types = scope_manager
            .scope_types
            .get(&self.scope.ok_or(SemanticError::NotResolvedYet)?)
            .cloned()
            .unwrap_or(vec![EType::Static(StaticType::Unit)]);

        let mut scope_type = EType::Static(StaticType::Unit);
        for return_type in return_types.into_iter() {
            if scope_type == EType::Static(StaticType::Unit)
                && return_type != EType::Static(StaticType::Unit)
            {
                scope_type = return_type;
                continue;
            }
            if scope_type != EType::Static(StaticType::Unit)
                && return_type != EType::Static(StaticType::Unit)
            {
                scope_type.compatible_with(&return_type, scope_manager, scope_id)?;
            }
        }

        self.metadata.info = crate::semantic::Info::Resolved {
            context: context.clone(),
            signature: Some(scope_type),
        };

        Ok(())
    }
}

impl Desugar<FunctionBlock> for FunctionBlock {
    fn desugar<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<FunctionBlock>, SemanticError> {
        for statement in self.statements.iter_mut() {
            if let Some(output) = statement.desugar::<G>(scope_manager, scope_id)? {
                *statement = output;
            }
        }
        Ok(None)
    }
}

impl Resolve for ClosureBlock {
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
        if self.scope.is_none() {
            let inner_scope = scope_manager.spawn_allocating(scope_id, true)?;
            self.scope = Some(inner_scope)
        }

        scope_manager
            .scope_states
            .insert(self.scope.unwrap(), ScopeState::Closure);

        for instruction in &mut self.statements {
            let _ = instruction.resolve::<G>(scope_manager, self.scope, context, &mut ())?;
        }

        // let return_type = self.type_of(&scope_manager, scope_id)?;

        let return_types = scope_manager
            .scope_types
            .get(&self.scope.ok_or(SemanticError::NotResolvedYet)?)
            .cloned()
            .unwrap_or(vec![EType::Static(StaticType::Unit)]);

        let mut scope_type = EType::Static(StaticType::Unit);
        for return_type in return_types.into_iter() {
            if scope_type == EType::Static(StaticType::Unit)
                && return_type != EType::Static(StaticType::Unit)
            {
                scope_type = return_type;
                continue;
            }
            if scope_type != EType::Static(StaticType::Unit)
                && return_type != EType::Static(StaticType::Unit)
            {
                scope_type.compatible_with(&return_type, scope_manager, scope_id)?;
            }
        }

        self.metadata.info = crate::semantic::Info::Resolved {
            context: context.clone(),
            signature: Some(scope_type),
        };

        Ok(())
    }
}

impl Desugar<ClosureBlock> for ClosureBlock {
    fn desugar<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<ClosureBlock>, SemanticError> {
        for statement in self.statements.iter_mut() {
            if let Some(output) = statement.desugar::<G>(scope_manager, scope_id)? {
                *statement = output;
            }
        }
        Ok(None)
    }
}

impl Resolve for LambdaBlock {
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
        if self.scope.is_none() {
            let inner_scope = scope_manager.spawn_allocating(scope_id, true)?;
            self.scope = Some(inner_scope)
        }

        scope_manager
            .scope_states
            .insert(self.scope.unwrap(), ScopeState::Lambda);

        for instruction in &mut self.statements {
            let _ = instruction.resolve::<G>(scope_manager, self.scope, context, &mut ())?;
        }

        // let return_type = self.type_of(&scope_manager, scope_id)?;

        let return_types = scope_manager
            .scope_types
            .get(&self.scope.ok_or(SemanticError::NotResolvedYet)?)
            .cloned()
            .unwrap_or(vec![EType::Static(StaticType::Unit)]);

        let mut scope_type = EType::Static(StaticType::Unit);
        for return_type in return_types.into_iter() {
            if scope_type == EType::Static(StaticType::Unit)
                && return_type != EType::Static(StaticType::Unit)
            {
                scope_type = return_type;
                continue;
            }
            if scope_type != EType::Static(StaticType::Unit)
                && return_type != EType::Static(StaticType::Unit)
            {
                scope_type.compatible_with(&return_type, scope_manager, scope_id)?;
            }
        }

        self.metadata.info = crate::semantic::Info::Resolved {
            context: context.clone(),
            signature: Some(scope_type),
        };

        Ok(())
    }
}

impl Desugar<LambdaBlock> for LambdaBlock {
    fn desugar<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<LambdaBlock>, SemanticError> {
        for statement in self.statements.iter_mut() {
            if let Some(output) = statement.desugar::<G>(scope_manager, scope_id)? {
                *statement = output;
            }
        }
        Ok(None)
    }
}

impl Resolve for ExprBlock {
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
        if self.scope.is_none() {
            let inner_scope = scope_manager.spawn_allocating(scope_id, false)?;
            self.scope = Some(inner_scope)
        }
        if scope_manager.is_scope_global(scope_id) {
            scope_manager
                .scope_states
                .insert(self.scope.unwrap(), ScopeState::IIFE);
        } else {
            scope_manager
                .scope_states
                .insert(self.scope.unwrap(), ScopeState::Inline);
        }

        for instruction in &mut self.statements {
            let _ = instruction.resolve::<G>(scope_manager, self.scope, context, &mut ())?;
        }

        if let Some(mapping) = scope_manager.allocating_scope.get(&self.scope.unwrap()) {
            if mapping.vars.len() == 0 && mapping.param_size == 0 {
                // The block is now Inlined
                scope_manager
                    .scope_states
                    .insert(self.scope.unwrap(), ScopeState::Inline);
            }
        }

        let return_types = scope_manager
            .scope_types
            .get(&self.scope.ok_or(SemanticError::NotResolvedYet)?)
            .cloned()
            .unwrap_or(vec![EType::Static(StaticType::Unit)]);

        let mut scope_type = EType::Static(StaticType::Unit);
        for return_type in return_types.into_iter() {
            if scope_type == EType::Static(StaticType::Unit)
                && return_type != EType::Static(StaticType::Unit)
            {
                scope_type = return_type;
                continue;
            }
            if scope_type != EType::Static(StaticType::Unit)
                && return_type != EType::Static(StaticType::Unit)
            {
                scope_type.compatible_with(&return_type, scope_manager, scope_id)?;
            }
        }

        self.metadata.info = crate::semantic::Info::Resolved {
            context: context.clone(),
            signature: Some(scope_type),
        };

        Ok(())
    }
}

impl Desugar<ExprBlock> for ExprBlock {
    fn desugar<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<ExprBlock>, SemanticError> {
        for statement in self.statements.iter_mut() {
            if let Some(output) = statement.desugar::<G>(scope_manager, scope_id)? {
                *statement = output;
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::TryParse,
        e_static, p_num,
        semantic::scope::{scope, static_types::StaticType},
    };

    use super::*;

    #[test]
    fn valid_no_return_scope() {
        let mut expr_scope = Block::parse(
            r##"
        {
            let x = 10;
            let y = x + 20;
        }
        "##
            .into(),
        )
        .unwrap()
        .1;
        let mut scope_manager = scope::ScopeManager::default();
        let res = expr_scope.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);
        let res_scope = expr_scope.scope.unwrap();

        let var_x = scope_manager
            .find_var_by_name("x", Some(res_scope))
            .expect("variable should declared");
        let x_type = var_x.ctype.clone();

        let var_y = scope_manager
            .find_var_by_name("y", Some(res_scope))
            .expect("variable should declared");
        let y_type = var_y.ctype.clone();

        assert_eq!(p_num!(I64), x_type);
        assert_eq!(p_num!(I64), y_type);

        let res = expr_scope
            .metadata
            .signature()
            .expect("Block should be resolved");
        assert_eq!(e_static!(StaticType::Unit), res)
    }

    #[test]
    fn valid_basic_scope() {
        let mut expr_scope = Block::parse(
            r##"
        {
            let x = 10;
            let y = x + 20;

            return y;
        }
        "##
            .into(),
        )
        .unwrap()
        .1;
        let mut scope_manager = scope::ScopeManager::default();
        let res = expr_scope.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);
        let res_scope = expr_scope.scope.unwrap();

        let var_x = scope_manager
            .find_var_by_name("x", Some(res_scope))
            .expect("variable should declared");
        let x_type = var_x.ctype.clone();

        let var_y = scope_manager
            .find_var_by_name("y", Some(res_scope))
            .expect("variable should declared");
        let y_type = var_y.ctype.clone();

        assert_eq!(p_num!(I64), x_type);
        assert_eq!(p_num!(I64), y_type);

        let res = expr_scope
            .metadata
            .signature()
            .expect("Block should be resolved");
        assert_eq!(p_num!(I64), res)
    }

    #[test]
    fn valid_if_scope() {
        let mut expr_scope = Block::parse(
            r##"
        {
            let x = 10;
            let y = x + 20;

            if false {
                return x;
            }

            return y;
        }
        "##
            .into(),
        )
        .unwrap()
        .1;
        let mut scope_manager = scope::ScopeManager::default();
        let res = expr_scope.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);
        let res_scope = expr_scope.scope.unwrap();

        let var_x = scope_manager
            .find_var_by_name("x", Some(res_scope))
            .expect("variable should declared");
        let x_type = var_x.ctype.clone();

        let var_y = scope_manager
            .find_var_by_name("y", Some(res_scope))
            .expect("variable should declared");
        let y_type = var_y.ctype.clone();

        assert_eq!(p_num!(I64), x_type);
        assert_eq!(p_num!(I64), y_type);

        let res = expr_scope
            .metadata
            .signature()
            .expect("Block should be resolved");
        assert_eq!(p_num!(I64), res);

        let mut expr_scope = Block::parse(
            r##"
        {
            let x = 10;
            let y = x + 20;

            if false {
                return x;
            } else {
                return y;
            }
        }
        "##
            .into(),
        )
        .unwrap()
        .1;
        let mut scope_manager = scope::ScopeManager::default();
        let res = expr_scope.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);
        let res_scope = expr_scope.scope.unwrap();

        let var_x = scope_manager
            .find_var_by_name("x", Some(res_scope))
            .expect("variable should declared");
        let x_type = var_x.ctype.clone();

        let var_y = scope_manager
            .find_var_by_name("y", Some(res_scope))
            .expect("variable should declared");
        let y_type = var_y.ctype.clone();

        assert_eq!(p_num!(I64), x_type);
        assert_eq!(p_num!(I64), y_type);

        let res = expr_scope
            .metadata
            .signature()
            .expect("Block should be resolved");
        assert_eq!(p_num!(I64), res);
    }

    #[test]
    fn valid_for_loop() {
        let mut expr_scope = Block::parse(
            r##"
        {
            let x = [10,20];
            for i in x {
                return i;
            }
        }
        "##
            .into(),
        )
        .unwrap()
        .1;
        let mut scope_manager = scope::ScopeManager::default();
        let res = expr_scope.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);
        let res_scope = expr_scope.scope.unwrap();

        let var_x = scope_manager
            .find_var_by_name("x", Some(res_scope))
            .expect("variable should declared");
        let x_type = var_x.ctype.clone();

        assert_eq!(p_num!(I64), x_type);

        let res = expr_scope
            .metadata
            .signature()
            .expect("Block should be resolved");
        assert_eq!(p_num!(I64), res)
    }

    #[test]
    fn robustness_scope() {
        let mut expr_scope = Block::parse(
            r##"
        {
            let x = 10;
            let y = x + 20;

            if false {
                return x;
            }
        }
        "##
            .into(),
        )
        .unwrap()
        .1;
        let mut scope_manager = scope::ScopeManager::default();
        let res = expr_scope.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &None,
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);
        let res_scope = expr_scope.scope.unwrap();

        let var_x = scope_manager
            .find_var_by_name("x", Some(res_scope))
            .expect("variable should declared");
        let x_type = var_x.ctype.clone();

        assert_eq!(p_num!(I64), x_type);

        let res = expr_scope
            .metadata
            .signature()
            .expect("Block should be resolved");
        assert_eq!(EType::Static(StaticType::Unit), res)
    }
}

use std::sync::atomic::Ordering;

use super::Block;

use crate::semantic::scope::scope::Scope;
use crate::{arw_read, arw_write, resolve_metadata};

use crate::semantic::scope::var_impl::{Var, VarState};
use crate::semantic::{CompatibleWith, EType, TypeOf};
use crate::semantic::{Resolve, SemanticError};

impl Resolve for Block {
    //type Output = ArcMutex;
    type Output = ();
    type Context = Option<EType>;
    type Extra = Vec<Var>;
    fn resolve<G:crate::GameEngineStaticFn>(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let mut inner_scope = Scope::spawn(scope, extra.clone())?;
        {
            if self.is_loop.load(Ordering::Acquire) {
                let _ = arw_write!(inner_scope, SemanticError::ConcurrencyError)?.to_loop();
            }
            let mut borrowed_caller = arw_write!(self.caller, SemanticError::ConcurrencyError)?;
            if let Some(caller) = borrowed_caller.as_mut() {
                caller.state = VarState::Parameter;
                let _ = arw_write!(inner_scope, SemanticError::ConcurrencyError)?
                    .register_var(caller.clone())?;
            }
            let can_capture = arw_read!(self.can_capture, SemanticError::ConcurrencyError)?.clone();
            let _ =
                arw_write!(inner_scope, SemanticError::ConcurrencyError)?.to_closure(can_capture);
        }

        for instruction in &mut self.instructions {
            let _ = instruction.resolve::<G>(&mut inner_scope, context, &mut ())?;
        }
        // {
        //     inner_scope.as_ref().borrow_mut().capture_needed_vars();
        // }

        self.inner_scope.replace(inner_scope);

        let return_type =
            self.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
        let _ = context.compatible_with(
            &return_type,
            &crate::arw_read!(scope, SemanticError::ConcurrencyError)?,
        )?;
        resolve_metadata!(self.metadata.info, self, scope, context);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        arw_read,
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
        let scope = scope::Scope::new();
        let res = expr_scope.resolve::<crate::vm::vm::NoopGameEngine>(&scope, &None, &mut Vec::default());
        assert!(res.is_ok(), "{:?}", res);
        let res_scope = expr_scope.inner_scope.clone().unwrap();

        let binding = arw_read!(res_scope, SemanticError::ConcurrencyError)
            .unwrap()
            .find_var(&"x".to_string().into())
            .unwrap();
        let var_x = binding.read().unwrap();
        let x_type = var_x
            .type_of(&arw_read!(res_scope, SemanticError::ConcurrencyError).unwrap())
            .unwrap();
        let binding = arw_read!(res_scope, SemanticError::ConcurrencyError)
            .unwrap()
            .find_var(&"y".to_string().into())
            .unwrap();
        let var_y = binding.read().unwrap();
        let y_type = var_y
            .type_of(&arw_read!(res_scope, SemanticError::ConcurrencyError).unwrap())
            .unwrap();

        assert_eq!(p_num!(I64), x_type);
        assert_eq!(p_num!(I64), y_type);

        let res = expr_scope
            .type_of(&arw_read!(res_scope, SemanticError::ConcurrencyError).unwrap())
            .unwrap();
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
        let scope = scope::Scope::new();
        let res = expr_scope.resolve::<crate::vm::vm::NoopGameEngine>(&scope, &Some(p_num!(I64)), &mut Vec::default());
        assert!(res.is_ok(), "{:?}", res);
        let res_scope = expr_scope.inner_scope.clone().unwrap();

        let res = expr_scope
            .type_of(&arw_read!(res_scope, SemanticError::ConcurrencyError).unwrap())
            .unwrap();
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
        let scope = scope::Scope::new();
        let res = expr_scope.resolve::<crate::vm::vm::NoopGameEngine>(&scope, &Some(p_num!(I64)), &mut Vec::default());
        assert!(res.is_ok(), "{:?}", res);
        let res_scope = expr_scope.inner_scope.clone().unwrap();

        let res = expr_scope
            .type_of(&arw_read!(res_scope, SemanticError::ConcurrencyError).unwrap())
            .unwrap();
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
        let scope = scope::Scope::new();
        let res = expr_scope.resolve::<crate::vm::vm::NoopGameEngine>(&scope, &Some(p_num!(I64)), &mut Vec::default());
        assert!(res.is_ok(), "{:?}", res);
        let res_scope = expr_scope.inner_scope.clone().unwrap();

        let res = expr_scope
            .type_of(&arw_read!(res_scope, SemanticError::ConcurrencyError).unwrap())
            .unwrap();
        assert_eq!(p_num!(I64), res)
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
        let scope = scope::Scope::new();
        let res = expr_scope.resolve::<crate::vm::vm::NoopGameEngine>(&scope, &Some(p_num!(I64)), &mut Vec::default());
        assert!(res.is_ok(), "{:?}", res);
        let res_scope = expr_scope.inner_scope.clone().unwrap();

        let res = expr_scope
            .type_of(&arw_read!(res_scope, SemanticError::ConcurrencyError).unwrap())
            .unwrap();
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
        let scope = scope::Scope::new();
        let res = expr_scope.resolve::<crate::vm::vm::NoopGameEngine>(&scope, &Some(p_num!(I64)), &mut Vec::default());
        assert!(res.is_err());
    }
}

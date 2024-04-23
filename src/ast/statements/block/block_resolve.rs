use super::Block;

use crate::resolve_metadata;
use crate::semantic::scope::scope_impl::Scope;

use crate::semantic::scope::var_impl::{Var, VarState};
use crate::semantic::{CompatibleWith, EType, Info, MutRc, TypeOf};
use crate::semantic::{Resolve, SemanticError};

impl Resolve for Block {
    //type Output = MutRc;
    type Output = ();
    type Context = Option<EType>;
    type Extra = Vec<Var>;
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let mut inner_scope = Scope::spawn(scope, extra.clone())?;
        {
            if self.is_loop.get() {
                inner_scope.as_ref().borrow_mut().to_loop();
            }
            if self.is_generator.get() {
                inner_scope.as_ref().borrow_mut().to_generator();
            }
            if let Some(caller) = self.caller.as_ref().borrow().as_ref() {
                let caller = caller.clone();
                caller.state.set(VarState::Parameter);
                let _ = inner_scope.as_ref().borrow_mut().register_var(caller)?;
            }
            inner_scope
                .as_ref()
                .borrow_mut()
                .to_closure(self.can_capture.get());
        }

        for instruction in &self.instructions {
            let _ = instruction.resolve(&mut inner_scope, context, &())?;
        }
        // {
        //     inner_scope.as_ref().borrow_mut().capture_needed_vars();
        // }
        {
            let mut mut_self = self.inner_scope.borrow_mut();
            mut_self.replace(inner_scope);
        }

        let return_type = self.type_of(&scope.borrow())?;
        let _ = context.compatible_with(&return_type, &scope.borrow())?;
        resolve_metadata!(self.metadata, self, scope, context);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::TryParse,
        e_static, p_num,
        semantic::{
            scope::{
                scope_impl,
                static_types::{NumberType, PrimitiveType, StaticType},
            },
            Either,
        },
    };

    use super::*;

    #[test]
    fn valid_no_return_scope() {
        let expr_scope = Block::parse(
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
        let scope = scope_impl::Scope::new();
        let res = expr_scope.resolve(&scope, &None, &Vec::default());
        assert!(res.is_ok(), "{:?}", res);
        let res_scope = expr_scope.inner_scope.borrow().clone().unwrap();

        let var_x = res_scope.borrow().find_var(&"x".into()).unwrap();
        let x_type = var_x.type_of(&res_scope.borrow()).unwrap();
        let var_y = res_scope.borrow().find_var(&"y".into()).unwrap();
        let y_type = var_y.type_of(&res_scope.borrow()).unwrap();

        assert_eq!(p_num!(I64), x_type);
        assert_eq!(p_num!(I64), y_type);

        let res = expr_scope.type_of(&res_scope.borrow()).unwrap();
        assert_eq!(e_static!(StaticType::Unit), res)
    }

    #[test]
    fn valid_basic_scope() {
        let expr_scope = Block::parse(
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
        let scope = scope_impl::Scope::new();
        let res = expr_scope.resolve(&scope, &Some(p_num!(I64)), &Vec::default());
        assert!(res.is_ok(), "{:?}", res);
        let res_scope = expr_scope.inner_scope.borrow().clone().unwrap();

        let res = expr_scope.type_of(&res_scope.borrow()).unwrap();
        assert_eq!(p_num!(I64), res)
    }

    #[test]
    fn valid_if_scope() {
        let expr_scope = Block::parse(
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
        let scope = scope_impl::Scope::new();
        let res = expr_scope.resolve(&scope, &Some(p_num!(I64)), &Vec::default());
        assert!(res.is_ok(), "{:?}", res);
        let res_scope = expr_scope.inner_scope.borrow().clone().unwrap();

        let res = expr_scope.type_of(&res_scope.borrow()).unwrap();
        assert_eq!(p_num!(I64), res);

        let expr_scope = Block::parse(
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
        let scope = scope_impl::Scope::new();
        let res = expr_scope.resolve(&scope, &Some(p_num!(I64)), &Vec::default());
        assert!(res.is_ok(), "{:?}", res);
        let res_scope = expr_scope.inner_scope.borrow().clone().unwrap();

        let res = expr_scope.type_of(&res_scope.borrow()).unwrap();
        assert_eq!(p_num!(I64), res)
    }

    #[test]
    fn valid_for_loop() {
        let expr_scope = Block::parse(
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
        let scope = scope_impl::Scope::new();
        let res = expr_scope.resolve(&scope, &Some(p_num!(I64)), &Vec::default());
        assert!(res.is_ok(), "{:?}", res);
        let res_scope = expr_scope.inner_scope.borrow().clone().unwrap();

        let res = expr_scope.type_of(&res_scope.borrow()).unwrap();
        assert_eq!(p_num!(I64), res)
    }

    #[test]
    fn robustness_scope() {
        let expr_scope = Block::parse(
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
        let scope = scope_impl::Scope::new();
        let res = expr_scope.resolve(&scope, &Some(p_num!(I64)), &Vec::default());
        assert!(res.is_err());
    }
}

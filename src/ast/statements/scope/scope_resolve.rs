use std::cell::RefCell;
use std::rc::Rc;

use super::Scope;

use crate::semantic::{scope::ScopeApi, EitherType, Resolve, SemanticError};
use crate::semantic::{CompatibleWith, TypeOf};

impl<OuterScope: ScopeApi> Resolve<OuterScope> for Scope<OuterScope> {
    //type Output = Rc<RefCell<OuterScope>>;
    type Output = ();
    type Context = Option<EitherType<OuterScope::UserType, OuterScope::StaticType>>;
    type Extra = Vec<OuterScope::Var>;
    fn resolve(
        &self,
        scope: &Rc<RefCell<OuterScope>>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let mut inner_scope = OuterScope::child_scope_with(scope, extra.clone())?;
        for instruction in &self.instructions {
            let _ = instruction.resolve(&mut inner_scope, context, &())?;
        }

        {
            let mut mut_self = self.inner_scope.borrow_mut();
            mut_self.replace(inner_scope);
        }

        let return_type = self.type_of(&scope.borrow())?;
        let _ = context.compatible_with(&return_type, &scope.borrow())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::TryParse,
        semantic::scope::{
            scope_impl,
            static_type_impl::{PrimitiveType, StaticType},
        },
    };

    use super::*;

    #[test]
    fn valid_no_return_scope() {
        let expr_scope = Scope::parse(
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
        assert!(res.is_ok());
        let res_scope = expr_scope.inner_scope.borrow().clone().unwrap();

        let var_x = res_scope.borrow().find_var(&"x".into()).unwrap();
        let x_type = var_x.type_of(&res_scope.borrow()).unwrap();
        let var_y = res_scope.borrow().find_var(&"y".into()).unwrap();
        let y_type = var_y.type_of(&res_scope.borrow()).unwrap();

        assert_eq!(
            EitherType::Static(StaticType::Primitive(PrimitiveType::Number).into()),
            x_type
        );
        assert_eq!(
            EitherType::Static(StaticType::Primitive(PrimitiveType::Number).into()),
            y_type
        );

        let res = expr_scope.type_of(&res_scope.borrow()).unwrap();
        assert_eq!(EitherType::Static(StaticType::Unit.into()), res)
    }

    #[test]
    fn valid_basic_scope() {
        let expr_scope = Scope::parse(
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
        let res = expr_scope.resolve(
            &scope,
            &Some(EitherType::Static(
                StaticType::Primitive(PrimitiveType::Number).into(),
            )),
            &Vec::default(),
        );
        assert!(res.is_ok());
        let res_scope = expr_scope.inner_scope.borrow().clone().unwrap();

        let res = expr_scope.type_of(&res_scope.borrow()).unwrap();
        assert_eq!(
            EitherType::Static(StaticType::Primitive(PrimitiveType::Number).into()),
            res
        )
    }

    #[test]
    fn valid_if_scope() {
        let expr_scope = Scope::parse(
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
        let res = expr_scope.resolve(
            &scope,
            &Some(EitherType::Static(
                StaticType::Primitive(PrimitiveType::Number).into(),
            )),
            &Vec::default(),
        );
        assert!(res.is_ok());
        let res_scope = expr_scope.inner_scope.borrow().clone().unwrap();

        let res = expr_scope.type_of(&res_scope.borrow()).unwrap();
        assert_eq!(
            EitherType::Static(StaticType::Primitive(PrimitiveType::Number).into()),
            res
        );

        let expr_scope = Scope::parse(
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
        let res = expr_scope.resolve(
            &scope,
            &Some(EitherType::Static(
                StaticType::Primitive(PrimitiveType::Number).into(),
            )),
            &Vec::default(),
        );
        assert!(res.is_ok());
        let res_scope = expr_scope.inner_scope.borrow().clone().unwrap();

        let res = expr_scope.type_of(&res_scope.borrow()).unwrap();
        assert_eq!(
            EitherType::Static(StaticType::Primitive(PrimitiveType::Number).into()),
            res
        )
    }

    #[test]
    fn valid_for_loop() {
        let expr_scope = Scope::parse(
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
        let res = expr_scope.resolve(
            &scope,
            &Some(EitherType::Static(
                StaticType::Primitive(PrimitiveType::Number).into(),
            )),
            &Vec::default(),
        );
        assert!(res.is_ok());
        let res_scope = expr_scope.inner_scope.borrow().clone().unwrap();

        let res = expr_scope.type_of(&res_scope.borrow()).unwrap();
        assert_eq!(
            EitherType::Static(StaticType::Primitive(PrimitiveType::Number).into()),
            res
        )
    }

    #[test]
    fn robustness_scope() {
        let expr_scope = Scope::parse(
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
        let res = expr_scope.resolve(
            &scope,
            &Some(EitherType::Static(
                StaticType::Primitive(PrimitiveType::Number).into(),
            )),
            &Vec::default(),
        );
        assert!(res.is_err());
    }
}

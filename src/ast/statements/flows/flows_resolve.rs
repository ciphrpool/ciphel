use super::{CallStat, Flow, IfStat, MatchStat, TryStat};
use crate::semantic::{
    scope::{
        static_types::StaticType, type_traits::TypeChecking, user_type_impl::UserType, ScopeApi,
    },
    EType, Either, MutRc, Resolve, SemanticError, TypeOf,
};
use std::{cell::RefCell, rc::Rc};

impl<Scope: ScopeApi> Resolve<Scope> for Flow<Scope> {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            Flow::If(value) => value.resolve(scope, context, extra),
            Flow::Match(value) => value.resolve(scope, context, extra),
            Flow::Try(value) => value.resolve(scope, context, extra),
            Flow::Call(value) => value.resolve(scope, &(), &()),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for IfStat<Scope> {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.condition.resolve(scope, &None, extra)?;
        // check that condition is a boolean
        let condition_type = self.condition.type_of(&scope.borrow())?;
        if !<EType as TypeChecking>::is_boolean(&condition_type) {
            return Err(SemanticError::ExpectedBoolean);
        }

        let _ = self.then_branch.resolve(scope, &context, &Vec::default())?;
        if let Some(else_branch) = &self.else_branch {
            let _ = else_branch.resolve(scope, &context, &Vec::default())?;
        }
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for MatchStat<Scope> {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.expr.resolve(scope, &None, extra)?;
        let expr_type = Some(self.expr.type_of(&scope.borrow())?);

        for value in &self.patterns {
            let vars = value.pattern.resolve(scope, &expr_type, &())?;
            for (index, var) in vars.iter().enumerate() {
                var.is_parameter.set((index, true));
            }
            // create a scope and Scope::child_scope())variable to it before resolving the expression
            let _ = value.scope.resolve(scope, &context, &vars)?;
        }
        if let Some(else_branch) = &self.else_branch {
            let _ = else_branch.resolve(scope, &context, &Vec::default())?;
        }
        Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for TryStat<Scope> {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        _extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        let _ = self.try_branch.resolve(scope, &context, &Vec::default())?;
        if let Some(else_branch) = &self.else_branch {
            let _ = else_branch.resolve(scope, &context, &Vec::default())?;
        }
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for CallStat<Scope> {
    type Output = ();
    type Context = ();
    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        _context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        self.call.resolve(scope, &None, extra)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;
    use crate::ast::TryParse;
    use crate::semantic::scope::scope_impl::Scope;
    use crate::semantic::scope::static_types::{NumberType, PrimitiveType, StaticType};
    use crate::semantic::scope::var_impl::Var;
    #[test]
    fn valid_if() {
        let expr = IfStat::parse(
            r##"
            if true {
                let x = 1;
            }
        "##
            .into(),
        )
        .unwrap()
        .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = IfStat::parse(
            r##"
            if 1 as bool {
                let x = 1;
            }
        "##
            .into(),
        )
        .unwrap()
        .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn valid_match() {
        let expr = MatchStat::parse(
            r##"
            match x {
                case 20 => {
                    let x = 1;
                },
                case 30 => {
                    let x = 1;
                },
            }
        "##
            .into(),
        )
        .unwrap()
        .1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                is_captured: Cell::new((0, false)),
                is_parameter: Cell::new((0, false)),
                address: Cell::new(None),
                id: "x".into(),
                type_sig: Either::Static(
                    StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                ),
            })
            .unwrap();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn valid_try() {
        let expr = TryStat::parse(
            r##"
            try {
                let x = 1;
            }
        "##
            .into(),
        )
        .unwrap()
        .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);
    }
}

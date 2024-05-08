use super::{CallStat, Flow, IfStat, MatchStat, TryStat};
use crate::semantic::scope::scope::Scope;
use crate::{
    ast::expressions::flows::Pattern,
    semantic::{
        scope::{
            static_types::StaticType,
            type_traits::TypeChecking,
            user_type_impl::{Enum, Union, UserType},
            var_impl::VarState,
        },
        EType, Either, MutRc, Resolve, SemanticError, TypeOf,
    },
};
use std::collections::HashMap;

impl Resolve for Flow {
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
    {
        match self {
            Flow::If(value) => value.resolve(scope, context, extra),
            Flow::Match(value) => value.resolve(scope, context, extra),
            Flow::Try(value) => value.resolve(scope, context, extra),
            Flow::Call(value) => value.resolve(scope, &(), &()),
        }
    }
}
impl Resolve for IfStat {
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
    {
        let _ = self.condition.resolve(scope, &None, &None)?;
        // check that condition is a boolean
        let condition_type = self.condition.type_of(&scope.borrow())?;
        if !<EType as TypeChecking>::is_boolean(&condition_type) {
            return Err(SemanticError::ExpectedBoolean);
        }

        let _ = self.then_branch.resolve(scope, &context, &Vec::default())?;

        for (else_if_cond, else_if_scope) in &self.else_if_branches {
            let _ = else_if_cond.resolve(scope, &None, &None)?;
            let condition_type = else_if_cond.type_of(&scope.borrow())?;
            if !<EType as TypeChecking>::is_boolean(&condition_type) {
                return Err(SemanticError::ExpectedBoolean);
            }
            let _ = else_if_scope.resolve(scope, &context, &Vec::default())?;
        }

        if let Some(else_branch) = &self.else_branch {
            let _ = else_branch.resolve(scope, &context, &Vec::default())?;
        }
        Ok(())
    }
}
impl Resolve for MatchStat {
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
    {
        let _ = self.expr.resolve(scope, &None, &None)?;
        let expr_type = Some(self.expr.type_of(&scope.borrow())?);

        let exhaustive_cases = match (&expr_type.as_ref()).unwrap() {
            Either::Static(value) => match value.as_ref() {
                StaticType::Primitive(_) => None,
                StaticType::String(_) => None,
                StaticType::StrSlice(_) => None,
                _ => return Err(SemanticError::InvalidPattern),
            },
            Either::User(value) => match value.as_ref() {
                UserType::Struct(_) => return Err(SemanticError::InvalidPattern),
                UserType::Enum(Enum { id: _, values }) => Some(values.clone()),
                UserType::Union(Union { id: _, variants }) => {
                    Some(variants.iter().map(|(v, _)| v).cloned().collect())
                }
            },
        };

        match &self.else_branch {
            Some(else_branch) => {
                for value in &self.patterns {
                    let previous_vars = value.patterns[0].resolve(scope, &expr_type, &())?;
                    for pattern in &value.patterns[1..] {
                        let vars = pattern.resolve(scope, &expr_type, &())?;
                        if previous_vars != vars {
                            return Err(SemanticError::IncorrectVariant);
                        }
                    }

                    for (_index, var) in previous_vars.iter().enumerate() {
                        var.state.set(VarState::Parameter);
                        var.is_declared.set(true);
                    }
                    // create a block and Scope::child_scope())variable to it before resolving the expression
                    let _ = value.scope.resolve(scope, &context, &previous_vars)?;
                }
                let _ = else_branch.resolve(scope, &context, &Vec::default())?;
                if let Some(exhaustive_cases) = exhaustive_cases {
                    let mut map = HashMap::new();
                    for case in exhaustive_cases {
                        *map.entry(case).or_insert(0) += 1;
                    }
                    for case in self.patterns.iter().flat_map(|p| {
                        p.patterns.iter().map(|pattern| match &pattern {
                            Pattern::Primitive(_) => None,
                            Pattern::String(_) => None,
                            Pattern::Enum { typename: _, value } => Some(value),
                            Pattern::Union {
                                typename: _,
                                variant,
                                vars: _,
                            } => Some(variant),
                        })
                    }) {
                        match case {
                            Some(case) => {
                                *map.entry(case.clone()).or_insert(0) -= 1;
                            }
                            None => return Err(SemanticError::InvalidPattern),
                        }
                    }
                    if map.values().all(|&count| count == 0) {
                        return Err(SemanticError::InvalidPattern);
                    }
                }
            }
            None => {
                for value in &self.patterns {
                    let previous_vars = value.patterns[0].resolve(scope, &expr_type, &())?;
                    for pattern in &value.patterns[1..] {
                        let vars = pattern.resolve(scope, &expr_type, &())?;
                        if previous_vars != vars {
                            return Err(SemanticError::IncorrectVariant);
                        }
                    }
                    for (_index, var) in previous_vars.iter().enumerate() {
                        var.state.set(VarState::Parameter);
                        var.is_declared.set(true);
                    }
                    // create a block and Scope::child_scope())variable to it before resolving the expression
                    let _ = value.scope.resolve(scope, &context, &previous_vars)?;
                }
            }
        }
        Ok(())
    }
}

impl Resolve for TryStat {
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
    {
        let _ = self.try_branch.resolve(scope, &context, &Vec::default())?;
        if let Some(else_branch) = &self.else_branch {
            let _ = else_branch.resolve(scope, &context, &Vec::default())?;
        }
        Ok(())
    }
}
impl Resolve for CallStat {
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
    {
        self.call.resolve(scope, &None, &None)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;
    use crate::ast::TryParse;
    use crate::p_num;
    use crate::semantic::scope::scope::Scope;
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
    fn valid_if_else_if() {
        let expr = IfStat::parse(
            r##"
            if true {
                let x = 1;
            } else if false {
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
    fn valid_if_else() {
        let expr = IfStat::parse(
            r##"
            if true {
                let x = 1;
            } else {
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
                }
                case 30 => {
                    let x = 1;
                }
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
                state: Cell::default(),
                id: "x".into(),
                type_sig: p_num!(I64),
                is_declared: Cell::new(false),
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

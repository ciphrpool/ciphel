use super::{ExprFlow, FnCall, IfExpr, MatchExpr, Pattern, PatternExpr, TryExpr};
use crate::ast::expressions::data::{VarID, Variable};
use crate::resolve_metadata;
use crate::semantic::scope::type_traits::{GetSubTypes, TypeChecking};
use crate::semantic::scope::user_type_impl::{Enum, Union};
use crate::semantic::scope::var_impl::VarState;
use crate::semantic::scope::BuildStaticType;
use crate::semantic::scope::BuildVar;
use crate::semantic::{
    scope::{static_types::StaticType, user_type_impl::UserType, var_impl::Var, ScopeApi},
    CompatibleWith, Either, Resolve, SemanticError, TypeOf,
};
use crate::semantic::{EType, Info, MutRc};
use crate::vm::platform::Lib;
use std::collections::{HashMap, HashSet};
use std::{cell::RefCell, rc::Rc};
impl<Scope: ScopeApi> Resolve<Scope> for ExprFlow<Scope> {
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
            ExprFlow::If(value) => value.resolve(scope, context, extra),
            ExprFlow::Match(value) => value.resolve(scope, context, extra),
            ExprFlow::Try(value) => value.resolve(scope, context, extra),
            ExprFlow::Call(value) => value.resolve(scope, context, extra),
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for IfExpr<Scope> {
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
        let _ = self.condition.resolve(scope, context, extra)?;
        // Check if condition is a boolean
        let condition_type = self.condition.type_of(&scope.borrow())?;
        if !<EType as TypeChecking>::is_boolean(&condition_type) {
            return Err(SemanticError::ExpectedBoolean);
        }

        let _ = self.then_branch.resolve(scope, context, &Vec::default())?;
        let _ = self.else_branch.resolve(scope, context, &Vec::default())?;

        let then_branch_type = self.then_branch.type_of(&scope.borrow())?;
        let _ = then_branch_type.compatible_with(&self.else_branch, &scope.borrow())?;
        resolve_metadata!(self.metadata, self, scope, context);
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for Pattern {
    type Output = Vec<Var>;
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
            Pattern::Primitive(value) => {
                let _ = value.resolve(scope, context, extra)?;
                Ok(Vec::default())
            }
            Pattern::String(value) => {
                match context {
                    Some(context) => match context {
                        Either::Static(context) => match context.as_ref() {
                            StaticType::String(_) => {}
                            StaticType::StrSlice(_) => {}
                            _ => return Err(SemanticError::IncompatibleTypes),
                        },
                        _ => return Err(SemanticError::IncompatibleTypes),
                    },
                    _ => return Err(SemanticError::IncompatibleTypes),
                }
                let _ = value.resolve(scope, &None, extra)?;
                Ok(Vec::default())
            }
            Pattern::Enum { typename, value } => {
                let borrowed_scope = scope.borrow();
                let user_type = borrowed_scope.find_type(typename)?;
                let Some(_) = user_type.get_variant(value) else {
                    return Err(SemanticError::IncorrectVariant);
                };
                Ok(Vec::default())
            }
            Pattern::Union {
                typename,
                variant,
                vars,
            } => {
                let borrowed_scope = scope.borrow();
                let user_type = borrowed_scope.find_type(typename)?;
                let variant_type: Option<EType> = user_type.get_variant(variant);
                let Some(variant_type) = variant_type else {
                    return Err(SemanticError::CantInferType);
                };
                let mut scope_vars = Vec::with_capacity(vars.len());
                let Some(fields) = <EType as GetSubTypes>::get_fields(&variant_type) else {
                    return Err(SemanticError::InvalidPattern);
                };
                if vars.len() != fields.len() {
                    return Err(SemanticError::InvalidPattern);
                }
                for (field_name, field_type) in fields.iter() {
                    let Some(var_name) = vars.iter().find(|name| {
                        field_name
                            .clone()
                            .map(|inner| if inner == **name { Some(()) } else { None })
                            .flatten()
                            .is_some()
                    }) else {
                        return Err(SemanticError::InvalidPattern);
                    };
                    let var = <Var as BuildVar<Scope>>::build_var(var_name, field_type);
                    var.state.set(VarState::Parameter);
                    scope_vars.push(var);
                }
                Ok(scope_vars)
            } // Pattern::Struct { typename, vars } => {
              //     let borrowed_scope = scope.borrow();
              //     let user_type = borrowed_scope.find_type(typename)?;
              //     let user_type = user_type.type_of(&scope.borrow())?;
              //     let mut scope_vars = Vec::with_capacity(vars.len());
              //     let Some(fields) = <EType as GetSubTypes>::get_fields(&user_type) else {
              //         return Err(SemanticError::InvalidPattern);
              //     };
              //     if vars.len() != fields.len() {
              //         return Err(SemanticError::InvalidPattern);
              //     }
              //     for (field_name, field_type) in fields.iter() {
              //         let Some(var_name) = vars.iter().find(|name| {
              //             field_name
              //                 .clone()
              //                 .map(|inner| if inner == **name { Some(()) } else { None })
              //                 .flatten()
              //                 .is_some()
              //         }) else {
              //             return Err(SemanticError::InvalidPattern);
              //         };
              //         scope_vars.push(<Var as BuildVar<Scope>>::build_var(var_name, field_type));
              //     }
              //     Ok(scope_vars)
              // }
              // Pattern::Tuple(value) => {
              //     let mut scope_vars = Vec::with_capacity(value.len());
              //     let Some(use_type) = context else {
              //         return Err(SemanticError::CantInferType);
              //     };
              //     let Some(fields) = <EType as GetSubTypes>::get_fields(&use_type) else {
              //         return Err(SemanticError::InvalidPattern);
              //     };
              //     if value.len() != fields.len() {
              //         return Err(SemanticError::InvalidPattern);
              //     }
              //     for (index, (_, field_type)) in fields.iter().enumerate() {
              //         let var_name = &value[index];
              //         scope_vars.push(<Var as BuildVar<Scope>>::build_var(var_name, field_type));
              //     }
              //     Ok(scope_vars)
              // }
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for PatternExpr<Scope> {
    type Output = ();
    type Context = Option<EType>;
    type Extra = Option<EType>;
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
        let vars = self.pattern.resolve(scope, &extra, &())?;
        for (index, var) in vars.iter().enumerate() {
            var.state.set(VarState::Parameter);
        }
        // create a scope and assign the pattern variable to it before resolving the expression
        let _ = self.expr.resolve(scope, context, &vars)?;
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for MatchExpr<Scope> {
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

        let exhaustive_cases = match (&expr_type.as_ref()).unwrap() {
            Either::Static(value) => match value.as_ref() {
                StaticType::Primitive(_) => None,
                StaticType::String(_) => None,
                StaticType::StrSlice(_) => None,
                _ => return Err(SemanticError::InvalidPattern),
            },
            Either::User(value) => match value.as_ref() {
                UserType::Struct(_) => return Err(SemanticError::InvalidPattern),
                UserType::Enum(Enum { id, values }) => Some(values.clone()),
                UserType::Union(Union { id, variants }) => {
                    Some(variants.iter().map(|(v, _)| v).cloned().collect())
                }
            },
        };

        match &self.else_branch {
            Some(else_branch) => {
                let _ = else_branch.resolve(scope, &context, &Vec::default())?;
                for pattern in &self.patterns {
                    let _ = pattern.resolve(scope, &context, &expr_type)?;
                }

                if let Some(exhaustive_cases) = exhaustive_cases {
                    let mut map = HashMap::new();
                    for case in exhaustive_cases {
                        *map.entry(case).or_insert(0) += 1;
                    }
                    for case in self.patterns.iter().map(|p| match &p.pattern {
                        Pattern::Primitive(_) => None,
                        Pattern::String(_) => None,
                        Pattern::Enum { typename, value } => Some(value),
                        Pattern::Union {
                            typename,
                            variant,
                            vars,
                        } => Some(variant),
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

                let else_branch_type = else_branch.type_of(&scope.borrow())?;

                let (maybe_err, _) =
                    self.patterns
                        .iter()
                        .fold((None, else_branch_type), |mut previous, pattern| {
                            if let Err(e) = previous.1.compatible_with(pattern, &scope.borrow()) {
                                previous.0 = Some(e);
                                previous
                            } else {
                                match pattern.type_of(&scope.borrow()) {
                                    Ok(pattern_type) => (None, pattern_type),
                                    Err(err) => (
                                        Some(err),
                                        Either::Static(
                                            <StaticType as BuildStaticType<Scope>>::build_unit()
                                                .into(),
                                        ),
                                    ),
                                }
                            }
                        });
                if let Some(err) = maybe_err {
                    return Err(err);
                }
            }
            None => {
                let Some(exhaustive_cases) = exhaustive_cases else {
                    return Err(SemanticError::InvalidPattern);
                };
                for pattern in &self.patterns {
                    let _ = pattern.resolve(scope, &context, &expr_type)?;
                }
            }
        }

        resolve_metadata!(self.metadata, self, scope, context);
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for TryExpr<Scope> {
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
        let _ = self.try_branch.resolve(scope, context, &Vec::default())?;
        let _ = self.else_branch.resolve(scope, context, &Vec::default())?;

        let try_branch_type = self.try_branch.type_of(&scope.borrow())?;
        let _ = try_branch_type.compatible_with(&self.else_branch, &scope.borrow())?;
        resolve_metadata!(self.metadata, self, scope, context);
        Ok(())
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for FnCall<Scope> {
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
        match &self.fn_var {
            Variable::Var(VarID { id, .. }) => {
                if let Some(api) = Lib::from(id) {
                    let _ = api.resolve(scope, context, &self.params)?;
                    *self.platform.as_ref().borrow_mut() = Some(api);
                    resolve_metadata!(self.metadata, self, scope, context);
                    return Ok(());
                }
            }
            _ => {}
        }

        let _ = self.fn_var.resolve(scope, context, extra)?;
        let fn_var_type = self.fn_var.type_of(&scope.borrow())?;
        if !<EType as TypeChecking>::is_callable(&fn_var_type) {
            return Err(SemanticError::ExpectedCallable);
        }

        for (index, expr) in self.params.iter().enumerate() {
            let param_context = <EType as GetSubTypes>::get_nth(&fn_var_type, &index);
            let _ = expr.resolve(scope, &param_context, &())?;
        }

        let Some(fields) = <EType as GetSubTypes>::get_fields(&fn_var_type) else {
            return Err(SemanticError::ExpectedCallable);
        };
        if self.params.len() != fields.len() {
            return Err(SemanticError::IncorrectArguments);
        }
        for (index, (_, field_type)) in fields.iter().enumerate() {
            let _ = field_type.compatible_with(&self.params[index], &scope.borrow())?;
        }
        resolve_metadata!(self.metadata, self, scope, context);
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use std::cell::Cell;

    use crate::{
        ast::TryParse,
        semantic::scope::{
            scope_impl::Scope,
            static_types::{
                AddrType, FnType, KeyType, MapType, NumberType, PrimitiveType, SliceType,
                StaticType, StringType, VecType,
            },
            user_type_impl::{Enum, Struct, Union, UserType},
            var_impl::Var,
        },
    };

    use super::*;

    #[test]
    fn valid_if() {
        let expr = IfExpr::parse("if true then 10 else 20".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_if() {
        let expr = IfExpr::parse("if 10 then 10 else 20".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());

        let expr = IfExpr::parse("if true then 10 else 'a'".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());
    }

    #[test]
    fn valid_match_basic() {
        let expr = MatchExpr::parse(
            r##"
            match x {
                case 20 => 1,
                case 30 => 2,
                else => 3
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
                type_sig: Either::Static(
                    StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                ),
            })
            .unwrap();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let match_type = expr.type_of(&scope.borrow()).unwrap();
        assert_eq!(
            Either::Static(StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into()),
            match_type
        );

        let expr = MatchExpr::parse(
            r##"
            match x {
                case Color::RED => 1,
                case Color::GREEN => 2,
            }
        "##
            .into(),
        )
        .unwrap()
        .1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_type(
                &"Color".into(),
                UserType::Enum(Enum {
                    id: "Color".into(),
                    values: {
                        let mut res = Vec::new();
                        res.push("RED".into());
                        res.push("GREEN".into());
                        res
                    },
                }),
            )
            .unwrap();
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                state: Cell::default(),
                id: "x".into(),
                type_sig: Either::User(
                    UserType::Enum(Enum {
                        id: "Color".into(),
                        values: {
                            let mut res = Vec::new();
                            res.push("RED".into());
                            res.push("GREEN".into());
                            res
                        },
                    })
                    .into(),
                ),
            })
            .unwrap();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);
        let match_type = expr.type_of(&scope.borrow()).unwrap();
        assert_eq!(
            Either::Static(StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into()),
            match_type
        );

        let expr = MatchExpr::parse(
            r##"
            match x { 
                case "red" => Color::RED,
                case "green" => Color::GREEN,
                else => Color::YELLOW
            }
        "##
            .into(),
        )
        .unwrap()
        .1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_type(
                &"Color".into(),
                UserType::Enum(Enum {
                    id: "Color".into(),
                    values: {
                        let mut res = Vec::new();
                        res.push("RED".into());
                        res.push("GREEN".into());
                        res.push("YELLOW".into());
                        res
                    },
                }),
            )
            .unwrap();
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                state: Cell::default(),
                id: "x".into(),
                type_sig: Either::Static(StaticType::String(StringType()).into()),
            })
            .unwrap();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let match_type = expr.type_of(&scope.borrow()).unwrap();
        assert_eq!(
            Either::User(
                UserType::Enum(Enum {
                    id: "Color".into(),
                    values: {
                        let mut res = Vec::new();
                        res.push("RED".into());
                        res.push("GREEN".into());
                        res.push("YELLOW".into());
                        res
                    },
                })
                .into()
            ),
            match_type
        );
    }

    #[test]
    fn robustness_match_basic() {
        let expr = MatchExpr::parse(
            r##"
            match x { 
                case 20 => true,
                case 30 => false,
                else => 'a'
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
                type_sig: Either::Static(
                    StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                ),
            })
            .unwrap();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());

        let expr = MatchExpr::parse(
            r##"
            match x { 
                case 20 => true,
                case 'a' => false,
                else => true
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
                type_sig: Either::Static(
                    StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                ),
            })
            .unwrap();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());
    }

    #[test]
    fn valid_match_complex() {
        let expr = MatchExpr::parse(
            r##"
            match x { 
                case Geo::Point {x,y} => x + y,
                case Geo::Axe{x} => x,
            }
        "##
            .into(),
        )
        .unwrap()
        .1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_type(
                &"Geo".into(),
                UserType::Union(Union {
                    id: "Geo".into(),
                    variants: {
                        let mut res = Vec::new();
                        res.push((
                            "Point".into(),
                            Struct {
                                id: "Point".into(),
                                fields: vec![
                                    (
                                        "x".into(),
                                        Either::Static(
                                            StaticType::Primitive(PrimitiveType::Number(
                                                NumberType::U64,
                                            ))
                                            .into(),
                                        ),
                                    ),
                                    (
                                        "y".into(),
                                        Either::Static(
                                            StaticType::Primitive(PrimitiveType::Number(
                                                NumberType::U64,
                                            ))
                                            .into(),
                                        ),
                                    ),
                                ],
                            },
                        ));
                        res.push((
                            "Axe".into(),
                            Struct {
                                id: "Axe".into(),
                                fields: {
                                    let mut res = Vec::new();
                                    res.push((
                                        "x".into(),
                                        Either::Static(
                                            StaticType::Primitive(PrimitiveType::Number(
                                                NumberType::U64,
                                            ))
                                            .into(),
                                        ),
                                    ));
                                    res
                                },
                            },
                        ));
                        res
                    },
                }),
            )
            .unwrap();
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                state: Cell::default(),
                id: "x".into(),
                type_sig: Either::User(
                    UserType::Union(Union {
                        id: "Geo".into(),
                        variants: {
                            let mut res = Vec::new();
                            res.push((
                                "Point".into(),
                                Struct {
                                    id: "Point".into(),
                                    fields: {
                                        let mut res = Vec::new();
                                        res.push((
                                            "x".into(),
                                            Either::Static(
                                                StaticType::Primitive(PrimitiveType::Number(
                                                    NumberType::U64,
                                                ))
                                                .into(),
                                            ),
                                        ));
                                        res.push((
                                            "y".into(),
                                            Either::Static(
                                                StaticType::Primitive(PrimitiveType::Number(
                                                    NumberType::U64,
                                                ))
                                                .into(),
                                            ),
                                        ));
                                        res
                                    },
                                },
                            ));
                            res.push((
                                "Axe".into(),
                                Struct {
                                    id: "Axe".into(),
                                    fields: {
                                        let mut res = Vec::new();
                                        res.push((
                                            "x".into(),
                                            Either::Static(
                                                StaticType::Primitive(PrimitiveType::Number(
                                                    NumberType::U64,
                                                ))
                                                .into(),
                                            ),
                                        ));
                                        res
                                    },
                                },
                            ));
                            res
                        },
                    })
                    .into(),
                ),
            })
            .unwrap();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);
        let match_type = expr.type_of(&scope.borrow()).unwrap();
        assert_eq!(
            Either::Static(StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into()),
            match_type
        );
    }

    #[test]
    fn valid_try() {
        let expr = TryExpr::parse("try 10 else 20".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn valid_call() {
        let expr = FnCall::parse("f(10,20+20)".into())
            .expect("Parsing should have succeeded")
            .1;

        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                state: Cell::default(),
                id: "f".into(),
                type_sig: Either::Static(
                    StaticType::StaticFn(FnType {
                        params: vec![
                            Either::Static(
                                StaticType::Primitive(PrimitiveType::Number(NumberType::I64))
                                    .into(),
                            ),
                            Either::Static(
                                StaticType::Primitive(PrimitiveType::Number(NumberType::I64))
                                    .into(),
                            ),
                        ],
                        ret: Box::new(Either::Static(
                            StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                        )),
                        scope_params_size: 24,
                    })
                    .into(),
                ),
            })
            .unwrap();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let ret_type = expr.type_of(&scope.borrow()).unwrap();
        assert_eq!(
            ret_type,
            Either::Static(StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into())
        )
    }

    #[test]
    fn robustness_call() {
        let expr = FnCall::parse("f(10,20+20)".into())
            .expect("Parsing should have succeeded")
            .1;

        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                state: Cell::default(),
                id: "f".into(),
                type_sig: Either::Static(
                    StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                ),
            })
            .unwrap();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());
    }

    #[test]
    fn valid_platform() {
        let expr = FnCall::parse("left(10)".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = FnCall::parse("right(10)".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = FnCall::parse("lock(10)".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = FnCall::parse("unlock(10)".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = FnCall::parse("show(10)".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = FnCall::parse("hide(10)".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = FnCall::parse("write(10,'a')".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = FnCall::parse("clear(10)".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = FnCall::parse("append(tab,10)".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                id: "tab".into(),
                state: Cell::default(),
                type_sig: Either::Static(
                    StaticType::Vec(VecType(Box::new(Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                    ))))
                    .into(),
                ),
            })
            .expect("Var registering should have succeeded");
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = FnCall::parse("insert(obj,\"id\",10)".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                id: "obj".into(),
                state: Cell::default(),
                type_sig: Either::Static(
                    StaticType::Map(MapType {
                        keys_type: KeyType::String(StringType()),
                        values_type: Box::new(Either::Static(
                            StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                        )),
                    })
                    .into(),
                ),
            })
            .expect("Var registering should have succeeded");
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = FnCall::parse("delete(tab,10)".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                id: "tab".into(),
                state: Cell::default(),
                type_sig: Either::Static(
                    StaticType::Vec(VecType(Box::new(Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                    ))))
                    .into(),
                ),
            })
            .expect("Var registering should have succeeded");
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = FnCall::parse("delete(obj,\"id\")".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                id: "obj".into(),
                state: Cell::default(),
                type_sig: Either::Static(
                    StaticType::Map(MapType {
                        keys_type: KeyType::String(StringType()),
                        values_type: Box::new(Either::Static(
                            StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                        )),
                    })
                    .into(),
                ),
            })
            .expect("Var registering should have succeeded");
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = FnCall::parse("free(&x)".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                id: "x".into(),
                state: Cell::default(),
                type_sig: Either::Static(
                    StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                ),
            })
            .expect("Var registering should have succeeded");
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = FnCall::parse("free(x)".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                id: "x".into(),
                state: Cell::default(),
                type_sig: Either::Static(
                    StaticType::Address(AddrType(Box::new(Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                    ))))
                    .into(),
                ),
            })
            .expect("Var registering should have succeeded");
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = FnCall::parse("spawn()".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = FnCall::parse("close()".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let expr = FnCall::parse("print(10)".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_platform() {
        let expr = FnCall::parse("left()".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());

        let expr = FnCall::parse("right('e')".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());

        let expr = FnCall::parse("lock(10.5)".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());

        let expr = FnCall::parse("unlock(\"a\")".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());

        let expr = FnCall::parse("show(true)".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());

        let expr = FnCall::parse("hide(unit)".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());

        let expr = FnCall::parse("write(10,true)".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());

        let expr = FnCall::parse("clear(10.7)".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());

        let expr = FnCall::parse("append(tab,'e')".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                id: "tab".into(),
                state: Cell::default(),
                type_sig: Either::Static(
                    StaticType::Vec(VecType(Box::new(Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                    ))))
                    .into(),
                ),
            })
            .expect("Var registering should have succeeded");
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());

        let expr = FnCall::parse("insert(obj,true,10)".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                id: "obj".into(),
                state: Cell::default(),
                type_sig: Either::Static(
                    StaticType::Map(MapType {
                        keys_type: KeyType::String(StringType()),
                        values_type: Box::new(Either::Static(
                            StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                        )),
                    })
                    .into(),
                ),
            })
            .expect("Var registering should have succeeded");
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());

        let expr = FnCall::parse("delete(10,10)".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                id: "tab".into(),
                state: Cell::default(),
                type_sig: Either::Static(
                    StaticType::Vec(VecType(Box::new(Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                    ))))
                    .into(),
                ),
            })
            .expect("Var registering should have succeeded");
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());

        let expr = FnCall::parse("delete(obj,'a')".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                id: "obj".into(),
                state: Cell::default(),
                type_sig: Either::Static(
                    StaticType::Map(MapType {
                        keys_type: KeyType::String(StringType()),
                        values_type: Box::new(Either::Static(
                            StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                        )),
                    })
                    .into(),
                ),
            })
            .expect("Var registering should have succeeded");
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());

        let expr = FnCall::parse("free(x)".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                id: "x".into(),
                state: Cell::default(),
                type_sig: Either::Static(
                    StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                ),
            })
            .expect("Var registering should have succeeded");
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());

        let expr = FnCall::parse("spawn(8)".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());

        let expr = FnCall::parse("close(8)".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &());
        assert!(res.is_err());
    }
}

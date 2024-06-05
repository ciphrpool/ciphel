use super::{ExprFlow, FCall, IfExpr, MatchExpr, Pattern, PatternExpr, TryExpr};
use crate::ast::expressions::flows::FormatItem;
use crate::semantic::scope::scope::Scope;
use crate::semantic::scope::static_types::TupleType;
use crate::semantic::scope::type_traits::{GetSubTypes, TypeChecking};
use crate::semantic::scope::user_type_impl::{Enum, Union};
use crate::semantic::scope::var_impl::VarState;
use crate::semantic::scope::BuildStaticType;
use crate::semantic::scope::BuildVar;
use crate::semantic::{
    scope::{static_types::StaticType, user_type_impl::UserType, var_impl::Var},
    CompatibleWith, Either, Resolve, SemanticError, TypeOf,
};
use crate::semantic::{EType, Info, MergeType};
use crate::{e_static, p_num, resolve_metadata};
use std::collections::HashMap;

impl Resolve for ExprFlow {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match self {
            ExprFlow::If(value) => value.resolve(scope, context, extra),
            ExprFlow::Match(value) => value.resolve(scope, context, extra),
            ExprFlow::Try(value) => value.resolve(scope, context, extra),
            ExprFlow::SizeOf(value, metadata) => {
                let _ = value.resolve(scope, &(), extra);
                metadata.info = Info::Resolved {
                    context: context.clone(),
                    signature: Some(p_num!(U64)),
                };
                Ok(())
            }
            ExprFlow::FCall(value) => value.resolve(scope, context, extra),
        }
    }
}
impl Resolve for IfExpr {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let _ = self.condition.resolve(scope, context, &mut None)?;
        // Check if condition is a boolean
        let condition_type = self
            .condition
            .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
        if !<EType as TypeChecking>::is_boolean(&condition_type) {
            return Err(SemanticError::ExpectedBoolean);
        }

        let _ = self
            .then_branch
            .resolve(scope, context, &mut Vec::default())?;
        let _ = self
            .else_branch
            .resolve(scope, context, &mut Vec::default())?;

        let then_branch_type = self
            .then_branch
            .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
        let _ = then_branch_type.compatible_with(
            &self.else_branch,
            &crate::arw_read!(scope, SemanticError::ConcurrencyError)?,
        )?;
        resolve_metadata!(self.metadata.info, self, scope, context);
        Ok(())
    }
}
impl Resolve for Pattern {
    type Output = Vec<Var>;
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
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
                let borrowed_scope = crate::arw_read!(scope, SemanticError::ConcurrencyError)?;
                let user_type = borrowed_scope.find_type(typename)?;
                match user_type.as_ref() {
                    UserType::Enum(_) => {}
                    _ => return Err(SemanticError::IncorrectVariant),
                }
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
                let borrowed_scope = &crate::arw_read!(scope, SemanticError::ConcurrencyError)?;
                let user_type = borrowed_scope.find_type(typename)?;
                match user_type.as_ref() {
                    UserType::Union(_) => {}
                    _ => return Err(SemanticError::IncorrectVariant),
                }
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
                    let mut var = <Var as BuildVar>::build_var(var_name, field_type);
                    var.state = VarState::Parameter;
                    scope_vars.push(var);
                }
                Ok(scope_vars)
            } // Pattern::Struct { typename, vars } => {
              //     let borrowed_scope = block.borrow();
              //     let user_type = borrowed_scope.find_type(typename)?;
              //     let user_type = user_type.type_of(&block.borrow())?;
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
              //         scope_vars.push(<Var as BuildVar>::build_var(var_name, field_type));
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
              //         scope_vars.push(<Var as BuildVar>::build_var(var_name, field_type));
              //     }
              //     Ok(scope_vars)
              // }
        }
    }
}
impl Resolve for PatternExpr {
    type Output = ();
    type Context = Option<EType>;
    type Extra = Option<EType>;
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let mut previous_vars = self.patterns[0].resolve(scope, &extra, &mut ())?;
        for pattern in &mut self.patterns[1..] {
            let vars = pattern.resolve(scope, &extra, &mut ())?;
            if previous_vars != vars {
                return Err(SemanticError::IncorrectVariant);
            }
        }
        for (_index, var) in previous_vars.iter_mut().enumerate() {
            var.state = VarState::Parameter;
            var.is_declared = true;
        }
        // create a block and assign the pattern variable to it before resolving the expression
        let _ = self.expr.resolve(scope, context, &mut previous_vars)?;
        Ok(())
    }
}
impl Resolve for MatchExpr {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let _ = self.expr.resolve(scope, &None, &mut None)?;
        let mut expr_type = Some(
            self.expr
                .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?,
        );

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

        match &mut self.else_branch {
            Some(else_branch) => {
                let _ = else_branch.resolve(scope, &context, &mut Vec::default())?;
                for pattern in &mut self.patterns {
                    let _ = pattern.resolve(scope, &context, &mut expr_type)?;
                }

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

                let else_branch_type = else_branch
                    .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;

                let (maybe_err, _) =
                    self.patterns
                        .iter()
                        .fold((None, else_branch_type), |mut previous, pattern| {
                            let borrowed_scope =
                                crate::arw_read!(scope, SemanticError::ConcurrencyError);
                            if let Ok(borrowed_scope) = borrowed_scope {
                                if let Err(e) = previous.1.compatible_with(pattern, &borrowed_scope)
                                {
                                    previous.0 = Some(e);
                                    previous
                                } else {
                                    let borrowed_scope =
                                        crate::arw_read!(scope, SemanticError::ConcurrencyError);
                                    if let Ok(borrowed_scope) = borrowed_scope {
                                        match pattern.type_of(&borrowed_scope) {
                                            Ok(pattern_type) => (None, pattern_type),
                                            Err(err) => (
                                                Some(err),
                                                Either::Static(
                                                    <StaticType as BuildStaticType>::build_unit()
                                                        .into(),
                                                ),
                                            ),
                                        }
                                    } else {
                                        (
                                            Some(SemanticError::ConcurrencyError),
                                            Either::Static(
                                                <StaticType as BuildStaticType>::build_unit()
                                                    .into(),
                                            ),
                                        )
                                    }
                                }
                            } else {
                                previous.0 = Some(SemanticError::ConcurrencyError);
                                previous
                            }
                        });
                if let Some(err) = maybe_err {
                    return Err(err);
                }
            }
            None => {
                let Some(_exhaustive_cases) = exhaustive_cases else {
                    return Err(SemanticError::InvalidPattern);
                };
                for pattern in &mut self.patterns {
                    let _ = pattern.resolve(scope, &context, &mut expr_type)?;
                }
            }
        }

        resolve_metadata!(self.metadata.info, self, scope, context);
        Ok(())
    }
}
impl Resolve for TryExpr {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        _extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let _ = self
            .try_branch
            .resolve(scope, context, &mut Vec::default())?;
        match &mut self.else_branch {
            Some(else_branch) => else_branch.resolve(scope, context, &mut Vec::default())?,
            None => {}
        }

        let try_branch_type = self
            .try_branch
            .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
        let else_branch_type = match &self.else_branch {
            Some(else_branch) => {
                else_branch.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?
            }
            None => e_static!(StaticType::Unit),
        };
        match &try_branch_type {
            Either::Static(value) => match value.as_ref() {
                StaticType::Tuple(TupleType(types)) => {
                    if else_branch_type.is_unit() {
                        return Err(SemanticError::IncompatibleTypes);
                    }
                    if let Some(maybe_err_type) = types.last() {
                        if maybe_err_type.is_err() {
                            let ty = if types.len() == 2 {
                                types[0].clone()
                            } else {
                                let reconstructed_tuple = e_static!(StaticType::Tuple(TupleType(
                                    types[0..types.len() - 1].to_vec()
                                )));
                                reconstructed_tuple
                            };
                            let _ = ty.compatible_with(
                                &else_branch_type,
                                &crate::arw_read!(scope, SemanticError::ConcurrencyError)?,
                            )?;
                            let res_type = ty.merge(
                                &else_branch_type,
                                &crate::arw_read!(scope, SemanticError::ConcurrencyError)?,
                            )?;

                            self.pop_last_err = true;
                            self.metadata.info = Info::Resolved {
                                context: context.clone(),
                                signature: Some(res_type),
                            };
                            return Ok(());
                        } else {
                            let _ = try_branch_type.compatible_with(
                                &else_branch_type,
                                &crate::arw_read!(scope, SemanticError::ConcurrencyError)?,
                            )?;
                            let res_type = try_branch_type.merge(
                                &else_branch_type,
                                &crate::arw_read!(scope, SemanticError::ConcurrencyError)?,
                            )?;
                            self.metadata.info = Info::Resolved {
                                context: context.clone(),
                                signature: Some(res_type),
                            };
                            return Ok(());
                        }
                    } else {
                        return Err(SemanticError::CantInferType);
                    }
                }
                StaticType::Error => {
                    if else_branch_type.is_unit() {
                        self.pop_last_err = true;

                        self.metadata.info = Info::Resolved {
                            context: context.clone(),
                            signature: Some(e_static!(StaticType::Unit)),
                        };
                        return Ok(());
                    } else {
                        return Err(SemanticError::IncompatibleTypes);
                    }
                }
                _ => {
                    if else_branch_type.is_unit() {
                        return Err(SemanticError::IncompatibleTypes);
                    }
                    let _ = try_branch_type.compatible_with(
                        &else_branch_type,
                        &crate::arw_read!(scope, SemanticError::ConcurrencyError)?,
                    )?;
                    let res_type = try_branch_type.merge(
                        &else_branch_type,
                        &crate::arw_read!(scope, SemanticError::ConcurrencyError)?,
                    )?;

                    self.metadata.info = Info::Resolved {
                        context: context.clone(),
                        signature: Some(res_type),
                    };
                    return Ok(());
                }
            },
            Either::User(_) => {
                if else_branch_type.is_unit() {
                    return Err(SemanticError::IncompatibleTypes);
                }
                let _ = try_branch_type.compatible_with(
                    &else_branch_type,
                    &crate::arw_read!(scope, SemanticError::ConcurrencyError)?,
                )?;
                let res_type = try_branch_type.merge(
                    &else_branch_type,
                    &crate::arw_read!(scope, SemanticError::ConcurrencyError)?,
                )?;

                self.metadata.info = Info::Resolved {
                    context: context.clone(),
                    signature: Some(res_type),
                };
                return Ok(());
            }
        }
    }
}

impl Resolve for FCall {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        _extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        for item in &mut self.value {
            match item {
                FormatItem::Str(_) => {}
                FormatItem::Expr(expr) => {
                    let _ = expr.resolve(scope, &None, &mut None)?;
                }
            }
        }
        resolve_metadata!(self.metadata.info, self, scope, context);
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    

    use crate::{
        ast::TryParse,
        e_static, p_num,
        semantic::scope::{
            scope::Scope,
            static_types::{
                StaticType,
                StringType,
            },
            user_type_impl::{Enum, Struct, Union, UserType},
            var_impl::Var,
        },
    };

    use super::*;

    #[test]
    fn valid_if() {
        let mut expr = IfExpr::parse("if true then 10 else 20".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn robustness_if() {
        let mut expr = IfExpr::parse("if 10 then 10 else 20".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut ());
        assert!(res.is_err());

        let mut expr = IfExpr::parse("if true then 10 else 'a'".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut ());
        assert!(res.is_err());
    }

    #[test]
    fn valid_match_basic() {
        let mut expr = MatchExpr::parse(
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
        let _ = crate::arw_write!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .register_var(Var {
                state: VarState::Local,
                id: "x".to_string().into(),
                type_sig: p_num!(I64),
                is_declared: false,
            })
            .unwrap();
        let res = expr.resolve(&scope, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);

        let match_type = expr
            .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError).unwrap())
            .unwrap();
        assert_eq!(p_num!(I64), match_type);

        let mut expr = MatchExpr::parse(
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
        let _ = crate::arw_write!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .register_type(
                &"Color".to_string().into(),
                UserType::Enum(Enum {
                    id: "Color".to_string().into(),
                    values: {
                        let mut res = Vec::new();
                        res.push("RED".to_string().into());
                        res.push("GREEN".to_string().into());
                        res
                    },
                }),
            )
            .unwrap();
        let _ = crate::arw_write!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .register_var(Var {
                state: VarState::Local,
                id: "x".to_string().into(),
                type_sig: Either::User(
                    UserType::Enum(Enum {
                        id: "Color".to_string().into(),
                        values: {
                            let mut res = Vec::new();
                            res.push("RED".to_string().into());
                            res.push("GREEN".to_string().into());
                            res
                        },
                    })
                    .into(),
                ),
                is_declared: false,
            })
            .unwrap();
        let res = expr.resolve(&scope, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);
        let match_type = expr
            .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError).unwrap())
            .unwrap();
        assert_eq!(p_num!(I64), match_type);

        let mut expr = MatchExpr::parse(
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
        let _ = crate::arw_write!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .register_type(
                &"Color".to_string().into(),
                UserType::Enum(Enum {
                    id: "Color".to_string().into(),
                    values: {
                        let mut res = Vec::new();
                        res.push("RED".to_string().into());
                        res.push("GREEN".to_string().into());
                        res.push("YELLOW".to_string().into());
                        res
                    },
                }),
            )
            .unwrap();
        let _ = crate::arw_write!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .register_var(Var {
                state: VarState::Local,
                id: "x".to_string().into(),
                type_sig: e_static!(StaticType::String(StringType())),
                is_declared: false,
            })
            .unwrap();
        let res = expr.resolve(&scope, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);

        let match_type = expr
            .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError).unwrap())
            .unwrap();
        assert_eq!(
            Either::User(
                UserType::Enum(Enum {
                    id: "Color".to_string().into(),
                    values: {
                        let mut res = Vec::new();
                        res.push("RED".to_string().into());
                        res.push("GREEN".to_string().into());
                        res.push("YELLOW".to_string().into());
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
        let mut expr = MatchExpr::parse(
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
        let _ = crate::arw_write!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .register_var(Var {
                state: VarState::Local,
                id: "x".to_string().into(),
                type_sig: p_num!(I64),
                is_declared: false,
            })
            .unwrap();
        let res = expr.resolve(&scope, &None, &mut ());
        assert!(res.is_err());

        let mut expr = MatchExpr::parse(
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
        let _ = crate::arw_write!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .register_var(Var {
                state: VarState::Local,
                id: "x".to_string().into(),
                type_sig: p_num!(I64),
                is_declared: false,
            })
            .unwrap();
        let res = expr.resolve(&scope, &None, &mut ());
        assert!(res.is_err());
    }

    #[test]
    fn valid_match_complex() {
        let mut expr = MatchExpr::parse(
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
        let _ = crate::arw_write!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .register_type(
                &"Geo".to_string().into(),
                UserType::Union(Union {
                    id: "Geo".to_string().into(),
                    variants: {
                        let mut res = Vec::new();
                        res.push((
                            "Point".to_string().into(),
                            Struct {
                                id: "Point".to_string().into(),
                                fields: vec![
                                    ("x".to_string().into(), p_num!(U64)),
                                    ("y".to_string().into(), p_num!(U64)),
                                ],
                            },
                        ));
                        res.push((
                            "Axe".to_string().into(),
                            Struct {
                                id: "Axe".to_string().into(),
                                fields: {
                                    let mut res = Vec::new();
                                    res.push(("x".to_string().into(), p_num!(U64)));
                                    res
                                },
                            },
                        ));
                        res
                    },
                }),
            )
            .unwrap();
        let _ = crate::arw_write!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .register_var(Var {
                state: VarState::Local,
                id: "x".to_string().into(),
                type_sig: Either::User(
                    UserType::Union(Union {
                        id: "Geo".to_string().into(),
                        variants: {
                            let mut res = Vec::new();
                            res.push((
                                "Point".to_string().into(),
                                Struct {
                                    id: "Point".to_string().into(),
                                    fields: {
                                        let mut res = Vec::new();
                                        res.push(("x".to_string().into(), p_num!(U64)));
                                        res.push(("y".to_string().into(), p_num!(U64)));
                                        res
                                    },
                                },
                            ));
                            res.push((
                                "Axe".to_string().into(),
                                Struct {
                                    id: "Axe".to_string().into(),
                                    fields: {
                                        let mut res = Vec::new();
                                        res.push(("x".to_string().into(), p_num!(U64)));
                                        res
                                    },
                                },
                            ));
                            res
                        },
                    })
                    .into(),
                ),
                is_declared: false,
            })
            .unwrap();
        let res = expr.resolve(&scope, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);
        let match_type = expr
            .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError).unwrap())
            .unwrap();
        assert_eq!(p_num!(U64), match_type);
    }

    #[test]
    fn valid_try() {
        let mut expr = TryExpr::parse("try 10 else 20".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn valid_try_no_else() {
        let mut expr = TryExpr::parse("try Ok()".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let res = expr.resolve(&scope, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);
    }
    #[test]
    fn valid_try_tuple_err() {
        let mut expr = TryExpr::parse("try (10,Err()) else 20".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let _ = expr
            .resolve(&scope, &None, &mut ())
            .expect("Resolutionb should have succeeded");
    }

    #[test]
    fn valid_try_tuple_multi_err() {
        let mut expr = TryExpr::parse("try (10,20,Err()) else (20,30)".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let _ = expr
            .resolve(&scope, &None, &mut ())
            .expect("Resolutionb should have succeeded");
    }
    #[test]
    fn robustness_try_tuple_err() {
        let mut expr = TryExpr::parse("try (10,20,Err()) else 20".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let _ = expr
            .resolve(&scope, &None, &mut ())
            .expect_err("Resolution shoud have failed");
    }
    #[test]
    fn robustness_try_tuple_err_no_else() {
        let mut expr = TryExpr::parse("try (10,Err())".into())
            .expect("Parsing should have succeeded")
            .1;
        let scope = Scope::new();
        let _ = expr
            .resolve(&scope, &None, &mut ())
            .expect_err("Resolution shoud have failed");
    }
}

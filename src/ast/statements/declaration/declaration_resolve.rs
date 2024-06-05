use super::{Declaration, DeclaredVar, PatternVar, TypedVar};
use crate::ast::expressions::data::{Data, ExprScope};
use crate::ast::expressions::{Atomic, Expression};
use crate::ast::statements::assignation::AssignValue;
use crate::semantic::scope::scope::Scope;
use crate::semantic::scope::static_types::ClosureType;
use crate::semantic::scope::type_traits::{GetSubTypes, TypeChecking};
use crate::semantic::scope::BuildVar;
use crate::semantic::{
    scope::{static_types::StaticType, var_impl::Var},
    Resolve, SemanticError, TypeOf,
};
use crate::semantic::{ArcMutex, CompatibleWith, EType, Either};
use crate::{arw_read, arw_write};

impl Resolve for Declaration {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        _context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match self {
            Declaration::Declared(value) => {
                let _ = value.resolve(scope, &(), &mut ())?;
                let var_type = value
                    .signature
                    .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;

                let var = <Var as BuildVar>::build_var(&value.id, &var_type);
                let _ = arw_write!(scope, SemanticError::ConcurrencyError)?.register_var(var)?;
                Ok(())
            }
            Declaration::Assigned {
                left: DeclaredVar::Typed(value),
                right,
            } => {
                let _ = value.resolve(scope, &(), &mut ())?;
                let var_type = value
                    .signature
                    .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
                let var = <Var as BuildVar>::build_var(&value.id, &var_type);
                if var_type.is_any() {
                    return Err(SemanticError::CantInferType);
                }
                if value.rec {
                    /* update params size */
                    let var_type = match var_type {
                        Either::Static(value) => match value.as_ref() {
                            StaticType::Closure(ClosureType {
                                params,
                                ret,
                                closed,
                                scope_params_size,
                            }) => Either::Static(
                                StaticType::Closure(ClosureType {
                                    params: params.clone(),
                                    ret: ret.clone(),
                                    closed: closed.clone(),
                                    scope_params_size: scope_params_size + 8,
                                })
                                .into(),
                            ),
                            _ => {
                                return Err(SemanticError::IncompatibleTypes);
                            }
                        },
                        _ => {
                            return Err(SemanticError::IncompatibleTypes);
                        }
                    };
                    let var = <Var as BuildVar>::build_var(&value.id, &var_type);
                    var.is_declared.set(true);
                    match right {
                        AssignValue::Expr(expr) => match expr.as_ref() {
                            Expression::Atomic(Atomic::Data(Data::Closure(closure))) => {
                                match &closure.scope {
                                    ExprScope::Scope(s) => s.set_caller(var.clone()),
                                    ExprScope::Expr(s) => s.set_caller(var.clone()),
                                }
                            }
                            _ => {
                                return Err(SemanticError::IncompatibleTypes);
                            }
                        },
                        _ => {
                            return Err(SemanticError::IncompatibleTypes);
                        }
                    }
                    let _ =
                        arw_write!(scope, SemanticError::ConcurrencyError)?.register_var(var)?;
                    let _ = right.resolve(scope, &Some(var_type.clone()), &mut ())?;
                    let _ = var_type.compatible_with(
                        right,
                        &crate::arw_read!(scope, SemanticError::ConcurrencyError)?,
                    )?;
                } else {
                    let _ = right.resolve(scope, &Some(var_type.clone()), &mut ())?;
                    let _ =
                        arw_write!(scope, SemanticError::ConcurrencyError)?.register_var(var)?;
                    let _ = var_type.compatible_with(
                        right,
                        &crate::arw_read!(scope, SemanticError::ConcurrencyError)?,
                    )?;
                }

                Ok(())
            }
            Declaration::Assigned { left, right } => {
                let _ = right.resolve(scope, &None, extra)?;
                let right_type =
                    right.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
                let vars = left.resolve(scope, &Some(right_type), &mut ())?;
                for var in vars {
                    let _ =
                        arw_write!(scope, SemanticError::ConcurrencyError)?.register_var(var)?;
                }
                Ok(())
            }
        }
    }
}
impl Resolve for TypedVar {
    type Output = ();
    type Context = ();
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
        self.signature.resolve(scope, context, extra)
    }
}
impl Resolve for DeclaredVar {
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
            DeclaredVar::Id(id) => {
                let mut vars = Vec::with_capacity(1);
                let Some(var_type) = context else {
                    return Err(SemanticError::CantInferType);
                };
                if var_type.is_any() {
                    return Err(SemanticError::CantInferType);
                }

                let var = <Var as BuildVar>::build_var(id, var_type);
                vars.push(var);
                Ok(vars)
            }
            DeclaredVar::Typed(_) => {
                unreachable!("Path already covered in Declaration::resolve")
            }
            DeclaredVar::Pattern(value) => value.resolve(scope, context, extra),
        }
    }
}
impl Resolve for PatternVar {
    type Output = Vec<Var>;
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
        match self {
            PatternVar::StructFields { typename, vars } => {
                let borrowed_scope = arw_read!(scope, SemanticError::ConcurrencyError)?;
                let user_type = borrowed_scope.find_type(typename)?;
                let user_type = user_type
                    .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;
                let mut scope_vars = Vec::with_capacity(vars.len());
                let Some(fields) = <EType as GetSubTypes>::get_fields(&user_type) else {
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
                    if field_type.is_any() {
                        return Err(SemanticError::CantInferType);
                    }
                    scope_vars.push(<Var as BuildVar>::build_var(var_name, field_type));
                }
                Ok(scope_vars)
            }
            PatternVar::Tuple(value) => {
                let mut scope_vars = Vec::with_capacity(value.len());
                let Some(user_type) = context else {
                    return Err(SemanticError::CantInferType);
                };
                let Some(fields) = <EType as GetSubTypes>::get_fields(user_type) else {
                    return Err(SemanticError::InvalidPattern);
                };

                if value.len() != fields.len() {
                    return Err(SemanticError::InvalidPattern);
                }
                for (index, (_, field_type)) in fields.iter().enumerate() {
                    let var_name = &value[index];
                    if field_type.is_any() {
                        return Err(SemanticError::CantInferType);
                    }

                    scope_vars.push(<Var as BuildVar>::build_var(var_name, field_type));
                }
                Ok(scope_vars)
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        ast::TryParse,
        e_static, p_num,
        semantic::scope::{
            scope::Scope,
            static_types::{NumberType, PrimitiveType, StaticType},
            user_type_impl::{Struct, UserType},
        },
    };

    use super::*;

    #[test]
    fn valid_declaration() {
        let mut decl = Declaration::parse("let x:u64 = 1;".into()).unwrap().1;

        let scope = Scope::new();
        let res = decl.resolve(&scope, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);

        let x_type = arw_read!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .find_var(&"x".to_string().into())
            .unwrap();
        assert_eq!(p_num!(U64), x_type.type_sig);

        let mut decl = Declaration::parse("let x = 1.0;".into()).unwrap().1;

        let scope = Scope::new();
        let res = decl.resolve(&scope, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);

        let x_type = arw_read!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .find_var(&"x".to_string().into())
            .unwrap();
        assert_eq!(p_num!(F64), x_type.type_sig);
    }

    #[test]
    fn robustness_declaration() {
        let mut decl = Declaration::parse("let x:char = 1;".into()).unwrap().1;

        let scope = Scope::new();
        let res = decl.resolve(&scope, &None, &mut ());
        assert!(res.is_err());
    }

    #[test]
    fn valid_declaration_pattern() {
        let mut decl = Declaration::parse("let (x,y) = (1,'a');".into()).unwrap().1;

        let scope = Scope::new();
        let res = decl.resolve(&scope, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);

        let x_type = arw_read!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .find_var(&"x".to_string().into())
            .unwrap();
        assert_eq!(p_num!(I64), x_type.type_sig);
        let y_type = arw_read!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .find_var(&"y".to_string().into())
            .unwrap();
        assert_eq!(
            e_static!(StaticType::Primitive(PrimitiveType::Char)),
            y_type.type_sig
        );
    }

    #[test]
    fn valid_declaration_pattern_struct() {
        let mut decl = Declaration::parse("let Point {x,y} = Point { x : 1 , y:2};".into())
            .unwrap()
            .1;

        let scope = Scope::new();
        let _ = crate::arw_write!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .register_type(
                &"Point".to_string().into(),
                UserType::Struct(Struct {
                    id: "Point".to_string().into(),
                    fields: {
                        let mut res = Vec::new();
                        res.push(("x".to_string().into(), p_num!(I64)));
                        res.push(("y".to_string().into(), p_num!(I64)));
                        res
                    },
                }),
            )
            .unwrap();
        let res = decl.resolve(&scope, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);

        let x_type = arw_read!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .find_var(&"x".to_string().into())
            .unwrap();
        assert_eq!(p_num!(I64), x_type.type_sig);
        let y_type = arw_read!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .find_var(&"y".to_string().into())
            .unwrap();
        assert_eq!(p_num!(I64), y_type.type_sig);
    }
}

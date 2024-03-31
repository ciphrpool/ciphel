use super::{Declaration, DeclaredVar, PatternVar, TypedVar};
use crate::ast::expressions::data::{Data, ExprScope};
use crate::ast::expressions::{Atomic, Expression};
use crate::ast::statements::assignation::AssignValue;
use crate::semantic::scope::static_types::ClosureType;
use crate::semantic::scope::type_traits::{GetSubTypes, TypeChecking};
use crate::semantic::scope::BuildVar;
use crate::semantic::{
    scope::{static_types::StaticType, user_type_impl::UserType, var_impl::Var, ScopeApi},
    Resolve, SemanticError, TypeOf,
};
use crate::semantic::{EType, Either, MutRc};
use crate::vm::platform::Lib;
use std::{cell::RefCell, rc::Rc};
impl<Scope: ScopeApi> Resolve<Scope> for Declaration<Scope> {
    type Output = ();
    type Context = Option<EType>;
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
        match self {
            Declaration::Declared(value) => {
                let _ = value.resolve(scope, &(), &())?;
                let var_type = value.signature.type_of(&scope.borrow())?;

                let var = <Var as BuildVar<Scope>>::build_var(&value.id, &var_type);
                let _ = scope.borrow_mut().register_var(var)?;
                Ok(())
            }
            Declaration::Assigned {
                left: DeclaredVar::Typed(value),
                right,
            } => {
                let _ = value.resolve(scope, &(), &())?;
                let var_type = value.signature.type_of(&scope.borrow())?;
                let var = <Var as BuildVar<Scope>>::build_var(&value.id, &var_type);

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
                    let var = <Var as BuildVar<Scope>>::build_var(&value.id, &var_type);
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
                    let _ = scope.borrow_mut().register_var(var)?;
                    let _ = right.resolve(scope, &Some(var_type), &())?;
                } else {
                    let _ = right.resolve(scope, &Some(var_type), &())?;
                    let _ = scope.borrow_mut().register_var(var)?;
                }

                Ok(())
            }
            Declaration::Assigned { left, right } => {
                let _ = right.resolve(scope, &None, extra)?;
                let right_type = right.type_of(&scope.borrow())?;

                let vars = left.resolve(scope, &Some(right_type), &())?;
                for var in vars {
                    let _ = scope.borrow_mut().register_var(var)?;
                }
                Ok(())
            }
        }
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for TypedVar {
    type Output = ();
    type Context = ();
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
        if let Some(_api) = Lib::from(&self.id) {
            return Err(SemanticError::PlatformAPIOverriding);
        }
        self.signature.resolve(scope, context, extra)
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for DeclaredVar {
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
            DeclaredVar::Id(id) => {
                if let Some(_api) = Lib::from(&id) {
                    return Err(SemanticError::PlatformAPIOverriding);
                }
                let mut vars = Vec::with_capacity(1);
                let Some(var_type) = context else {
                    return Err(SemanticError::CantInferType);
                };
                if var_type.is_any() || var_type.is_unit() {
                    return Err(SemanticError::CantInferType);
                }
                if let Some(_api) = Lib::from(&id) {
                    return Err(SemanticError::PlatformAPIOverriding);
                }
                let var = <Var as BuildVar<Scope>>::build_var(id, var_type);
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
impl<Scope: ScopeApi> Resolve<Scope> for PatternVar {
    type Output = Vec<Var>;
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
        match self {
            PatternVar::StructFields { typename, vars } => {
                let borrowed_scope = scope.borrow();
                let user_type = borrowed_scope.find_type(typename)?;
                let user_type = user_type.type_of(&scope.borrow())?;
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
                    if field_type.is_any() || field_type.is_unit() {
                        return Err(SemanticError::CantInferType);
                    }
                    if let Some(_api) = Lib::from(&var_name) {
                        return Err(SemanticError::PlatformAPIOverriding);
                    }
                    scope_vars.push(<Var as BuildVar<Scope>>::build_var(var_name, field_type));
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
                    if field_type.is_any() || field_type.is_unit() {
                        return Err(SemanticError::CantInferType);
                    }
                    if let Some(_api) = Lib::from(&var_name) {
                        return Err(SemanticError::PlatformAPIOverriding);
                    }
                    scope_vars.push(<Var as BuildVar<Scope>>::build_var(var_name, field_type));
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
        semantic::scope::{
            scope_impl::Scope,
            static_types::{NumberType, PrimitiveType, StaticType},
            user_type_impl::{Struct, UserType},
        },
    };

    use super::*;

    #[test]
    fn valid_declaration() {
        let decl = Declaration::parse("let x:u64 = 1;".into()).unwrap().1;

        let scope = Scope::new();
        let res = decl.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let x_type = scope.borrow().find_var(&"x".into()).unwrap();
        assert_eq!(
            Either::Static(StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into()),
            x_type.type_sig
        );

        let decl = Declaration::parse("let x = 1.0;".into()).unwrap().1;

        let scope = Scope::new();
        let res = decl.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let x_type = scope.borrow().find_var(&"x".into()).unwrap();
        assert_eq!(
            Either::Static(StaticType::Primitive(PrimitiveType::Number(NumberType::F64)).into()),
            x_type.type_sig
        );
    }

    #[test]
    fn robustness_declaration() {
        let decl = Declaration::parse("let x:char = 1;".into()).unwrap().1;

        let scope = Scope::new();
        let res = decl.resolve(&scope, &None, &());
        assert!(res.is_err());
    }

    #[test]
    fn valid_declaration_pattern() {
        let decl = Declaration::parse("let (x,y) = (1,'a');".into()).unwrap().1;

        let scope = Scope::new();
        let res = decl.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let x_type = scope.borrow().find_var(&"x".into()).unwrap();
        assert_eq!(
            Either::Static(StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into()),
            x_type.type_sig
        );
        let y_type = scope.borrow().find_var(&"y".into()).unwrap();
        assert_eq!(
            Either::Static(StaticType::Primitive(PrimitiveType::Char).into()),
            y_type.type_sig
        );
    }

    #[test]
    fn valid_declaration_pattern_struct() {
        let decl = Declaration::parse("let Point {x,y} = Point { x : 1 , y:2};".into())
            .unwrap()
            .1;

        let scope = Scope::new();
        let _ = scope
            .borrow_mut()
            .register_type(
                &"Point".into(),
                UserType::Struct(Struct {
                    id: "Point".into(),
                    fields: {
                        let mut res = Vec::new();
                        res.push((
                            "x".into(),
                            Either::Static(
                                StaticType::Primitive(PrimitiveType::Number(NumberType::I64))
                                    .into(),
                            ),
                        ));
                        res.push((
                            "y".into(),
                            Either::Static(
                                StaticType::Primitive(PrimitiveType::Number(NumberType::I64))
                                    .into(),
                            ),
                        ));
                        res
                    },
                }),
            )
            .unwrap();
        let res = decl.resolve(&scope, &None, &());
        assert!(res.is_ok(), "{:?}", res);

        let x_type = scope.borrow().find_var(&"x".into()).unwrap();
        assert_eq!(
            Either::Static(StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into()),
            x_type.type_sig
        );
        let y_type = scope.borrow().find_var(&"y".into()).unwrap();
        assert_eq!(
            Either::Static(StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into()),
            y_type.type_sig
        );
    }
}

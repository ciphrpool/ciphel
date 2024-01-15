use super::{Declaration, DeclaredVar, PatternVar, TypedVar};
use crate::semantic::scope::type_traits::GetSubTypes;
use crate::semantic::scope::BuildVar;
use crate::semantic::EitherType;
use crate::semantic::{scope::ScopeApi, Resolve, SemanticError, TypeOf};
use std::{cell::RefCell, rc::Rc};
impl<Scope: ScopeApi> Resolve<Scope> for Declaration<Scope> {
    type Output = ();
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
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
                let var = Scope::Var::build_var(&value.id, &var_type);
                let _ = scope.borrow_mut().register_var(var)?;
                Ok(())
            }
            Declaration::Assigned {
                left: DeclaredVar::Typed(value),
                right,
            } => {
                let _ = value.resolve(scope, &(), &())?;
                let var_type = value.signature.type_of(&scope.borrow())?;
                let var = Scope::Var::build_var(&value.id, &var_type);
                let _ = right.resolve(scope, &Some(var_type), &())?;
                let _ = scope.borrow_mut().register_var(var)?;
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
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        self.signature.resolve(scope, context, extra)
    }
}
impl<Scope: ScopeApi> Resolve<Scope> for DeclaredVar {
    type Output = Vec<Scope::Var>;
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            DeclaredVar::Id(id) => {
                let mut vars = Vec::with_capacity(1);
                let Some(var_type) = context else {
                    return Err(SemanticError::CantInferType);
                };
                let var = Scope::Var::build_var(id, var_type);
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
    type Output = Vec<Scope::Var>;
    type Context = Option<EitherType<Scope::UserType, Scope::StaticType>>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &Rc<RefCell<Scope>>,
        context: &Self::Context,
        _extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
        Scope: ScopeApi,
    {
        match self {
            PatternVar::UnionInline {
                typename,
                variant,
                vars,
            } => {
                let borrowed_scope = scope.borrow();
                let user_type = borrowed_scope.find_type(typename)?;
                let variant_type: Option<EitherType<Scope::UserType, Scope::StaticType>> =
                    user_type.get_variant(variant);
                let Some(variant_type) = variant_type else {
                    return Err(SemanticError::CantInferType);
                };
                let mut scope_vars = Vec::with_capacity(vars.len());
                let Some(fields) = <EitherType<
                    <Scope as ScopeApi>::UserType,
                    <Scope as ScopeApi>::StaticType,
                > as GetSubTypes<Scope>>::get_fields(
                    &variant_type
                ) else {
                    return Err(SemanticError::InvalidPattern);
                };
                if vars.len() != fields.len() {
                    return Err(SemanticError::InvalidPattern);
                }
                for (index, (_, field_type)) in fields.iter().enumerate() {
                    let var_name = &vars[index];
                    scope_vars.push(Scope::Var::build_var(var_name, field_type));
                }
                Ok(scope_vars)
            }
            PatternVar::UnionFields {
                typename,
                variant,
                vars,
            } => {
                let borrowed_scope = scope.borrow();
                let user_type = borrowed_scope.find_type(typename)?;
                let variant_type: Option<EitherType<Scope::UserType, Scope::StaticType>> =
                    user_type.get_variant(variant);
                let Some(variant_type) = variant_type else {
                    return Err(SemanticError::CantInferType);
                };
                let mut scope_vars = Vec::with_capacity(vars.len());
                let Some(fields) = <EitherType<
                    <Scope as ScopeApi>::UserType,
                    <Scope as ScopeApi>::StaticType,
                > as GetSubTypes<Scope>>::get_fields(
                    &variant_type
                ) else {
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
                    scope_vars.push(Scope::Var::build_var(var_name, field_type));
                }
                Ok(scope_vars)
            }
            PatternVar::StructInline { typename, vars } => {
                let borrowed_scope = scope.borrow();
                let user_type = borrowed_scope.find_type(typename)?;
                let user_type = user_type.type_of(&scope.borrow())?;
                let mut scope_vars = Vec::with_capacity(vars.len());
                let Some(fields) = <EitherType<
                    <Scope as ScopeApi>::UserType,
                    <Scope as ScopeApi>::StaticType,
                > as GetSubTypes<Scope>>::get_fields(&user_type) else {
                    return Err(SemanticError::InvalidPattern);
                };
                if vars.len() != fields.len() {
                    return Err(SemanticError::InvalidPattern);
                }
                for (index, (_, field_type)) in fields.iter().enumerate() {
                    let var_name = &vars[index];
                    scope_vars.push(Scope::Var::build_var(var_name, field_type));
                }
                Ok(scope_vars)
            }
            PatternVar::StructFields { typename, vars } => {
                let borrowed_scope = scope.borrow();
                let user_type = borrowed_scope.find_type(typename)?;
                let user_type = user_type.type_of(&scope.borrow())?;
                let mut scope_vars = Vec::with_capacity(vars.len());
                let Some(fields) = <EitherType<
                    <Scope as ScopeApi>::UserType,
                    <Scope as ScopeApi>::StaticType,
                > as GetSubTypes<Scope>>::get_fields(&user_type) else {
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
                    scope_vars.push(Scope::Var::build_var(var_name, field_type));
                }
                Ok(scope_vars)
            }
            PatternVar::Tuple(value) => {
                let mut scope_vars = Vec::with_capacity(value.len());
                let Some(user_type) = context else {
                    return Err(SemanticError::CantInferType);
                };
                let Some(fields) = <EitherType<
                    <Scope as ScopeApi>::UserType,
                    <Scope as ScopeApi>::StaticType,
                > as GetSubTypes<Scope>>::get_fields(user_type) else {
                    return Err(SemanticError::InvalidPattern);
                };
                if value.len() != fields.len() {
                    return Err(SemanticError::InvalidPattern);
                }
                for (index, (_, field_type)) in fields.iter().enumerate() {
                    let var_name = &value[index];
                    scope_vars.push(Scope::Var::build_var(var_name, field_type));
                }
                Ok(scope_vars)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        ast::TryParse,
        semantic::scope::{
            scope_impl::Scope,
            static_types::{PrimitiveType, StaticType},
            user_type_impl::{Struct, UserType},
        },
    };

    use super::*;

    #[test]
    fn valid_declaration() {
        let decl = Declaration::parse("let x:number = 1;".into()).unwrap().1;

        let scope = Scope::new();
        let res = decl.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let x_type = scope.borrow().find_var(&"x".into()).unwrap();
        assert_eq!(
            EitherType::Static(StaticType::Primitive(PrimitiveType::Number).into()),
            x_type.type_sig
        );

        let decl = Declaration::parse("let x = 1.0;".into()).unwrap().1;

        let scope = Scope::new();
        let res = decl.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let x_type = scope.borrow().find_var(&"x".into()).unwrap();
        assert_eq!(
            EitherType::Static(StaticType::Primitive(PrimitiveType::Float).into()),
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
        assert!(res.is_ok());

        let x_type = scope.borrow().find_var(&"x".into()).unwrap();
        assert_eq!(
            EitherType::Static(StaticType::Primitive(PrimitiveType::Number).into()),
            x_type.type_sig
        );
        let y_type = scope.borrow().find_var(&"y".into()).unwrap();
        assert_eq!(
            EitherType::Static(StaticType::Primitive(PrimitiveType::Char).into()),
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
                            EitherType::Static(StaticType::Primitive(PrimitiveType::Number).into()),
                        ));
                        res.push((
                            "y".into(),
                            EitherType::Static(StaticType::Primitive(PrimitiveType::Number).into()),
                        ));
                        res
                    },
                }),
            )
            .unwrap();
        let res = decl.resolve(&scope, &None, &());
        assert!(res.is_ok());

        let x_type = scope.borrow().find_var(&"x".into()).unwrap();
        assert_eq!(
            EitherType::Static(StaticType::Primitive(PrimitiveType::Number).into()),
            x_type.type_sig
        );
        let y_type = scope.borrow().find_var(&"y".into()).unwrap();
        assert_eq!(
            EitherType::Static(StaticType::Primitive(PrimitiveType::Number).into()),
            y_type.type_sig
        );
    }
}

use super::{Declaration, DeclaredVar, PatternVar, TypedVar};
use crate::ast::statements::Statement;
use crate::semantic::scope::static_types::TupleType;
use crate::semantic::scope::user_types::{Struct, UserType};
use crate::semantic::{scope::static_types::StaticType, Resolve, SemanticError, TypeOf};
use crate::semantic::{CompatibleWith, Desugar, EType};

impl Resolve for Declaration {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        _context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match self {
            Declaration::Declared(value) => {
                let _ = value.resolve::<G>(scope_manager, scope_id, &(), &mut ())?;
                let var_type = value.signature.type_of(&scope_manager, scope_id)?;

                let var_id = scope_manager.register_var(value.name.as_str(), var_type, scope_id)?;
                value.id.insert(var_id);
                Ok(())
            }
            Declaration::Assigned {
                left: DeclaredVar::Typed(value),
                right,
            } => {
                let _ = value
                    .signature
                    .resolve::<G>(scope_manager, scope_id, &(), &mut ())?;
                let var_type = value.signature.type_of(&scope_manager, scope_id)?;
                if EType::Static(StaticType::Any) == var_type {
                    return Err(SemanticError::CantInferType(format!(
                        "of this variable {}",
                        value.name
                    )));
                }
                let var_id =
                    scope_manager.register_var(value.name.as_str(), var_type.clone(), scope_id)?;
                let _ = value.id.insert(var_id);

                let _ = right.resolve::<G>(
                    scope_manager,
                    scope_id,
                    &Some(var_type.clone()),
                    &mut (),
                )?;
                let _ = var_type.compatible_with(
                    &right.type_of(scope_manager, scope_id)?,
                    &scope_manager,
                    scope_id,
                )?;

                Ok(())
            }
            Declaration::Assigned { left, right } => {
                let _ = right.resolve::<G>(scope_manager, scope_id, &None, extra)?;
                let right_type = right.type_of(&scope_manager, scope_id)?;
                let _ = left.resolve::<G>(scope_manager, scope_id, &Some(right_type), &mut ())?;
                Ok(())
            }
            Declaration::RecClosure { left, right } => {
                todo!()
            }
        }
    }
}

impl Desugar<Statement> for Declaration {
    fn desugar<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Option<Statement>, SemanticError> {
        match self {
            Declaration::Declared(typed_var) => Ok(None),
            Declaration::Assigned { left, right } => {
                if let Some(output) = right.desugar::<G>(scope_manager, scope_id)? {
                    *right = output;
                }
                Ok(None)
            }
            Declaration::RecClosure { left, right } => todo!(),
        }
    }
}

impl Resolve for TypedVar {
    type Output = ();
    type Context = ();
    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        self.signature
            .resolve::<G>(scope_manager, scope_id, context, extra)
    }
}
impl Resolve for DeclaredVar {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match self {
            DeclaredVar::Id { name, id } => {
                let Some(var_type) = context else {
                    return Err(SemanticError::CantInferType(format!(
                        "of this variable {}",
                        name
                    )));
                };
                if EType::Static(StaticType::Any) == *var_type {
                    return Err(SemanticError::CantInferType(format!(
                        "of this variable {}",
                        name
                    )));
                }
                let var_id = scope_manager.register_var(name, var_type.clone(), scope_id)?;
                id.insert(var_id);
                Ok(())
            }
            DeclaredVar::Typed(_) => {
                unreachable!("Path already covered in Declaration::resolve")
            }
            DeclaredVar::Pattern(value) => {
                value.resolve::<G>(scope_manager, scope_id, context, extra)
            }
        }
    }
}
impl Resolve for PatternVar {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: &Self::Context,
        _extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match self {
            PatternVar::StructFields {
                ref typename,
                vars,
                ids,
            } => {
                let Some(EType::User { id: type_id, .. }) = context else {
                    return Err(SemanticError::IncompatibleTypes);
                };

                let UserType::Struct(Struct { fields, .. }) =
                    scope_manager.find_type_by_id(*type_id, scope_id)?.clone()
                else {
                    return Err(SemanticError::ExpectedStruct);
                };

                if vars.len() != fields.len() {
                    return Err(SemanticError::InvalidPattern);
                }

                let buffer = ids.insert(Vec::with_capacity(vars.len()));

                for (field_name, field_type) in fields {
                    let id = scope_manager.register_var(&field_name, field_type, scope_id)?;
                    buffer.push(id);
                }
                Ok(())
            }
            PatternVar::Tuple { names, ids } => {
                let Some(EType::Static(StaticType::Tuple(TupleType(types)))) = context else {
                    return Err(SemanticError::IncompatibleTypes);
                };

                if names.len() != types.len() {
                    return Err(SemanticError::InvalidPattern);
                }

                let buffer = ids.insert(Vec::with_capacity(names.len()));

                for (var_name, var_type) in names.iter().zip(types) {
                    let id = scope_manager.register_var(&var_name, var_type.clone(), scope_id)?;
                    buffer.push(id);
                }

                Ok(())
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
            scope::ScopeManager,
            static_types::{PrimitiveType, StaticType},
            user_types::{Struct, UserType},
        },
    };

    use super::*;

    #[test]
    fn valid_declaration() {
        let mut decl = Declaration::parse("let x:u64 = 1;".into()).unwrap().1;

        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res =
            decl.resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);

        let variable = scope_manager.find_var_by_name("x", None).unwrap();
        assert_eq!(p_num!(U64), variable.ctype);

        let mut decl = Declaration::parse("let x = 1.0;".into()).unwrap().1;

        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res =
            decl.resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);

        let variable = scope_manager.find_var_by_name("x", None).unwrap();
        assert_eq!(p_num!(F64), variable.ctype);
    }

    #[test]
    fn robustness_declaration() {
        let mut decl = Declaration::parse("let x:char = 1;".into()).unwrap().1;

        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res =
            decl.resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ());
        assert!(res.is_err());
    }

    #[test]
    fn valid_declaration_pattern() {
        let mut decl = Declaration::parse("let (x,y) = (1,'a');".into()).unwrap().1;

        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let res =
            decl.resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);

        let variable = scope_manager.find_var_by_name("x", None).unwrap();
        assert_eq!(p_num!(I64), variable.ctype);

        let variable = scope_manager.find_var_by_name("y", None).unwrap();
        assert_eq!(
            e_static!(StaticType::Primitive(PrimitiveType::Char)),
            variable.ctype
        );
    }

    #[test]
    fn valid_declaration_pattern_struct() {
        let mut decl = Declaration::parse("let Point {x,y} = Point { x : 1 , y:2};".into())
            .unwrap()
            .1;

        let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
        let _ = scope_manager
            .register_type(
                "Point",
                UserType::Struct(Struct {
                    id: "Point".to_string().into(),
                    fields: {
                        let mut res = Vec::new();
                        res.push(("x".to_string().into(), p_num!(I64)));
                        res.push(("y".to_string().into(), p_num!(I64)));
                        res
                    },
                }),
                None,
            )
            .unwrap();
        let res =
            decl.resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ());
        assert!(res.is_ok(), "{:?}", res);

        let variable = scope_manager.find_var_by_name("x", None).unwrap();
        assert_eq!(p_num!(I64), variable.ctype);
        let variable = scope_manager.find_var_by_name("y", None).unwrap();
        assert_eq!(p_num!(I64), variable.ctype);
    }
}

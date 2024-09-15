use super::{Definition, EnumDef, FnDef, StructDef, TypeDef, UnionDef};

use crate::ast::statements::block::BlockCommonApi;
use crate::semantic::scope::scope::ScopeState;
use crate::semantic::scope::static_types::FnType;
use crate::semantic::scope::BuildUserType;
use crate::semantic::EType;
use crate::semantic::SizeOf;
use crate::semantic::{
    scope::{static_types::StaticType, user_type_impl::UserType},
    Resolve, SemanticError, TypeOf,
};

impl Resolve for Definition {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        _context: &Self::Context,
        _extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match self {
            Definition::Type(value) => value.resolve::<G>(scope_manager, scope_id, &(), &mut ()),
            Definition::Fn(value) => value.resolve::<G>(scope_manager, scope_id, &(), &mut ()),
        }
    }
}

impl Resolve for TypeDef {
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
        let _ = match self {
            TypeDef::Struct(value) => value.resolve::<G>(scope_manager, scope_id, context, extra),
            TypeDef::Union(value) => value.resolve::<G>(scope_manager, scope_id, context, extra),
            TypeDef::Enum(value) => value.resolve::<G>(scope_manager, scope_id, context, extra),
        }?;
        let id = match &self {
            TypeDef::Struct(value) => &value.id,
            TypeDef::Union(value) => &value.id,
            TypeDef::Enum(value) => &value.id,
        };

        let type_def = UserType::build_usertype(self, &scope_manager, scope_id)?;

        let _ = scope_manager.register_type(id, type_def, scope_id)?;
        Ok(())
    }
}

impl Resolve for StructDef {
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
        for (_, type_siq) in &mut self.fields {
            let _ = type_siq.resolve::<G>(scope_manager, scope_id, context, extra)?;
        }
        Ok(())
    }
}

impl Resolve for UnionDef {
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
        for (_, variant) in &mut self.variants {
            for (_, type_sig) in variant {
                let _ = type_sig.resolve::<G>(scope_manager, scope_id, context, extra);
            }
        }

        Ok(())
    }
}

impl Resolve for EnumDef {
    type Output = ();
    type Context = ();
    type Extra = ();
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        _scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        _context: &Self::Context,
        _extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        Ok(())
    }
}

impl Resolve for FnDef {
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
        for value in &mut self.params {
            let _ = value.resolve::<G>(scope_manager, scope_id, context, extra)?;
        }

        let _ = self
            .ret
            .resolve::<G>(scope_manager, scope_id, context, extra)?;
        let return_type = self.ret.type_of(&scope_manager, scope_id)?;

        let inner_scope = self.scope.init_from_parent(scope_manager, scope_id)?;
        scope_manager
            .scope_states
            .insert(inner_scope, ScopeState::Function);

        let mut param_types = Vec::with_capacity(self.params.len());
        for arg in &self.params {
            let argtype = arg.type_of(scope_manager, scope_id)?;
            param_types.push(argtype.clone());
            let _ =
                scope_manager.register_parameter(arg.name.as_str(), argtype, Some(inner_scope))?;
        }

        let param_total_size = param_types.iter().map(|t| t.size_of()).sum();

        let fn_type_sig = EType::Static(StaticType::StaticFn(FnType {
            params: param_types,
            ret: Box::new(return_type.clone()),
            scope_params_size: param_total_size,
        }));

        let id = scope_manager.register_var(self.name.as_str(), fn_type_sig, scope_id)?;
        self.id.insert(id);

        let _ = self
            .scope
            .resolve::<G>(scope_manager, scope_id, &Some(return_type), &mut ())?;

        //let _ = return_type.compatible_with(&self.block, &block.borrow())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        ast::TryParse,
        e_static, p_num,
        semantic::scope::{
            scope,
            static_types::{FnType, StaticType, StringType},
            user_type_impl::{Enum, Struct, Union, UserType},
        },
    };

    use super::*;

    #[test]
    fn valid_struct() {
        let mut type_def = TypeDef::parse(
            r#"
            struct Point {
                x : i64,
                y : i64
            }
        "#
            .into(),
        )
        .unwrap()
        .1;
        let mut scope_manager = scope::ScopeManager::default();
        let res = type_def.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &(),
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);

        let res_type = scope_manager.find_type_by_name(&"Point", None).unwrap();

        assert_eq!(
            UserType::Struct(Struct {
                id: "Point".to_string(),
                fields: {
                    let mut res = Vec::new();
                    res.push(("x".to_string(), p_num!(I64)));
                    res.push(("y".to_string(), p_num!(I64)));
                    res
                },
            }),
            res_type.clone()
        )
        .into()
    }

    #[test]
    fn valid_advanced_struct() {
        let mut type_def = TypeDef::parse(
            r#"
            struct Line {
                start : Point,
                end : Point
            }
        "#
            .into(),
        )
        .unwrap()
        .1;
        let mut scope_manager = scope::ScopeManager::default();
        let point_id = scope_manager
            .register_type(
                "Point",
                UserType::Struct(Struct {
                    id: "Point".to_string(),
                    fields: {
                        let mut res = Vec::new();
                        res.push(("x".to_string(), p_num!(U64)));
                        res.push(("y".to_string(), p_num!(U64)));
                        res
                    },
                }),
                None,
            )
            .unwrap();
        let res = type_def.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &(),
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);

        let res_type = scope_manager.find_type_by_name(&"Line", None).unwrap();

        assert_eq!(
            UserType::Struct(Struct {
                id: "Line".to_string(),
                fields: {
                    let mut res = Vec::new();
                    res.push((
                        "start".to_string(),
                        crate::semantic::EType::User {
                            id: point_id,
                            size: res_type.size_of(),
                        },
                    ));
                    res.push((
                        "end".to_string(),
                        crate::semantic::EType::User {
                            id: point_id,
                            size: res_type.size_of(),
                        },
                    ));
                    res
                },
            }),
            res_type.clone()
        )
    }

    #[test]
    fn valid_union() {
        let mut type_def = TypeDef::parse(
            r#"
            union Geo {
                Point {
                    x : u64,
                    y : u64,
                },
                Axe {
                    x : u64,
                }
            }
        "#
            .into(),
        )
        .unwrap()
        .1;
        let mut scope_manager = scope::ScopeManager::default();
        let res = type_def.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &(),
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);

        let res_type = scope_manager.find_type_by_name(&"Geo", None).unwrap();

        assert_eq!(
            UserType::Union(Union {
                id: "Geo".to_string(),
                variants: {
                    let mut res = Vec::new();
                    res.push((
                        "Point".to_string(),
                        Struct {
                            id: "Point".to_string(),
                            fields: {
                                let mut res = Vec::new();
                                res.push(("x".to_string(), p_num!(U64)));
                                res.push(("y".to_string(), p_num!(U64)));
                                res
                            },
                        },
                    ));
                    res.push((
                        "Axe".to_string(),
                        Struct {
                            id: "Axe".to_string(),
                            fields: {
                                let mut res = Vec::new();
                                res.push(("x".to_string(), p_num!(U64)));
                                res
                            },
                        },
                    ));
                    res
                },
            }),
            res_type.clone()
        )
    }

    #[test]
    fn valid_enum() {
        let mut type_def = TypeDef::parse(
            r#"
            enum Geo {
                Point
            }
        "#
            .into(),
        )
        .unwrap()
        .1;
        let mut scope_manager = scope::ScopeManager::default();
        let res = type_def.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &(),
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);

        let res_type = scope_manager.find_type_by_name(&"Geo", None).unwrap();

        assert_eq!(
            UserType::Enum(Enum {
                id: "Geo".to_string(),
                values: {
                    let mut res = Vec::new();
                    res.push("Point".to_string());
                    res
                },
            }),
            res_type.clone()
        )
    }

    #[test]
    fn valid_function_no_args_unit() {
        let mut function = FnDef::parse(
            r##"

        fn main() -> Unit {

        }

        "##
            .into(),
        )
        .unwrap()
        .1;
        let mut scope_manager = scope::ScopeManager::default();
        let res = function.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &(),
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);

        let function_var = scope_manager
            .find_var_by_name("main", None)
            .unwrap()
            .clone();
        let function_type = &function_var.ctype;

        assert_eq!(
            *function_type,
            crate::semantic::EType::Static(
                StaticType::StaticFn(FnType {
                    params: vec![],
                    ret: Box::new(e_static!(StaticType::Unit)),
                    scope_params_size: 0,
                })
                .into()
            )
        )
    }

    #[test]
    fn valid_function_args_unit() {
        let mut function = FnDef::parse(
            r##"

        fn main(x:u64,text:String) -> Unit {

        }

        "##
            .into(),
        )
        .unwrap()
        .1;
        let mut scope_manager = scope::ScopeManager::default();
        let res = function.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &(),
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);

        let function_var = scope_manager
            .find_var_by_name(&"main", None)
            .unwrap()
            .clone();
        let function_type = &function_var.ctype;

        assert_eq!(
            *function_type,
            crate::semantic::EType::Static(
                StaticType::StaticFn(FnType {
                    params: vec![p_num!(U64), e_static!(StaticType::String(StringType()))],
                    ret: Box::new(e_static!(StaticType::Unit)),
                    scope_params_size: 16,
                })
                .into()
            )
        );

        let function_scope = function.scope;
        let x_var = scope_manager.find_var_by_name(&"x", None).unwrap();
        assert_eq!(p_num!(U64), x_var.ctype);
        let text_var = scope_manager.find_var_by_name(&"text", None).unwrap();
        assert_eq!(e_static!(StaticType::String(StringType())), text_var.ctype);
    }

    #[test]
    fn valid_function_no_args_returns() {
        let mut function = FnDef::parse(
            r##"

        fn main() -> u64 {
            let x:u64 = 10;
            return x;
        }

        "##
            .into(),
        )
        .unwrap()
        .1;
        let mut scope_manager = scope::ScopeManager::default();
        let res = function.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &(),
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);

        let function_var = scope_manager
            .find_var_by_name(&"main", None)
            .unwrap()
            .clone();
        let function_type = &function_var.ctype;

        assert_eq!(
            *function_type,
            crate::semantic::EType::Static(
                StaticType::StaticFn(FnType {
                    params: vec![],
                    ret: Box::new(p_num!(U64)),
                    scope_params_size: 8,
                })
                .into()
            )
        )
    }

    #[test]
    fn valid_function_no_args_captures() {
        let mut function = FnDef::parse(
            r##"

        fn main() -> Unit {
            x = 10;
        }

        "##
            .into(),
        )
        .unwrap()
        .1;
        let mut scope_manager = scope::ScopeManager::default();

        let _ = scope_manager
            .register_var("x", p_num!(I64), None)
            .expect("registering vars should succeed");

        let res = function.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &(),
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn valid_args_captures() {
        let mut function = FnDef::parse(
            r##"

        fn main(y:u64) -> Unit {
            x = 10;
            y = 10;
        }

        "##
            .into(),
        )
        .unwrap()
        .1;
        let mut scope_manager = scope::ScopeManager::default();

        let _ = scope_manager
            .register_var("x", p_num!(I64), None)
            .expect("registering vars should succeed");
        let _ = scope_manager
            .register_var("y", p_num!(I64), None)
            .expect("registering vars should succeed");
        let res = function.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &(),
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);
    }

    #[test]
    fn valid_function_rec_no_args_returns() {
        let mut function = FnDef::parse(
            r##"

        fn main() -> u64 {
            let x:u64 = main();
            return x;
        }

        "##
            .into(),
        )
        .unwrap()
        .1;
        let mut scope_manager = scope::ScopeManager::default();
        let res = function.resolve::<crate::vm::vm::NoopGameEngine>(
            &mut scope_manager,
            None,
            &(),
            &mut (),
        );
        assert!(res.is_ok(), "{:?}", res);

        let function_var = scope_manager
            .find_var_by_name(&"main", None)
            .unwrap()
            .clone();
        let function_type = &function_var.ctype;

        assert_eq!(
            *function_type,
            crate::semantic::EType::Static(
                StaticType::StaticFn(FnType {
                    params: vec![],
                    ret: Box::new(p_num!(U64)),
                    scope_params_size: 0,
                })
                .into()
            )
        )
    }
}

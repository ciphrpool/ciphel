use super::{Definition, EnumDef, FnDef, StructDef, TypeDef, UnionDef};

use crate::arw_write;
use crate::e_static;
use crate::semantic::scope::scope::Scope;
use crate::semantic::scope::var_impl::VarState;
use crate::semantic::scope::BuildStaticType;
use crate::semantic::scope::BuildUserType;
use crate::semantic::scope::BuildVar;
use crate::semantic::EType;
use crate::semantic::SizeOf;
use crate::semantic::{
    scope::{static_types::StaticType, user_type_impl::UserType, var_impl::Var},
    Resolve, SemanticError, TypeOf,
};

impl Resolve for Definition {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve<G:crate::GameEngineStaticFn>(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        _context: &Self::Context,
        _extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match self {
            Definition::Type(value) => value.resolve::<G>(scope, &(), &mut ()),
            Definition::Fn(value) => value.resolve::<G>(scope, &(), &mut ()),
        }
    }
}

impl Resolve for TypeDef {
    type Output = ();
    type Context = ();
    type Extra = ();
    fn resolve<G:crate::GameEngineStaticFn>(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        let _ = match self {
            TypeDef::Struct(value) => value.resolve::<G>(scope, context, extra),
            TypeDef::Union(value) => value.resolve::<G>(scope, context, extra),
            TypeDef::Enum(value) => value.resolve::<G>(scope, context, extra),
        }?;
        let id = match &self {
            TypeDef::Struct(value) => &value.id,
            TypeDef::Union(value) => &value.id,
            TypeDef::Enum(value) => &value.id,
        };

        let type_def = UserType::build_usertype(
            self,
            &crate::arw_read!(scope, SemanticError::ConcurrencyError)?,
        )?;

        let mut borrowed_scope = arw_write!(scope, SemanticError::ConcurrencyError)?;
        let _ = borrowed_scope.register_type(id, type_def)?;
        Ok(())
    }
}

impl Resolve for StructDef {
    type Output = ();
    type Context = ();
    type Extra = ();
    fn resolve<G:crate::GameEngineStaticFn>(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        for (_, type_siq) in &mut self.fields {
            let _ = type_siq.resolve::<G>(scope, context, extra)?;
        }
        Ok(())
    }
}

impl Resolve for UnionDef {
    type Output = ();
    type Context = ();
    type Extra = ();
    fn resolve<G:crate::GameEngineStaticFn>(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        for (_, variant) in &mut self.variants {
            for (_, type_sig) in variant {
                let _ = type_sig.resolve::<G>(scope, context, extra);
            }
        }

        Ok(())
    }
}

impl Resolve for EnumDef {
    type Output = ();
    type Context = ();
    type Extra = ();
    fn resolve<G:crate::GameEngineStaticFn>(
        &mut self,
        _scope: &crate::semantic::ArcRwLock<Scope>,
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
    fn resolve<G:crate::GameEngineStaticFn>(
        &mut self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        context: &Self::Context,
        extra: &mut Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        for value in &mut self.params {
            let _ = value.resolve::<G>(scope, context, extra)?;
        }

        let _ = self.ret.resolve::<G>(scope, context, extra)?;
        let return_type = self
            .ret
            .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?;

        let borrowed_scope = crate::arw_read!(scope, SemanticError::ConcurrencyError)?;

        let mut vars = self
            .params
            .iter()
            .filter_map(|param| {
                param
                    .type_of(&borrowed_scope)
                    .ok()
                    .map(|p| (param.id.clone(), p))
            })
            .enumerate()
            .map(|(_index, (id, param))| {
                let mut var = <Var as BuildVar>::build_var(&id, &param);
                var.state = VarState::Parameter;
                var.is_declared = true;
                var
            })
            .collect::<Vec<Var>>();
        drop(borrowed_scope);
        // convert to FnType -> GOAL : Retrieve function type signature
        let params = {
            let mut params = Vec::with_capacity(self.params.len());
            for p in &self.params {
                params.push(p.type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?);
            }
            params
        };
        let static_type: StaticType = StaticType::build_fn(
            &params,
            &self
                .ret
                .type_of(&crate::arw_read!(scope, SemanticError::ConcurrencyError)?)?,
            params.iter().map(|p| p.size_of()).sum::<usize>() + 8,
            &crate::arw_read!(scope, SemanticError::ConcurrencyError)?,
        )?;
        let fn_type_sig = e_static!(static_type);
        let mut var = <Var as BuildVar>::build_var(&self.id, &fn_type_sig);
        var.is_declared = true;
        self.scope.set_caller(var.clone());
        let _ = arw_write!(scope, SemanticError::ConcurrencyError)?.register_var(var)?;
        let _ = self.scope.resolve::<G>(scope, &Some(return_type), &mut vars)?;
        //let _ = return_type.compatible_with(&self.block, &block.borrow())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        arw_read,
        ast::TryParse,
        e_static, p_num,
        semantic::scope::{
            scope,
            static_types::{FnType, StaticType, StringType},
            user_type_impl::{Enum, Struct, Union, UserType},
            var_impl::Var,
        },
        vm::vm::CodeGenerationError,
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
        let scope = scope::Scope::new();
        let res = type_def.resolve::<crate::vm::vm::NoopGameEngine>(&scope, &(), &mut ());
        assert!(res.is_ok(), "{:?}", res);

        let res_type = arw_read!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .find_type(&"Point".to_string().into())
            .unwrap();

        assert_eq!(
            UserType::Struct(Struct {
                id: "Point".to_string().into(),
                fields: {
                    let mut res = Vec::new();
                    res.push(("x".to_string().into(), p_num!(I64)));
                    res.push(("y".to_string().into(), p_num!(I64)));
                    res
                },
            }),
            res_type.as_ref().clone()
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
        let scope = scope::Scope::new();
        let _ = crate::arw_write!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .register_type(
                &"Point".to_string().into(),
                UserType::Struct(Struct {
                    id: "Point".to_string().into(),
                    fields: {
                        let mut res = Vec::new();
                        res.push(("x".to_string().into(), p_num!(U64)));
                        res.push(("y".to_string().into(), p_num!(U64)));
                        res
                    },
                }),
            )
            .unwrap();
        let res = type_def.resolve::<crate::vm::vm::NoopGameEngine>(&scope, &(), &mut ());
        assert!(res.is_ok(), "{:?}", res);

        let res_type = arw_read!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .find_type(&"Line".to_string().into())
            .unwrap();

        assert_eq!(
            UserType::Struct(Struct {
                id: "Line".to_string().into(),
                fields: {
                    let mut res = Vec::new();
                    res.push((
                        "start".to_string().into(),
                        crate::semantic::Either::User(
                            UserType::Struct(Struct {
                                id: "Point".to_string().into(),
                                fields: vec![
                                    ("x".to_string().into(), p_num!(U64)),
                                    ("y".to_string().into(), p_num!(U64)),
                                ],
                            })
                            .into(),
                        ),
                    ));
                    res.push((
                        "end".to_string().into(),
                        crate::semantic::Either::User(
                            UserType::Struct(Struct {
                                id: "Point".to_string().into(),
                                fields: vec![
                                    ("x".to_string().into(), p_num!(U64)),
                                    ("y".to_string().into(), p_num!(U64)),
                                ],
                            })
                            .into(),
                        ),
                    ));
                    res
                },
            }),
            res_type.as_ref().clone()
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
        let scope = scope::Scope::new();
        let res = type_def.resolve::<crate::vm::vm::NoopGameEngine>(&scope, &(), &mut ());
        assert!(res.is_ok(), "{:?}", res);

        let res_type = arw_read!(scope, CodeGenerationError::ConcurrencyError)
            .unwrap()
            .find_type(&"Geo".to_string().into())
            .unwrap();

        assert_eq!(
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
            }),
            res_type.as_ref().clone()
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
        let scope = scope::Scope::new();
        let res = type_def.resolve::<crate::vm::vm::NoopGameEngine>(&scope, &(), &mut ());
        assert!(res.is_ok(), "{:?}", res);

        let res_type = arw_read!(scope, CodeGenerationError::ConcurrencyError)
            .unwrap()
            .find_type(&"Geo".to_string().into())
            .unwrap();

        assert_eq!(
            UserType::Enum(Enum {
                id: "Geo".to_string().into(),
                values: {
                    let mut res = Vec::new();
                    res.push("Point".to_string().into());
                    res
                },
            }),
            res_type.as_ref().clone()
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
        let scope = scope::Scope::new();
        let res = function.resolve::<crate::vm::vm::NoopGameEngine>(&scope, &(), &mut ());
        assert!(res.is_ok(), "{:?}", res);

        let function_var = arw_read!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .find_var(&"main".to_string().into())
            .unwrap()
            .clone();
        let function_var = function_var.as_ref().read().unwrap();
        let function_type = &function_var.type_sig;

        assert_eq!(
            *function_type,
            crate::semantic::Either::Static(
                StaticType::StaticFn(FnType {
                    params: vec![],
                    ret: Box::new(e_static!(StaticType::Unit)),
                    scope_params_size: 8,
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
        let scope = scope::Scope::new();
        let res = function.resolve::<crate::vm::vm::NoopGameEngine>(&scope, &(), &mut ());
        assert!(res.is_ok(), "{:?}", res);

        let function_var = arw_read!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .find_var(&"main".to_string().into())
            .unwrap()
            .clone();
        let function_var = function_var.as_ref().read().unwrap();
        let function_type = &function_var.type_sig;

        assert_eq!(
            *function_type,
            crate::semantic::Either::Static(
                StaticType::StaticFn(FnType {
                    params: vec![p_num!(U64), e_static!(StaticType::String(StringType()))],
                    ret: Box::new(e_static!(StaticType::Unit)),
                    scope_params_size: 24,
                })
                .into()
            )
        );

        let function_scope = function.scope;
        let binding = arw_read!(
            function_scope.inner_scope.clone().unwrap(),
            SemanticError::ConcurrencyError
        )
        .unwrap()
        .find_var(&"x".to_string().into())
        .unwrap();
        let x_type = binding.read().unwrap();
        assert_eq!(p_num!(U64), x_type.type_sig);
        let binding = arw_read!(
            function_scope.inner_scope.clone().unwrap(),
            SemanticError::ConcurrencyError
        )
        .unwrap()
        .find_var(&"text".to_string().into())
        .unwrap();
        let text_type = binding.read().unwrap();
        assert_eq!(
            e_static!(StaticType::String(StringType())),
            text_type.type_sig
        );
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
        let scope = scope::Scope::new();
        let res = function.resolve::<crate::vm::vm::NoopGameEngine>(&scope, &(), &mut ());
        assert!(res.is_ok(), "{:?}", res);

        let function_var = arw_read!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .find_var(&"main".to_string().into())
            .unwrap()
            .clone();
        let function_var = function_var.as_ref().read().unwrap();
        let function_type = &function_var.type_sig;

        assert_eq!(
            *function_type,
            crate::semantic::Either::Static(
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
        let scope = scope::Scope::new();

        let _ = crate::arw_write!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .register_var(Var {
                state: VarState::Local,
                id: "x".to_string().into(),
                type_sig: p_num!(I64),
                is_declared: false,
            })
            .expect("registering vars should succeed");

        let res = function.resolve::<crate::vm::vm::NoopGameEngine>(&scope, &(), &mut ());
        assert!(res.is_ok(), "{:?}", res);

        // let captured_vars = function
        //     .env
        //     .as_ref()
        //     .borrow()
        //     .clone()
        //     .values()
        //     .cloned()
        //     .map(|(v, _)| v.as_ref().clone())
        //     .collect::<Vec<Var>>();
        // assert_eq!(
        //     captured_vars,
        //     vec![Var {
        //         id: "x".to_string().into(),
        //         state: VarState::Local,
        //         type_sig: Either::Static(
        //             StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into()
        //         ),
        //     }]
        // )
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
        let scope = scope::Scope::new();

        let _ = crate::arw_write!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .register_var(Var {
                state: VarState::Local,
                id: "x".to_string().into(),
                type_sig: p_num!(U64),
                is_declared: false,
            })
            .expect("registering vars should succeed");
        let _ = crate::arw_write!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .register_var(Var {
                state: VarState::Local,
                id: "y".to_string().into(),
                type_sig: p_num!(U64),
                is_declared: false,
            })
            .expect("registering vars should succeed");
        let res = function.resolve::<crate::vm::vm::NoopGameEngine>(&scope, &(), &mut ());
        assert!(res.is_ok(), "{:?}", res);

        // let captured_vars = function
        //     .env
        //     .as_ref()
        //     .borrow()
        //     .clone()
        //     .values()
        //     .cloned()
        //     .map(|(v, _)| v.as_ref().clone())
        //     .collect::<Vec<Var>>();
        // assert_eq!(
        //     captured_vars,
        //     vec![Var {
        //         id: "x".to_string().into(),
        //         state: VarState::Local,
        //         type_sig: Either::Static(
        //             StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into()
        //         ),
        //     }]
        // )
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
        let scope = scope::Scope::new();
        let res = function.resolve::<crate::vm::vm::NoopGameEngine>(&scope, &(), &mut ());
        assert!(res.is_ok(), "{:?}", res);

        let function_var = arw_read!(scope, SemanticError::ConcurrencyError)
            .unwrap()
            .find_var(&"main".to_string().into())
            .unwrap()
            .clone();
        let function_var = function_var.as_ref().read().unwrap();
        let function_type = &function_var.type_sig;

        assert_eq!(
            *function_type,
            crate::semantic::Either::Static(
                StaticType::StaticFn(FnType {
                    params: vec![],
                    ret: Box::new(p_num!(U64)),
                    scope_params_size: 8,
                })
                .into()
            )
        )
    }
}

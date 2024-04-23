use super::{Definition, EnumDef, EventCondition, EventDef, FnDef, StructDef, TypeDef, UnionDef};

use crate::e_static;
use crate::semantic::scope::scope_impl::Scope;
use crate::semantic::scope::var_impl::VarState;
use crate::semantic::scope::BuildStaticType;
use crate::semantic::scope::BuildUserType;
use crate::semantic::scope::BuildVar;
use crate::semantic::EType;
use crate::semantic::Either;
use crate::semantic::MutRc;
use crate::semantic::SizeOf;
use crate::semantic::{
    scope::{static_types::StaticType, user_type_impl::UserType, var_impl::Var},
    Resolve, SemanticError, TypeOf,
};


impl Resolve for Definition {
    type Output = ();
    type Context = Option<EType>;
    type Extra = ();
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        _context: &Self::Context,
        _extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        match self {
            Definition::Type(value) => value.resolve(scope, &(), &()),
            Definition::Fn(value) => value.resolve(scope, &(), &()),
            Definition::Event(value) => value.resolve(scope, &(), &()),
        }
    }
}

impl Resolve for TypeDef {
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
    {
        let _ = match self {
            TypeDef::Struct(value) => value.resolve(scope, context, extra),
            TypeDef::Union(value) => value.resolve(scope, context, extra),
            TypeDef::Enum(value) => value.resolve(scope, context, extra),
        }?;
        let id = match &self {
            TypeDef::Struct(value) => &value.id,
            TypeDef::Union(value) => &value.id,
            TypeDef::Enum(value) => &value.id,
        };

        let type_def = UserType::build_usertype(self, &scope.borrow())?;

        let mut borrowed_scope = scope.borrow_mut();
        let _ = borrowed_scope.register_type(id, type_def)?;
        Ok(())
    }
}

impl Resolve for StructDef {
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
    {
        for (_, type_siq) in &self.fields {
            let _ = type_siq.resolve(scope, context, extra)?;
        }
        Ok(())
    }
}

impl Resolve for UnionDef {
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
    {
        for (_, variant) in &self.variants {
            for (_, type_sig) in variant {
                let _ = type_sig.resolve(scope, context, extra);
            }
        }

        Ok(())
    }
}

impl Resolve for EnumDef {
    type Output = ();
    type Context = ();
    type Extra = ();
    fn resolve(
        &self,
        _scope: &MutRc<Scope>,
        _context: &Self::Context,
        _extra: &Self::Extra,
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
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        for value in &self.params {
            let _ = value.resolve(scope, context, extra)?;
        }

        let _ = self.ret.resolve(scope, context, extra)?;
        let return_type = self.ret.type_of(&scope.borrow())?;

        let vars = self
            .params
            .iter()
            .filter_map(|param| {
                param
                    .type_of(&scope.borrow())
                    .ok()
                    .map(|p| (param.id.clone(), p))
            })
            .enumerate()
            .map(|(_index, (id, param))| {
                let var = <Var as BuildVar>::build_var(&id, &param);
                var.state.set(VarState::Parameter);
                var.is_declared.set(true);
                var
            })
            .collect::<Vec<Var>>();

        // convert to FnType -> GOAL : Retrieve function type signature
        let params = {
            let mut params = Vec::with_capacity(self.params.len());
            for p in &self.params {
                params.push(p.type_of(&scope.borrow())?);
            }
            params
        };
        let static_type: StaticType = StaticType::build_fn(
            &params,
            &self.ret.type_of(&scope.borrow())?,
            params.iter().map(|p| p.size_of()).sum::<usize>() + 8,
            &scope.borrow(),
        )?;
        let fn_type_sig = e_static!(static_type);
        let var = <Var as BuildVar>::build_var(&self.id, &fn_type_sig);
        var.is_declared.set(true);
        self.scope.set_caller(var.clone());
        let _ = scope.borrow_mut().register_var(var)?;
        let _ = self.scope.resolve(scope, &Some(return_type), &vars)?;
        //let _ = return_type.compatible_with(&self.block, &block.borrow())?;
        Ok(())
    }
}

impl Resolve for EventDef {
    type Output = ();
    type Context = ();
    type Extra = ();
    fn resolve(
        &self,
        _scope: &MutRc<Scope>,
        _context: &Self::Context,
        _extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl Resolve for EventCondition {
    type Output = ();
    type Context = ();
    type Extra = ();
    fn resolve(
        &self,
        _scope: &MutRc<Scope>,
        _context: &Self::Context,
        _extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError>
    where
        Self: Sized,
    {
        todo!()
    }
}

#[cfg(test)]
mod tests {

    use std::cell::Cell;

    use crate::{
        ast::TryParse,
        e_static, p_num,
        semantic::scope::{
            scope_impl,
            static_types::{FnType, NumberType, PrimitiveType, SliceType, StaticType, StringType},
            user_type_impl::{Enum, Struct, Union, UserType},
            var_impl::Var,
        },
    };

    use super::*;

    #[test]
    fn valid_struct() {
        let type_def = TypeDef::parse(
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
        let scope = scope_impl::Scope::new();
        let res = type_def.resolve(&scope, &(), &());
        assert!(res.is_ok(), "{:?}", res);

        let res_type = scope.borrow().find_type(&"Point".into()).unwrap();

        assert_eq!(
            UserType::Struct(Struct {
                id: "Point".into(),
                fields: {
                    let mut res = Vec::new();
                    res.push(("x".into(), p_num!(I64)));
                    res.push(("y".into(), p_num!(I64)));
                    res
                },
            }),
            res_type.as_ref().clone()
        )
        .into()
    }

    #[test]
    fn valid_advanced_struct() {
        let type_def = TypeDef::parse(
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
        let scope = scope_impl::Scope::new();
        let _ = scope
            .borrow_mut()
            .register_type(
                &"Point".into(),
                UserType::Struct(Struct {
                    id: "Point".into(),
                    fields: {
                        let mut res = Vec::new();
                        res.push(("x".into(), p_num!(U64)));
                        res.push(("y".into(), p_num!(U64)));
                        res
                    },
                }),
            )
            .unwrap();
        let res = type_def.resolve(&scope, &(), &());
        assert!(res.is_ok(), "{:?}", res);

        let res_type = scope.borrow().find_type(&"Line".into()).unwrap();

        assert_eq!(
            UserType::Struct(Struct {
                id: "Line".into(),
                fields: {
                    let mut res = Vec::new();
                    res.push((
                        "start".into(),
                        Either::User(
                            UserType::Struct(Struct {
                                id: "Point".into(),
                                fields: vec![("x".into(), p_num!(U64)), ("y".into(), p_num!(U64))],
                            })
                            .into(),
                        ),
                    ));
                    res.push((
                        "end".into(),
                        Either::User(
                            UserType::Struct(Struct {
                                id: "Point".into(),
                                fields: vec![("x".into(), p_num!(U64)), ("y".into(), p_num!(U64))],
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
        let type_def = TypeDef::parse(
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
        let scope = scope_impl::Scope::new();
        let res = type_def.resolve(&scope, &(), &());
        assert!(res.is_ok(), "{:?}", res);

        let res_type = scope.borrow().find_type(&"Geo".into()).unwrap();

        assert_eq!(
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
                                res.push(("x".into(), p_num!(U64)));
                                res.push(("y".into(), p_num!(U64)));
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
                                res.push(("x".into(), p_num!(U64)));
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
        let type_def = TypeDef::parse(
            r#"
            enum Geo {
                Point
            }
        "#
            .into(),
        )
        .unwrap()
        .1;
        let scope = scope_impl::Scope::new();
        let res = type_def.resolve(&scope, &(), &());
        assert!(res.is_ok(), "{:?}", res);

        let res_type = scope.borrow().find_type(&"Geo".into()).unwrap();

        assert_eq!(
            UserType::Enum(Enum {
                id: "Geo".into(),
                values: {
                    let mut res = Vec::new();
                    res.push("Point".into());
                    res
                },
            }),
            res_type.as_ref().clone()
        )
    }

    #[test]
    fn valid_function_no_args_unit() {
        let function = FnDef::parse(
            r##"

        fn main() -> Unit {

        }

        "##
            .into(),
        )
        .unwrap()
        .1;
        let scope = scope_impl::Scope::new();
        let res = function.resolve(&scope, &(), &());
        assert!(res.is_ok(), "{:?}", res);

        let function_var = scope.borrow().find_var(&"main".into()).unwrap().clone();
        let function_var = function_var.as_ref().clone();
        let function_type = function_var.type_sig;

        assert_eq!(
            function_type,
            Either::Static(
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
        let function = FnDef::parse(
            r##"

        fn main(x:u64,text:string) -> Unit {

        }

        "##
            .into(),
        )
        .unwrap()
        .1;
        let scope = scope_impl::Scope::new();
        let res = function.resolve(&scope, &(), &());
        assert!(res.is_ok(), "{:?}", res);

        let function_var = scope.borrow().find_var(&"main".into()).unwrap().clone();
        let function_var = function_var.as_ref().clone();
        let function_type = function_var.type_sig;

        assert_eq!(
            function_type,
            Either::Static(
                StaticType::StaticFn(FnType {
                    params: vec![p_num!(U64), e_static!(StaticType::String(StringType()))],
                    ret: Box::new(e_static!(StaticType::Unit)),
                    scope_params_size: 24,
                })
                .into()
            )
        );

        let function_scope = function.scope;
        let x_type = function_scope
            .inner_scope
            .borrow()
            .clone()
            .unwrap()
            .borrow()
            .find_var(&"x".into())
            .unwrap();
        assert_eq!(p_num!(U64), x_type.type_sig);
        let text_type = function_scope
            .inner_scope
            .borrow()
            .clone()
            .unwrap()
            .borrow()
            .find_var(&"text".into())
            .unwrap();
        assert_eq!(
            e_static!(StaticType::String(StringType())),
            text_type.type_sig
        );
    }

    #[test]
    fn valid_function_no_args_returns() {
        let function = FnDef::parse(
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
        let scope = scope_impl::Scope::new();
        let res = function.resolve(&scope, &(), &());
        assert!(res.is_ok(), "{:?}", res);

        let function_var = scope.borrow().find_var(&"main".into()).unwrap().clone();
        let function_var = function_var.as_ref().clone();
        let function_type = function_var.type_sig;

        assert_eq!(
            function_type,
            Either::Static(
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
        let function = FnDef::parse(
            r##"

        fn main() -> Unit {
            x = 10;
        }

        "##
            .into(),
        )
        .unwrap()
        .1;
        let scope = scope_impl::Scope::new();

        let _ = scope
            .borrow_mut()
            .register_var(Var {
                state: Cell::default(),
                id: "x".into(),
                type_sig: p_num!(I64),
                is_declared: Cell::new(false),
            })
            .expect("registering vars should succeed");

        let res = function.resolve(&scope, &(), &());
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
        //         id: "x".into(),
        //         state: Cell::default(),
        //         type_sig: Either::Static(
        //             StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into()
        //         ),
        //     }]
        // )
    }

    #[test]
    fn valid_args_captures() {
        let function = FnDef::parse(
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
        let scope = scope_impl::Scope::new();

        let _ = scope
            .borrow_mut()
            .register_var(Var {
                state: Cell::default(),
                id: "x".into(),
                type_sig: p_num!(U64),
                is_declared: Cell::new(false),
            })
            .expect("registering vars should succeed");
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                state: Cell::default(),
                id: "y".into(),
                type_sig: p_num!(U64),
                is_declared: Cell::new(false),
            })
            .expect("registering vars should succeed");
        let res = function.resolve(&scope, &(), &());
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
        //         id: "x".into(),
        //         state: Cell::default(),
        //         type_sig: Either::Static(
        //             StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into()
        //         ),
        //     }]
        // )
    }

    #[test]
    fn valid_function_rec_no_args_returns() {
        let function = FnDef::parse(
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
        let scope = scope_impl::Scope::new();
        let res = function.resolve(&scope, &(), &());
        assert!(res.is_ok(), "{:?}", res);

        let function_var = scope.borrow().find_var(&"main".into()).unwrap().clone();
        let function_var = function_var.as_ref().clone();
        let function_type = function_var.type_sig;

        assert_eq!(
            function_type,
            Either::Static(
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

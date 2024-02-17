use super::{Definition, EnumDef, EventCondition, EventDef, FnDef, StructDef, TypeDef, UnionDef};
use crate::ast::types::FnType;
use crate::ast::types::Types;
use crate::semantic::scope::BuildUserType;
use crate::semantic::scope::BuildVar;
use crate::semantic::EType;
use crate::semantic::Either;
use crate::semantic::MutRc;
use crate::semantic::{
    scope::{static_types::StaticType, user_type_impl::UserType, var_impl::Var, ScopeApi},
    Resolve, SemanticError, TypeOf,
};
use crate::vm::platform::api::PlatformApi;
use std::{cell::RefCell, rc::Rc};
impl<Scope: ScopeApi> Resolve<Scope> for Definition<Scope> {
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
        Scope: ScopeApi,
    {
        match self {
            Definition::Type(value) => value.resolve(scope, &(), &()),
            Definition::Fn(value) => value.resolve(scope, &(), &()),
            Definition::Event(value) => value.resolve(scope, &(), &()),
        }
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for TypeDef {
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

impl<Scope: ScopeApi> Resolve<Scope> for StructDef {
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
        for (_, type_siq) in &self.fields {
            let _ = type_siq.resolve(scope, context, extra)?;
        }
        Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for UnionDef {
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
        for (_, variant) in &self.variants {
            for (_, type_sig) in variant {
                let _ = type_sig.resolve(scope, context, extra);
            }
        }

        Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for EnumDef {
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
        Scope: ScopeApi,
    {
        Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for FnDef<Scope> {
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
        if let Some(_api) = PlatformApi::from(&self.id) {
            return Err(SemanticError::PlatformAPIOverriding);
        }
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
            .map(|(index, (id, param))| {
                let var = <Var as BuildVar<Scope>>::build_var(&id, &param);
                var.is_parameter.set((index, true));
                var
            })
            .collect::<Vec<Var>>();

        let _ = self.scope.resolve(scope, &Some(return_type), &vars)?;

        let env_vars = self.scope.find_outer_vars()?;
        {
            let mut borrowed_env = self.env.borrow_mut();
            borrowed_env.extend(env_vars);
        }

        //let _ = return_type.compatible_with(&self.scope, &scope.borrow())?;

        // convert to FnType -> GOAL : Retrieve function type signature
        let params = self
            .params
            .iter()
            .map(|type_var| type_var.signature.clone())
            .collect::<Types>();

        let ret = self.ret.clone();
        let fn_type = FnType { params, ret };

        let fn_type_sig = fn_type.type_of(&scope.borrow())?;
        let var = <Var as BuildVar<Scope>>::build_var(&self.id, &fn_type_sig);
        let _ = scope.borrow_mut().register_var(var)?;
        Ok(())
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for EventDef<Scope> {
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
        Scope: ScopeApi,
    {
        todo!()
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for EventCondition {
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
        Scope: ScopeApi,
    {
        todo!()
    }
}

#[cfg(test)]
mod tests {

    use std::cell::Cell;

    use crate::{
        ast::TryParse,
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
                    res.push((
                        "x".into(),
                        Either::Static(
                            StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                        ),
                    ));
                    res.push((
                        "y".into(),
                        Either::Static(
                            StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                        ),
                    ));
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
                        res.push((
                            "x".into(),
                            Either::Static(
                                StaticType::Primitive(PrimitiveType::Number(NumberType::U64))
                                    .into(),
                            ),
                        ));
                        res.push((
                            "y".into(),
                            Either::Static(
                                StaticType::Primitive(PrimitiveType::Number(NumberType::U64))
                                    .into(),
                            ),
                        ));
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
                            })
                            .into(),
                        ),
                    ));
                    res.push((
                        "end".into(),
                        Either::User(
                            UserType::Struct(Struct {
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
        let function = FnDef::<scope_impl::Scope>::parse(
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

        let (function_var, _) = scope.borrow().find_var(&"main".into()).unwrap().clone();
        let function_var = function_var.as_ref().clone();
        let function_type = function_var.type_sig;

        assert_eq!(
            function_type,
            Either::Static(
                StaticType::Fn(FnType {
                    params: vec![],
                    ret: Box::new(Either::Static(StaticType::Unit.into()))
                })
                .into()
            )
        )
    }

    #[test]
    fn valid_function_args_unit() {
        let function = FnDef::<scope_impl::Scope>::parse(
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

        let (function_var, _) = scope.borrow().find_var(&"main".into()).unwrap().clone();
        let function_var = function_var.as_ref().clone();
        let function_type = function_var.type_sig;

        assert_eq!(
            function_type,
            Either::Static(
                StaticType::Fn(FnType {
                    params: vec![
                        Either::Static(
                            StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into()
                        ),
                        Either::Static(StaticType::String(StringType()).into())
                    ],
                    ret: Box::new(Either::Static(StaticType::Unit.into()))
                })
                .into()
            )
        );

        let function_scope = function.scope;
        let (x_type, _) = function_scope
            .inner_scope
            .borrow()
            .clone()
            .unwrap()
            .borrow()
            .find_var(&"x".into())
            .unwrap();
        assert_eq!(
            Either::Static(StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into()),
            x_type.type_sig
        );
        let (text_type, _) = function_scope
            .inner_scope
            .borrow()
            .clone()
            .unwrap()
            .borrow()
            .find_var(&"text".into())
            .unwrap();
        assert_eq!(
            Either::Static(StaticType::String(StringType()).into()),
            text_type.type_sig
        );
    }

    #[test]
    fn valid_function_no_args_returns() {
        let function = FnDef::<scope_impl::Scope>::parse(
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

        let (function_var, _) = scope.borrow().find_var(&"main".into()).unwrap().clone();
        let function_var = function_var.as_ref().clone();
        let function_type = function_var.type_sig;

        assert_eq!(
            function_type,
            Either::Static(
                StaticType::Fn(FnType {
                    params: vec![],
                    ret: Box::new(Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into()
                    ))
                })
                .into()
            )
        )
    }

    #[test]
    fn valid_function_no_args_captures() {
        let function = FnDef::<scope_impl::Scope>::parse(
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
                captured: Cell::new(false),
                is_parameter: Cell::new((0, false)),
                address: Cell::new(None),
                id: "x".into(),
                type_sig: Either::Static(
                    StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into(),
                ),
            })
            .expect("registering vars should succeed");

        let res = function.resolve(&scope, &(), &());
        assert!(res.is_ok(), "{:?}", res);

        let captured_vars = function
            .env
            .as_ref()
            .borrow()
            .clone()
            .values()
            .cloned()
            .map(|v| v.as_ref().clone())
            .collect::<Vec<Var>>();
        assert_eq!(
            captured_vars,
            vec![Var {
                id: "x".into(),
                address: Cell::new(None),
                captured: Cell::new(false),
                is_parameter: Cell::new((0, false)),
                type_sig: Either::Static(
                    StaticType::Primitive(PrimitiveType::Number(NumberType::I64)).into()
                ),
            }]
        )
    }

    #[test]
    fn valid_args_captures() {
        let function = FnDef::<scope_impl::Scope>::parse(
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
                captured: Cell::new(false),
                is_parameter: Cell::new((0, false)),
                address: Cell::new(None),
                id: "x".into(),
                type_sig: Either::Static(
                    StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
                ),
            })
            .expect("registering vars should succeed");
        let _ = scope
            .borrow_mut()
            .register_var(Var {
                captured: Cell::new(false),
                is_parameter: Cell::new((0, false)),
                address: Cell::new(None),
                id: "y".into(),
                type_sig: Either::Static(
                    StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
                ),
            })
            .expect("registering vars should succeed");
        let res = function.resolve(&scope, &(), &());
        assert!(res.is_ok(), "{:?}", res);

        let captured_vars = function
            .env
            .as_ref()
            .borrow()
            .clone()
            .values()
            .cloned()
            .map(|v| v.as_ref().clone())
            .collect::<Vec<Var>>();
        assert_eq!(
            captured_vars,
            vec![Var {
                id: "x".into(),
                captured: Cell::new(false),
                is_parameter: Cell::new((0, false)),
                address: Cell::new(None),
                type_sig: Either::Static(
                    StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into()
                ),
            }]
        )
    }
}

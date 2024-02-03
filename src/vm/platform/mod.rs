use crate::semantic::scope::static_types::FnType;

use super::{allocator::Memory, vm::RuntimeError};

pub mod names {
    pub const LEFT: &str = "left";
    pub const RIGHT: &str = "right";
    pub const LOCK: &str = "lock";
    pub const UNLOCK: &str = "unlock";
    pub const SHOW: &str = "show";
    pub const HIDE: &str = "hide";
    pub const WRITE: &str = "write";
    pub const CLEAR: &str = "clear";
    pub const APPEND: &str = "append";
    pub const INSERT: &str = "insert";
    pub const DELETE: &str = "delete";
    pub const FREE: &str = "free";
    pub const SPAWN: &str = "spawn";
    pub const CLOSE: &str = "close";
    pub const PRINT: &str = "print";
}

pub struct PlatformFunction {
    name: &'static str,
    signature: FnType,
}

pub trait PlatformExecutor {
    fn left(memory: &Memory) -> Result<(), RuntimeError>;
    fn right(memory: &Memory) -> Result<(), RuntimeError>;
    fn lock(memory: &Memory) -> Result<(), RuntimeError>;
    fn unlock(memory: &Memory) -> Result<(), RuntimeError>;
    fn show(memory: &Memory) -> Result<(), RuntimeError>;
    fn hide(memory: &Memory) -> Result<(), RuntimeError>;
    fn write(memory: &Memory) -> Result<(), RuntimeError>;
    fn clear(memory: &Memory) -> Result<(), RuntimeError>;
    fn append(memory: &Memory) -> Result<(), RuntimeError>;
    fn insert(memory: &Memory) -> Result<(), RuntimeError>;
    fn delete(memory: &Memory) -> Result<(), RuntimeError>;
    fn free(memory: &Memory) -> Result<(), RuntimeError>;
    fn spawn(memory: &Memory) -> Result<(), RuntimeError>;
    fn close(memory: &Memory) -> Result<(), RuntimeError>;
    fn print(memory: &Memory) -> Result<(), RuntimeError>;
}

pub mod api {
    use std::{
        cell::{Ref, RefCell},
        rc::Rc,
    };

    use crate::{
        ast::{
            expressions::{Expression},
            utils::strings::ID,
        },
        semantic::{
            scope::{
                static_types::StaticType,
                type_traits::{GetSubTypes, TypeChecking},
                user_type_impl::UserType,
                BuildStaticType, ScopeApi,
            },
            CompatibleWith, Either, SemanticError, TypeOf,
        },
    };

    use super::names;

    pub enum PlatformApi {
        LEFT,
        RIGHT,
        LOCK,
        UNLOCK,
        SHOW,
        HIDE,
        WRITE,
        CLEAR,
        APPEND,
        INSERT,
        DELETE,
        FREE,
        SPAWN,
        CLOSE,
        PRINT,
    }

    impl PlatformApi {
        pub fn from(id: &ID) -> Option<Self> {
            match id.as_str() {
                names::LEFT => Some(Self::LEFT),
                names::RIGHT => Some(Self::RIGHT),
                names::LOCK => Some(Self::LOCK),
                names::UNLOCK => Some(Self::UNLOCK),
                names::SHOW => Some(Self::SHOW),
                names::HIDE => Some(Self::HIDE),
                names::WRITE => Some(Self::WRITE),
                names::CLEAR => Some(Self::CLEAR),
                names::APPEND => Some(Self::APPEND),
                names::INSERT => Some(Self::INSERT),
                names::DELETE => Some(Self::DELETE),
                names::FREE => Some(Self::FREE),
                names::SPAWN => Some(Self::SPAWN),
                names::CLOSE => Some(Self::CLOSE),
                names::PRINT => Some(Self::PRINT),
                _ => None,
            }
        }

        fn accept<Scope: ScopeApi>(
            &self,
            args: &Vec<Expression<Scope>>,
            scope: &Ref<Scope>,
        ) -> Result<(), SemanticError> {
            match self {
                PlatformApi::LEFT
                | PlatformApi::RIGHT
                | PlatformApi::LOCK
                | PlatformApi::UNLOCK
                | PlatformApi::SHOW
                | PlatformApi::HIDE
                | PlatformApi::CLEAR => {
                    if args.len() != 1 {
                        return Err(SemanticError::IncorrectArguments);
                    }
                    let arg = args.first().unwrap();
                    let arg_type = arg.type_of(scope)?;
                    if !<Either<UserType, StaticType> as TypeChecking>::is_u64(&arg_type) {
                        return Err(SemanticError::IncorrectArguments);
                    }
                    Ok(())
                }
                PlatformApi::WRITE => {
                    if args.len() != 2 {
                        return Err(SemanticError::IncorrectArguments);
                    }
                    let cell = &args[0];
                    let cell_type = cell.type_of(scope)?;
                    if !<Either<UserType, StaticType> as TypeChecking>::is_u64(&cell_type) {
                        return Err(SemanticError::IncorrectArguments);
                    }
                    let data = &args[1];
                    let data_type = data.type_of(scope)?;
                    if !<Either<UserType, StaticType> as TypeChecking>::is_char(&data_type) {
                        return Err(SemanticError::IncorrectArguments);
                    }
                    Ok(())
                }
                PlatformApi::APPEND => {
                    if args.len() != 2 {
                        return Err(SemanticError::IncorrectArguments);
                    }
                    let vector = &args[0];
                    let vector_type = vector.type_of(scope)?;
                    let element = &args[1];
                    let element_type = element.type_of(scope)?;
                    if !<Either<UserType, StaticType> as TypeChecking>::is_vec(&vector_type) {
                        return Err(SemanticError::IncorrectArguments);
                    }
                    let item_type =
                        <Either<UserType, StaticType> as GetSubTypes>::get_item(&vector_type)
                            .unwrap();
                    let _ = item_type.compatible_with(&element_type, scope)?;
                    Ok(())
                }
                PlatformApi::INSERT => {
                    if args.len() != 3 {
                        return Err(SemanticError::IncorrectArguments);
                    }
                    let map = &args[0];
                    let map_type = map.type_of(scope)?;

                    let expr_key = &args[1];
                    let expr_key_type = expr_key.type_of(scope)?;

                    let expr_value = &args[2];
                    let expr_value_type = expr_value.type_of(scope)?;

                    if !<Either<UserType, StaticType> as TypeChecking>::is_map(&map_type) {
                        return Err(SemanticError::IncorrectArguments);
                    }
                    let value_type =
                        <Either<UserType, StaticType> as GetSubTypes>::get_item(&map_type).unwrap();

                    let key_type =
                        <Either<UserType, StaticType> as GetSubTypes>::get_key(&map_type).unwrap();

                    let _ = key_type.compatible_with(&expr_key_type, scope)?;
                    let _ = value_type.compatible_with(&expr_value_type, scope)?;
                    Ok(())
                }
                PlatformApi::DELETE => {
                    if args.len() != 2 {
                        return Err(SemanticError::IncorrectArguments);
                    }
                    let iterator = &args[0];
                    let iterator_type = iterator.type_of(scope)?;

                    let expr_key = &args[1];
                    let expr_key_type = expr_key.type_of(scope)?;

                    if <Either<UserType, StaticType> as TypeChecking>::is_map(&iterator_type) {
                        let key_type =
                            <Either<UserType, StaticType> as GetSubTypes>::get_key(&iterator_type)
                                .unwrap();

                        let _ = key_type.compatible_with(&expr_key_type, scope)?;
                    } else if <Either<UserType, StaticType> as TypeChecking>::is_vec(&iterator_type)
                    {
                        if !<Either<UserType, StaticType> as TypeChecking>::is_u64(&expr_key_type) {
                            return Err(SemanticError::IncorrectArguments);
                        }
                    } else {
                        return Err(SemanticError::IncorrectArguments);
                    }

                    Ok(())
                }
                PlatformApi::FREE => {
                    if args.len() != 1 {
                        return Err(SemanticError::IncorrectArguments);
                    }
                    let arg = args.first().unwrap();
                    let arg_type = arg.type_of(scope)?;
                    if !<Either<UserType, StaticType> as TypeChecking>::is_addr(&arg_type) {
                        return Err(SemanticError::IncorrectArguments);
                    }
                    Ok(())
                }
                PlatformApi::SPAWN | PlatformApi::CLOSE => {
                    if args.len() != 0 {
                        return Err(SemanticError::IncorrectArguments);
                    }
                    Ok(())
                }
                PlatformApi::PRINT => Ok(()),
            }
        }

        pub fn resolve<Scope: ScopeApi>(
            &self,
            args: &Vec<Expression<Scope>>,
            scope: &Rc<RefCell<Scope>>,
        ) -> Result<(), SemanticError> {
            let _ = self.accept(args, &scope.borrow())?;
            Ok(())
        }

        pub fn returns<Scope: ScopeApi>(self) -> Either<UserType, StaticType> {
            Either::Static(<StaticType as BuildStaticType<Scope>>::build_unit().into())
        }
    }
}

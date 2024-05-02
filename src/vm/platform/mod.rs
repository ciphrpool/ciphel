use std::cell::Ref;

use crate::{
    ast::expressions::Expression,
    semantic::{EType, MutRc, Resolve, SemanticError, TypeOf},
};

use self::{
    core::{CoreCasm, CoreFn},
    stdlib::{StdCasm, StdFn},
};
use crate::semantic::scope::scope::Scope;

use super::{
    allocator::{heap::Heap, stack::Stack},
    casm::CasmProgram,
    stdio::StdIO,
    vm::{CasmMetadata, CodeGenerationError, Executable, GenerateCode, RuntimeError},
};

pub mod core;
pub mod stdlib;
pub mod utils;

#[derive(Debug, Clone, PartialEq)]
pub enum Lib {
    Core(CoreFn),
    Std(StdFn),
}

#[derive(Debug, Clone, PartialEq)]
pub enum LibCasm {
    Core(CoreCasm),
    Std(StdCasm),
}

impl CasmMetadata for LibCasm {
    fn name(&self, stdio: &mut StdIO, program: &CasmProgram) {
        match self {
            LibCasm::Core(value) => value.name(stdio, program),
            LibCasm::Std(value) => value.name(stdio, program),
        }
    }
}

impl Lib {
    pub fn from(suffixe: &Option<String>, id: &String) -> Option<Self> {
        if let Some(value) = CoreFn::from(suffixe, id) {
            return Some(Lib::Core(value));
        }
        if let Some(value) = StdFn::from(suffixe, id) {
            return Some(Lib::Std(value));
        }
        None
    }
}

impl Resolve for Lib {
    type Output = ();
    type Context = Option<EType>;
    type Extra = Vec<Expression>;
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError> {
        match self {
            Lib::Core(value) => value.resolve(scope, context, extra),
            Lib::Std(value) => value.resolve(scope, context, extra),
        }
    }
}

impl TypeOf for Lib {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Self: Sized + Resolve,
    {
        match self {
            Lib::Core(value) => value.type_of(scope),
            Lib::Std(value) => value.type_of(scope),
        }
    }
}

impl GenerateCode for Lib {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Lib::Core(value) => value.gencode(scope, instructions),
            Lib::Std(value) => value.gencode(scope, instructions),
        }
    }
}
impl Executable for LibCasm {
    fn execute(
        &self,
        program: &CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
    ) -> Result<(), RuntimeError> {
        match self {
            LibCasm::Core(value) => value.execute(program, stack, heap, stdio),
            LibCasm::Std(value) => value.execute(program, stack, heap, stdio),
        }
    }
}

// pub mod lexem {
//     pub const LEFT: &str = "left";
//     pub const RIGHT: &str = "right";
//     pub const LOCK: &str = "lock";
//     pub const UNLOCK: &str = "unlock";
//     pub const SHOW: &str = "show";
//     pub const HIDE: &str = "hide";
//     pub const WRITE: &str = "write";
//     pub const CLEAR: &str = "clear";
//     pub const APPEND: &str = "append";
//     pub const INSERT: &str = "insert";
//     pub const DELETE: &str = "delete";
//     pub const FREE: &str = "free";
//     pub const SPAWN: &str = "spawn_with_scope";
//     pub const CLOSE: &str = "close";
//     pub const PRINT: &str = "print";
// }

// pub trait PlatformExecutor {
//     fn left(memory: &Memory) -> Result<(), RuntimeError>;
//     fn right(memory: &Memory) -> Result<(), RuntimeError>;
//     fn lock(memory: &Memory) -> Result<(), RuntimeError>;
//     fn unlock(memory: &Memory) -> Result<(), RuntimeError>;
//     fn show(memory: &Memory) -> Result<(), RuntimeError>;
//     fn hide(memory: &Memory) -> Result<(), RuntimeError>;
//     fn write(memory: &Memory) -> Result<(), RuntimeError>;
//     fn clear(memory: &Memory) -> Result<(), RuntimeError>;
//     fn append(memory: &Memory) -> Result<(), RuntimeError>;
//     fn insert(memory: &Memory) -> Result<(), RuntimeError>;
//     fn delete(memory: &Memory) -> Result<(), RuntimeError>;
//     fn free(memory: &Memory) -> Result<(), RuntimeError>;
//     fn spawn_with_scope(memory: &Memory) -> Result<(), RuntimeError>;
//     fn close(memory: &Memory) -> Result<(), RuntimeError>;
//     fn print(memory: &Memory) -> Result<(), RuntimeError>;
// }

// pub mod api {
//     use std::{
//         cell::{Ref, RefCell},
//         rc::Rc,
//     };

//     use crate::{
//         ast::{expressions::Expression, utils::strings::ID},
//         semantic::{
//             block::{
//                 static_types::{NumberType, PrimitiveType, StaticType},
//                 type_traits::{GetSubTypes, TypeChecking},
//                 user_type_impl::UserType,
//                 BuildStaticType,
//             },
//             CompatibleWith, EType, Either, MutRc<Scope>, Resolve, SemanticError, TypeOf,
//         },
//     };

//     use super::lexem;

//     pub enum PlatformApi {
//         LEFT,
//         RIGHT,
//         LOCK,
//         UNLOCK,
//         SHOW,
//         HIDE,
//         WRITE,
//         CLEAR,
//         APPEND,
//         INSERT,
//         DELETE,
//         FREE,
//         SPAWN,
//         CLOSE,
//         PRINT,
//     }

//     impl PlatformApi {
//         pub fn from(id: &ID) -> Option<Self> {
//             match id.as_str() {
//                 lexem::LEFT => Some(Self::LEFT),
//                 lexem::RIGHT => Some(Self::RIGHT),
//                 lexem::LOCK => Some(Self::LOCK),
//                 lexem::UNLOCK => Some(Self::UNLOCK),
//                 lexem::SHOW => Some(Self::SHOW),
//                 lexem::HIDE => Some(Self::HIDE),
//                 lexem::WRITE => Some(Self::WRITE),
//                 lexem::CLEAR => Some(Self::CLEAR),
//                 lexem::APPEND => Some(Self::APPEND),
//                 lexem::INSERT => Some(Self::INSERT),
//                 lexem::DELETE => Some(Self::DELETE),
//                 lexem::FREE => Some(Self::FREE),
//                 lexem::SPAWN => Some(Self::SPAWN),
//                 lexem::CLOSE => Some(Self::CLOSE),
//                 lexem::PRINT => Some(Self::PRINT),
//                 _ => None,
//             }
//         }

//         fn accept(
//             &self,
//             args: &Vec<Expression>,
//             block: &MutRc<Scope>,
//         ) -> Result<(), SemanticError> {
//             match self {
//                 PlatformApi::LEFT
//                 | PlatformApi::RIGHT
//                 | PlatformApi::LOCK
//                 | PlatformApi::UNLOCK
//                 | PlatformApi::SHOW
//                 | PlatformApi::HIDE
//                 | PlatformApi::CLEAR => {
//                     if args.len() != 1 {
//                         return Err(SemanticError::IncorrectArguments);
//                     }
//                     let arg = args.first().unwrap();
//                     let _ = arg.resolve(
//                         block,
//                         &Some(Either::Static(
//                             StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
//                         )),
//                         &(),
//                     )?;
//                     let arg_type = arg.type_of(&block.borrow())?;
//                     if !<EType as TypeChecking>::is_u64(&arg_type) {
//                         return Err(SemanticError::IncorrectArguments);
//                     }
//                     Ok(())
//                 }
//                 PlatformApi::WRITE => {
//                     if args.len() != 2 {
//                         return Err(SemanticError::IncorrectArguments);
//                     }
//                     let cell = &args[0];
//                     let _ = cell.resolve(
//                         block,
//                         &Some(Either::Static(
//                             StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
//                         )),
//                         &(),
//                     )?;
//                     let cell_type = cell.type_of(&block.borrow())?;
//                     if !<EType as TypeChecking>::is_u64(&cell_type) {
//                         return Err(SemanticError::IncorrectArguments);
//                     }
//                     let data = &args[1];
//                     let _ = data.resolve(
//                         block,
//                         &Some(Either::Static(
//                             StaticType::Primitive(PrimitiveType::Char).into(),
//                         )),
//                         &(),
//                     )?;
//                     let data_type = data.type_of(&block.borrow())?;
//                     if !<EType as TypeChecking>::is_char(&data_type) {
//                         return Err(SemanticError::IncorrectArguments);
//                     }
//                     Ok(())
//                 }
//                 PlatformApi::APPEND => {
//                     if args.len() != 2 {
//                         return Err(SemanticError::IncorrectArguments);
//                     }
//                     let vector = &args[0];
//                     let _ = vector.resolve(block, &None, &())?;
//                     let vector_type = vector.type_of(&block.borrow())?;
//                     let element = &args[1];
//                     let _ = element.resolve(block, &None, &())?;
//                     let element_type = element.type_of(&block.borrow())?;
//                     if !<EType as TypeChecking>::is_vec(&vector_type) {
//                         return Err(SemanticError::IncorrectArguments);
//                     }
//                     let item_type = <EType as GetSubTypes>::get_item(&vector_type).unwrap();
//                     let _ = item_type.compatible_with(&element_type, &block.borrow())?;
//                     Ok(())
//                 }
//                 PlatformApi::INSERT => {
//                     if args.len() != 3 {
//                         return Err(SemanticError::IncorrectArguments);
//                     }
//                     let map = &args[0];
//                     let _ = map.resolve(block, &None, &())?;
//                     let map_type = map.type_of(&block.borrow())?;

//                     let expr_key = &args[1];
//                     let _ = expr_key.resolve(block, &None, &())?;
//                     let expr_key_type = expr_key.type_of(&block.borrow())?;

//                     let expr_value = &args[2];
//                     let _ = expr_value.resolve(block, &None, &())?;
//                     let expr_value_type = expr_value.type_of(&block.borrow())?;

//                     if !<EType as TypeChecking>::is_map(&map_type) {
//                         return Err(SemanticError::IncorrectArguments);
//                     }
//                     let value_type = <EType as GetSubTypes>::get_item(&map_type).unwrap();

//                     let key_type = <EType as GetSubTypes>::get_key(&map_type).unwrap();

//                     let _ = key_type.compatible_with(&expr_key_type, &block.borrow())?;
//                     let _ = value_type.compatible_with(&expr_value_type, &block.borrow())?;
//                     Ok(())
//                 }
//                 PlatformApi::DELETE => {
//                     if args.len() != 2 {
//                         return Err(SemanticError::IncorrectArguments);
//                     }
//                     let iterator = &args[0];
//                     let _ = iterator.resolve(block, &None, &())?;
//                     let iterator_type = iterator.type_of(&block.borrow())?;

//                     if <EType as TypeChecking>::is_map(&iterator_type) {
//                         let key_type = <EType as GetSubTypes>::get_key(&iterator_type).unwrap();

//                         let expr_key = &args[1];
//                         let _ = expr_key.resolve(block, &Some(key_type), &())?;
//                         let expr_key_type = expr_key.type_of(&block.borrow())?;
//                         // let _ = key_type.compatible_with(&expr_key_type, &block.borrow())?;
//                     } else if <EType as TypeChecking>::is_vec(&iterator_type) {
//                         let expr_key = &args[1];
//                         let _ = expr_key.resolve(
//                             block,
//                             &Some(Either::Static(
//                                 StaticType::Primitive(PrimitiveType::Number(NumberType::U64))
//                                     .into(),
//                             )),
//                             &(),
//                         )?;
//                         let expr_key_type = expr_key.type_of(&block.borrow())?;

//                         if !<EType as TypeChecking>::is_u64(&expr_key_type) {
//                             return Err(SemanticError::IncorrectArguments);
//                         }
//                     } else {
//                         return Err(SemanticError::IncorrectArguments);
//                     }

//                     Ok(())
//                 }
//                 PlatformApi::FREE => {
//                     if args.len() != 1 {
//                         return Err(SemanticError::IncorrectArguments);
//                     }
//                     let arg = args.first().unwrap();
//                     let _ = arg.resolve(block, &None, &())?;
//                     let arg_type = arg.type_of(&block.borrow())?;
//                     if !<EType as TypeChecking>::is_addr(&arg_type) {
//                         return Err(SemanticError::IncorrectArguments);
//                     }
//                     Ok(())
//                 }
//                 PlatformApi::SPAWN | PlatformApi::CLOSE => {
//                     if args.len() != 0 {
//                         return Err(SemanticError::IncorrectArguments);
//                     }
//                     Ok(())
//                 }
//                 PlatformApi::PRINT => Ok(()),
//             }
//         }

//         pub fn resolve(
//             &self,
//             args: &Vec<Expression>,
//             block: &MutRc<Scope>,
//         ) -> Result<(), SemanticError> {
//             let _ = self.accept(args, block)?;
//             Ok(())
//         }

//         pub fn returns(self) -> EType {
//             e_static!(<StaticType as BuildStaticType>::build_unit())
//         }
//     }
// }

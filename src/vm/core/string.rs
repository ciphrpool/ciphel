use crate::semantic::scope::static_types::{PrimitiveType, StaticType, StrSliceType, StringType};
use crate::semantic::TypeOf;
use crate::vm::allocator::{align, MemoryAddress};
use crate::vm::asm::operation::{GetNumFrom, OpPrimitive, PopNum};
use crate::vm::asm::Asm;
use crate::vm::core::lexem;
use crate::vm::core::CoreAsm;
use crate::vm::program::Program;
use crate::vm::runtime::RuntimeError;
use crate::vm::scheduler::Executable;
use crate::vm::{CodeGenerationError, GenerateCode};
use crate::{
    ast::expressions::Expression,
    semantic::{EType, Metadata, Resolve, ResolveCore, SemanticError},
    vm::{
        allocator::{heap::Heap, stack::Stack},
        stdio::StdIO,
    },
};

use super::PathFinder;

#[derive(Debug, Clone, PartialEq)]
pub enum StringFn {
    String {},
    Append {},
    CharAt { for_string: bool },
    ToConstStr {},
}

impl PathFinder for StringFn {
    fn find(path: &[String], name: &str) -> Option<Self>
    where
        Self: Sized,
    {
        if (path.len() == 1 && path[0] == lexem::STRING) || path.len() == 0 {
            return match name {
                lexem::STRING => Some(StringFn::String {}),
                lexem::APPEND => Some(StringFn::Append {}),
                lexem::CHARAT => Some(StringFn::CharAt { for_string: true }),
                lexem::TO_CONST_STR => Some(StringFn::ToConstStr {}),
                _ => None,
            };
        }
        None
    }
}
impl ResolveCore for StringFn {
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: Option<&EType>,
        parameters: &mut Vec<Expression>,
    ) -> Result<EType, SemanticError> {
        fn string_param<E: crate::vm::external::Engine>(
            param: &mut Expression,
            scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
            scope_id: Option<u128>,
        ) -> Result<StringType, SemanticError> {
            let _ = param.resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
            let EType::Static(StaticType::String(string_type)) =
                param.type_of(scope_manager, scope_id)?
            else {
                return Err(SemanticError::IncorrectArguments);
            };
            Ok(string_type)
        }

        fn string_or_slice_param<E: crate::vm::external::Engine>(
            param: &mut Expression,
            scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
            scope_id: Option<u128>,
        ) -> Result<bool, SemanticError> {
            let _ = param.resolve::<E>(scope_manager, scope_id, &None, &mut None)?;

            match param.type_of(scope_manager, scope_id)? {
                EType::Static(StaticType::String(_)) => Ok(true),
                EType::Static(StaticType::StrSlice(_)) => Ok(false),
                _ => Err(SemanticError::IncorrectArguments),
            }
        }

        match self {
            StringFn::String {} => {
                if parameters.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let str_slice = &mut parameters[0];
                let _ = str_slice.resolve::<E>(
                    scope_manager,
                    scope_id,
                    &Some(EType::Static(StaticType::StrSlice(StrSliceType()))),
                    &mut None,
                )?;

                let EType::Static(StaticType::StrSlice(StrSliceType())) =
                    str_slice.type_of(scope_manager, scope_id)?
                else {
                    return Err(SemanticError::IncorrectArguments);
                };

                Ok(EType::Static(StaticType::String(StringType())))
            }
            StringFn::Append {} => {
                if parameters.len() != 2 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let (first_part, second_part) = parameters.split_at_mut(1);
                let string = &mut first_part[0];
                let const_str = &mut second_part[1];
                let _ = string_param::<E>(string, scope_manager, scope_id)?;

                let _ = const_str.resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
                let EType::Static(StaticType::StrSlice(_)) =
                    const_str.type_of(scope_manager, scope_id)?
                else {
                    return Err(SemanticError::IncorrectArguments);
                };

                Ok(EType::Static(StaticType::String(StringType())))
            }
            StringFn::CharAt { for_string } => {
                if parameters.len() != 2 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let (first_part, second_part) = parameters.split_at_mut(1);
                let string = &mut first_part[0];
                let index = &mut second_part[1];
                *for_string = string_or_slice_param::<E>(string, scope_manager, scope_id)?;

                let _ = index.resolve::<E>(
                    scope_manager,
                    scope_id,
                    &Some(crate::p_num!(U64)),
                    &mut None,
                )?;
                let crate::p_num!(U64) = index.type_of(scope_manager, scope_id)? else {
                    return Err(SemanticError::IncorrectArguments);
                };

                Ok(EType::Static(StaticType::Primitive(PrimitiveType::Char)))
            }
            StringFn::ToConstStr {} => {
                if parameters.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let string = &mut parameters[0];
                let _ = string_param::<E>(string, scope_manager, scope_id)?;

                Ok(EType::Static(StaticType::StrSlice(StrSliceType())))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StringAsm {
    String {},
    Append {},
    CharAtStrslice,
    CharAtString,
    ToConstStr,
}

impl<E: crate::vm::external::Engine> crate::vm::AsmName<E> for StringAsm {
    fn name(&self, stdio: &mut StdIO, program: &crate::vm::program::Program<E>, engine: &mut E) {
        match self {
            StringAsm::String {} => stdio.push_asm_lib(engine, "string"),
            StringAsm::Append {} => stdio.push_asm_lib(engine, "append"),
            StringAsm::CharAtString => stdio.push_asm_lib(engine, "char_at"),
            StringAsm::CharAtStrslice => stdio.push_asm_lib(engine, "char_at"),
            StringAsm::ToConstStr => stdio.push_asm_lib(engine, "to_const_str"),
        }
    }
}

impl crate::vm::AsmWeight for StringAsm {
    fn weight(&self) -> crate::vm::Weight {
        match self {
            StringAsm::String {} => crate::vm::Weight::MEDIUM,
            StringAsm::Append {} => crate::vm::Weight::MEDIUM,
            StringAsm::CharAtString => crate::vm::Weight::MEDIUM,
            StringAsm::CharAtStrslice => crate::vm::Weight::MEDIUM,
            StringAsm::ToConstStr => crate::vm::Weight::MEDIUM,
        }
    }
}

impl GenerateCode for StringFn {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        match *self {
            StringFn::String {} => {
                instructions.push(Asm::Core(CoreAsm::String(StringAsm::String {})))
            }
            StringFn::Append {} => {
                instructions.push(Asm::Core(CoreAsm::String(StringAsm::Append {})))
            }
            StringFn::CharAt { for_string } => {
                if for_string {
                    instructions.push(Asm::Core(CoreAsm::String(StringAsm::CharAtString {})))
                } else {
                    instructions.push(Asm::Core(CoreAsm::String(StringAsm::CharAtStrslice {})))
                }
            }
            StringFn::ToConstStr {} => {
                instructions.push(Asm::Core(CoreAsm::String(StringAsm::ToConstStr {})))
            }
        }
        Ok(())
    }
}
/*
    STRING LAYOUT
    CAPACITY u64
    LEN u64
    ...DATA
*/
pub const STRING_HEADER: usize = 16;

impl<E: crate::vm::external::Engine> Executable<E> for StringAsm {
    fn execute<P: crate::vm::scheduler::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler::Scheduler<P>,
        signal_handler: &mut crate::vm::runtime::SignalHandler<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::TID>,
    ) -> Result<(), RuntimeError> {
        match self {
            StringAsm::String {} => {
                let slice: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                let string = OpPrimitive::get_string_from(slice, stack, heap)?;
                let len = string.len();
                let cap = len * 2;
                let address = heap.alloc(cap)?;

                /* Write capacity */
                let _ = heap.write(address, &(cap as u64).to_le_bytes())?;
                /* Write len */
                let _ = heap.write(address.add(8), &(len as u64).to_le_bytes())?;

                /* Write slice */
                let _ = heap.write(address.add(STRING_HEADER), string.as_bytes())?;

                /* Push vec address */
                let address: u64 = address.into(stack);
                stack.push_with(&address.to_le_bytes())?;
            }
            StringAsm::Append {} => {
                let slice = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                let mut address = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;

                let string = OpPrimitive::get_string_from(slice, stack, heap)?;

                let mut cap = OpPrimitive::get_num_from::<u64>(address, stack, heap)? as usize;
                let mut len =
                    OpPrimitive::get_num_from::<u64>(address.add(8), stack, heap)? as usize;

                let previous_len = len;

                if len + string.len() >= cap {
                    // Reallocate
                    let size = align(((len + string.len()) * 2) + STRING_HEADER);
                    len += string.len();
                    cap = (len + string.len()) * 2;
                    address = heap.realloc(address, size)?;
                }

                /* Write capacity */
                let _ = heap.write(address, &(cap as u64).to_le_bytes())?;
                /* Write len */
                let _ = heap.write(address.add(8), &(len as u64).to_le_bytes())?;

                /* Write new data */
                let _ = heap.write(address.add(STRING_HEADER + previous_len), string.as_bytes())?;

                /* Push vec address */
                let address: u64 = address.into(stack);
                stack.push_with(&address.to_le_bytes())?;
            }
            StringAsm::CharAtString => {
                let index = OpPrimitive::pop_num::<u64>(stack)?;
                let address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                let string = OpPrimitive::get_string_from(address.add(8), stack, heap)?;
                let charac = string
                    .chars()
                    .nth(index as usize)
                    .ok_or(RuntimeError::IndexOutOfBound)?;

                let mut buffer = [0u8; 4];
                let _ = charac.encode_utf8(&mut buffer);
                stack.push_with(&buffer)?;
            }

            StringAsm::CharAtStrslice => {
                let index = OpPrimitive::pop_num::<u64>(stack)?;
                let address = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                let string = OpPrimitive::get_string_from(address, stack, heap)?;

                let charac = string
                    .chars()
                    .nth(index as usize)
                    .ok_or(RuntimeError::IndexOutOfBound)?;

                let mut buffer = [0u8; 4];
                let _ = charac.encode_utf8(&mut buffer);
                stack.push_with(&buffer)?;
            }
            StringAsm::ToConstStr => {
                let address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;

                let address: u64 = address.add(8).into(stack);
                stack.push_with(&address.to_le_bytes())?;
            }
        }
        scheduler.next();
        Ok(())
    }
}

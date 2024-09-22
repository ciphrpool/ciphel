use crate::semantic::scope::static_types::{PrimitiveType, StaticType, StrSliceType, StringType};
use crate::semantic::TypeOf;
use crate::vm::allocator::{align, MemoryAddress};
use crate::vm::casm::operation::{GetNumFrom, OpPrimitive, PopNum};
use crate::vm::casm::Casm;
use crate::vm::core::lexem;
use crate::vm::core::CoreCasm;
use crate::{
    ast::expressions::Expression,
    semantic::{EType, Metadata, Resolve, ResolvePlatform, SemanticError},
    vm::{
        allocator::{heap::Heap, stack::Stack},
        casm::CasmProgram,
        stdio::StdIO,
        vm::{CasmMetadata, CodeGenerationError, Executable, GenerateCode, RuntimeError},
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
impl ResolvePlatform for StringFn {
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: Option<&EType>,
        parameters: &mut Vec<Expression>,
    ) -> Result<EType, SemanticError> {
        fn string_param<G: crate::GameEngineStaticFn>(
            param: &mut Expression,
            scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
            scope_id: Option<u128>,
        ) -> Result<StringType, SemanticError> {
            let _ = param.resolve::<G>(scope_manager, scope_id, &None, &mut None)?;
            let EType::Static(StaticType::String(string_type)) =
                param.type_of(scope_manager, scope_id)?
            else {
                return Err(SemanticError::IncorrectArguments);
            };
            Ok(string_type)
        }

        fn string_or_slice_param<G: crate::GameEngineStaticFn>(
            param: &mut Expression,
            scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
            scope_id: Option<u128>,
        ) -> Result<bool, SemanticError> {
            let _ = param.resolve::<G>(scope_manager, scope_id, &None, &mut None)?;

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
                let _ = str_slice.resolve::<G>(
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
                let _ = string_param::<G>(string, scope_manager, scope_id)?;

                let _ = const_str.resolve::<G>(scope_manager, scope_id, &None, &mut None)?;
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
                *for_string = string_or_slice_param::<G>(string, scope_manager, scope_id)?;

                let _ = index.resolve::<G>(
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
                let _ = string_param::<G>(string, scope_manager, scope_id)?;

                Ok(EType::Static(StaticType::StrSlice(StrSliceType())))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StringCasm {
    String {},
    Append {},
    CharAtStrslice,
    CharAtString,
    ToConstStr,
}

impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for StringCasm {
    fn name(&self, stdio: &mut StdIO, program: &mut CasmProgram, engine: &mut G) {
        match self {
            StringCasm::String {} => stdio.push_casm_lib(engine, "string"),
            StringCasm::Append {} => stdio.push_casm_lib(engine, "append"),
            StringCasm::CharAtString => stdio.push_casm_lib(engine, "char_at"),
            StringCasm::CharAtStrslice => stdio.push_casm_lib(engine, "char_at"),
            StringCasm::ToConstStr => stdio.push_casm_lib(engine, "to_const_str"),
        }
    }

    fn weight(&self) -> crate::vm::vm::CasmWeight {
        match self {
            StringCasm::String {} => crate::vm::vm::CasmWeight::MEDIUM,
            StringCasm::Append {} => crate::vm::vm::CasmWeight::MEDIUM,
            StringCasm::CharAtString => crate::vm::vm::CasmWeight::MEDIUM,
            StringCasm::CharAtStrslice => crate::vm::vm::CasmWeight::MEDIUM,
            StringCasm::ToConstStr => crate::vm::vm::CasmWeight::MEDIUM,
        }
    }
}

impl GenerateCode for StringFn {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        match *self {
            StringFn::String {} => {
                instructions.push(Casm::Core(CoreCasm::String(StringCasm::String {})))
            }
            StringFn::Append {} => {
                instructions.push(Casm::Core(CoreCasm::String(StringCasm::Append {})))
            }
            StringFn::CharAt { for_string } => {
                if for_string {
                    instructions.push(Casm::Core(CoreCasm::String(StringCasm::CharAtString {})))
                } else {
                    instructions.push(Casm::Core(CoreCasm::String(StringCasm::CharAtStrslice {})))
                }
            }
            StringFn::ToConstStr {} => {
                instructions.push(Casm::Core(CoreCasm::String(StringCasm::ToConstStr {})))
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

impl<G: crate::GameEngineStaticFn> Executable<G> for StringCasm {
    fn execute(
        &self,
        program: &mut CasmProgram,
        stack: &mut Stack,
        heap: &mut Heap,
        stdio: &mut StdIO,
        engine: &mut G,
        tid: usize,
    ) -> Result<(), RuntimeError> {
        match self {
            StringCasm::String {} => {
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
            StringCasm::Append {} => {
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
            StringCasm::CharAtString => {
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

            StringCasm::CharAtStrslice => {
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
            StringCasm::ToConstStr => {
                let address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;

                let address: u64 = address.add(8).into(stack);
                stack.push_with(&address.to_le_bytes())?;
            }
        }
        program.incr();
        Ok(())
    }
}

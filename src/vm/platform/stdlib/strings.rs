use crate::ast::utils::strings::ID;
use crate::e_static;
use crate::semantic::scope::scope::ScopeManager;
use crate::semantic::scope::static_types::{NumberType, PrimitiveType, StaticType, StringType};
use crate::semantic::{EType, ResolvePlatform, TypeOf};
use crate::vm::allocator::align;
use crate::vm::allocator::heap::Heap;
use crate::vm::allocator::stack::Stack;
use crate::vm::casm::operation::{OpPrimitive, PopNum};
use crate::vm::casm::Casm;
use crate::vm::platform::utils::lexem;
use crate::vm::platform::LibCasm;

use crate::vm::stdio::StdIO;
use crate::vm::vm::{CasmMetadata, Executable, RuntimeError};
use crate::{
    ast::expressions::Expression,
    semantic::{Resolve, SemanticError},
    vm::{
        casm::CasmProgram,
        vm::{CodeGenerationError, GenerateCode},
    },
};
#[derive(Debug, Clone, PartialEq)]
pub enum StringsFn {
    ToStr(StringsCasm),
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum StringsCasm {
    ToStr(ToStrCasm),
    Join(JoinCasm),
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum ToStrCasm {
    ToStrU128,
    ToStrU64,
    ToStrU32,
    ToStrU16,
    ToStrU8,
    ToStrI128,
    ToStrI64,
    ToStrI32,
    ToStrI16,
    ToStrI8,
    ToStrF64,
    ToStrChar,
    ToStrBool,
    ToStrStrSlice,
    ToStrString,
}

impl<G: crate::GameEngineStaticFn> CasmMetadata<G> for StringsCasm {
    fn name(&self, stdio: &mut StdIO, program: &mut CasmProgram, engine: &mut G) {
        match self {
            StringsCasm::ToStr(_) => stdio.push_casm_lib(engine, "to_str"),
            StringsCasm::Join(_) => stdio.push_casm_lib(engine, "str_join"),
        }
    }
    fn weight(&self) -> crate::vm::vm::CasmWeight {
        crate::vm::vm::CasmWeight::HIGH
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum JoinCasm {
    NoSepFromSlice(Option<usize>),
}

impl StringsFn {
    pub fn from(suffixe: &Option<ID>, id: &ID) -> Option<Self> {
        match suffixe {
            Some(suffixe) => {
                if *suffixe != lexem::STD {
                    return None;
                }
            }
            None => {}
        }
        match id.as_str() {
            lexem::TOSTR => Some(StringsFn::ToStr(StringsCasm::ToStr(ToStrCasm::ToStrI64))),
            _ => None,
        }
    }
}

impl ResolvePlatform for StringsFn {
    fn resolve<G: crate::GameEngineStaticFn>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: Option<&EType>,
        parameters: &mut Vec<Expression>,
    ) -> Result<EType, SemanticError> {
        match self {
            StringsFn::ToStr(casm) => {
                if parameters.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }

                let src = &mut parameters[0];

                let _ = src.resolve::<G>(scope_manager, scope_id, &None, &mut None)?;
                let src_type = src.type_of(&scope_manager, scope_id)?;

                match &src_type {
                    EType::Static(value) => match value {
                        StaticType::Primitive(p) => match p {
                            PrimitiveType::Number(n) => match n {
                                NumberType::U8 => *casm = StringsCasm::ToStr(ToStrCasm::ToStrU8),
                                NumberType::U16 => *casm = StringsCasm::ToStr(ToStrCasm::ToStrU16),
                                NumberType::U32 => *casm = StringsCasm::ToStr(ToStrCasm::ToStrU32),
                                NumberType::U64 => *casm = StringsCasm::ToStr(ToStrCasm::ToStrU64),
                                NumberType::U128 => {
                                    *casm = StringsCasm::ToStr(ToStrCasm::ToStrU128)
                                }
                                NumberType::I8 => *casm = StringsCasm::ToStr(ToStrCasm::ToStrI8),
                                NumberType::I16 => *casm = StringsCasm::ToStr(ToStrCasm::ToStrI16),
                                NumberType::I32 => *casm = StringsCasm::ToStr(ToStrCasm::ToStrI32),
                                NumberType::I64 => *casm = StringsCasm::ToStr(ToStrCasm::ToStrI64),
                                NumberType::I128 => {
                                    *casm = StringsCasm::ToStr(ToStrCasm::ToStrI128)
                                }
                                NumberType::F64 => *casm = StringsCasm::ToStr(ToStrCasm::ToStrF64),
                            },
                            PrimitiveType::Char => *casm = StringsCasm::ToStr(ToStrCasm::ToStrChar),
                            PrimitiveType::Bool => *casm = StringsCasm::ToStr(ToStrCasm::ToStrBool),
                        },
                        StaticType::String(_) => *casm = StringsCasm::ToStr(ToStrCasm::ToStrString),
                        StaticType::StrSlice(_) => {
                            *casm = StringsCasm::ToStr(ToStrCasm::ToStrStrSlice)
                        }
                        _ => return Err(SemanticError::IncorrectArguments),
                    },
                    _ => return Err(SemanticError::IncorrectArguments),
                }

                Ok(e_static!(StaticType::String(StringType())))
            }
        }
    }
}

impl GenerateCode for StringsFn {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        match self {
            StringsFn::ToStr(to_str_casm) => instructions.push(Casm::Platform(LibCasm::Std(
                super::StdCasm::Strings(*to_str_casm),
            ))),
        }
        Ok(())
    }
}

pub fn push_string(src: String, stack: &mut Stack, heap: &mut Heap) -> Result<(), RuntimeError> {
    let len = src.len();
    let cap = align(len as usize) as u64;
    let alloc_size = cap + 16;

    let len_bytes = len.to_le_bytes().as_slice().to_vec();
    let cap_bytes = cap.to_le_bytes().as_slice().to_vec();
    let address = heap.alloc(alloc_size as usize)?;

    let data = src.into_bytes();

    /* Write len */
    let _ = heap.write(address, &len_bytes)?;
    /* Write capacity */
    let _ = heap.write(address.add(8), &cap_bytes)?;
    /* Write data */
    let _ = heap.write(address.add(16), &data)?;

    let address: u64 = address.into(stack);
    let _ = stack.push_with(&address.to_le_bytes())?;
    Ok(())
}

impl<G: crate::GameEngineStaticFn> Executable<G> for StringsCasm {
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
            StringsCasm::ToStr(value) => {
                let res = match value {
                    ToStrCasm::ToStrU128 => {
                        let n = OpPrimitive::pop_num::<u128>(stack)?;
                        format!("{}", n)
                    }
                    ToStrCasm::ToStrU64 => {
                        let n = OpPrimitive::pop_num::<u64>(stack)?;
                        format!("{}", n)
                    }
                    ToStrCasm::ToStrU32 => {
                        let n = OpPrimitive::pop_num::<u32>(stack)?;
                        format!("{}", n)
                    }
                    ToStrCasm::ToStrU16 => {
                        let n = OpPrimitive::pop_num::<u16>(stack)?;
                        format!("{}", n)
                    }
                    ToStrCasm::ToStrU8 => {
                        let n = OpPrimitive::pop_num::<u8>(stack)?;
                        format!("{}", n)
                    }
                    ToStrCasm::ToStrI128 => {
                        let n = OpPrimitive::pop_num::<i128>(stack)?;
                        format!("{}", n)
                    }
                    ToStrCasm::ToStrI64 => {
                        let n = OpPrimitive::pop_num::<i64>(stack)?;
                        format!("{}", n)
                    }
                    ToStrCasm::ToStrI32 => {
                        let n = OpPrimitive::pop_num::<i32>(stack)?;
                        format!("{}", n)
                    }
                    ToStrCasm::ToStrI16 => {
                        let n = OpPrimitive::pop_num::<i16>(stack)?;
                        format!("{}", n)
                    }
                    ToStrCasm::ToStrI8 => {
                        let n = OpPrimitive::pop_num::<i8>(stack)?;
                        format!("{}", n)
                    }
                    ToStrCasm::ToStrF64 => {
                        let n = OpPrimitive::pop_float(stack)?;
                        format!("{}", n)
                    }
                    ToStrCasm::ToStrChar => {
                        let n = OpPrimitive::pop_char(stack)?;
                        format!("{}", n)
                    }
                    ToStrCasm::ToStrBool => {
                        let n = OpPrimitive::pop_bool(stack)?;
                        format!("{}", n)
                    }
                    ToStrCasm::ToStrStrSlice => {
                        let address = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                        let n = OpPrimitive::get_string_from(address, stack, heap)?;
                        format!("{}", n)
                    }
                    ToStrCasm::ToStrString => {
                        let address = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                        let n = OpPrimitive::get_string_from(address, stack, heap)?;
                        format!("{}", n)
                    }
                };
                let len = res.len();
                let cap = align(len as usize) as u64;
                let alloc_size = cap + 16;

                let len_bytes = len.to_le_bytes().as_slice().to_vec();
                let cap_bytes = cap.to_le_bytes().as_slice().to_vec();
                let address = heap.alloc(alloc_size as usize)?;

                let data = res.into_bytes();

                /* Write len */
                let _ = heap.write(address, &len_bytes)?;
                /* Write capacity */
                let _ = heap.write(address.add(8), &cap_bytes)?;
                /* Write data */
                let _ = heap.write(address.add(16), &data)?;

                let address: u64 = address.into(stack);
                let _ = stack.push_with(&address.to_le_bytes())?;
            }
            StringsCasm::Join(value) => match value {
                JoinCasm::NoSepFromSlice(opt_len) => {
                    let len = match opt_len {
                        Some(len) => *len,
                        None => {
                            let n = OpPrimitive::pop_num::<u64>(stack)?;
                            n as usize
                        }
                    };
                    let mut slice_data = Vec::new();
                    for _ in 0..len {
                        let string_heap_address = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;

                        let string_len_bytes = heap.read(string_heap_address, 8)?;
                        let string_len_bytes =
                            TryInto::<&[u8; 8]>::try_into(string_len_bytes.as_slice())
                                .map_err(|_| RuntimeError::Deserialization)?;
                        let string_len = u64::from_le_bytes(*string_len_bytes);
                        let string_data =
                            heap.read(string_heap_address.add(16), string_len as usize)?;

                        let _ = heap.free(string_heap_address)?;

                        slice_data.push(string_data);
                    }
                    let data = slice_data.into_iter().rev().flatten().collect::<Vec<u8>>();

                    let len = data.len();
                    let cap = align(len as usize) as u64;
                    let alloc_size = cap + 16;

                    let len_bytes = len.to_le_bytes().as_slice().to_vec();
                    let cap_bytes = cap.to_le_bytes().as_slice().to_vec();
                    let address = heap.alloc(alloc_size as usize)?;

                    /* Write len */
                    let _ = heap.write(address, &len_bytes)?;
                    /* Write capacity */
                    let _ = heap.write(address.add(8), &cap_bytes)?;
                    /* Write data */
                    let _ = heap.write(address.add(16), &data)?;

                    let address: u64 = address.into(stack);
                    let _ = stack.push_with(&address.to_le_bytes())?;
                }
            },
        }

        program.incr();
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::ast::statements::Statement;
    use crate::ast::TryParse;
    use crate::clear_stack;
    use crate::vm::vm::Runtime;

    // #[test]
    // fn valid_format_i64() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = f"Hello {10}";
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let result = compile_statement_for_string!(statement);
    //     assert_eq!(result, "Hello 10");
    // }

    // #[test]
    // fn valid_format_u64() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = f"Hello {10u64}";
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let result = compile_statement_for_string!(statement);
    //     assert_eq!(result, "Hello 10");
    // }

    // #[test]
    // fn valid_format_float() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = f"Hello {20.5}";
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let result = compile_statement_for_string!(statement);
    //     assert_eq!(result, "Hello 20.5");
    // }

    // #[test]
    // fn valid_format_bool() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = f"Hello {true}";
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let result = compile_statement_for_string!(statement);
    //     assert_eq!(result, "Hello true");

    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = f"Hello {false}";
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let result = compile_statement_for_string!(statement);
    //     assert_eq!(result, "Hello false");

    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = f"Hello {false} {true}";
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let result = compile_statement_for_string!(statement);
    //     assert_eq!(result, "Hello false true");
    // }

    // #[test]
    // fn valid_format_char() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = f"Hello {'a'}";
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let result = compile_statement_for_string!(statement);
    //     assert_eq!(result, "Hello a");
    // }
}

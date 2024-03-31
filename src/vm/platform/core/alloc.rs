use std::cell::Ref;

use nom_supreme::parser_ext::Value;
use num_traits::ToBytes;

use crate::{
    ast::{expressions::Expression, types::PrimitiveType},
    semantic::{
        scope::{
            static_types::{self, NumberType, StaticType, StringType},
            ScopeApi,
        },
        AccessLevel, EType, Either, MutRc, Resolve, SemanticError, SizeOf, TypeOf,
    },
    vm::{
        allocator::{align, stack::Offset, MemoryAddress},
        casm::{
            alloc::{Access, Alloc},
            memcopy::MemCopy,
            operation::{Addition, OpPrimitive, Operation, OperationKind, Substraction},
            serialize::Serialized,
            Casm, CasmProgram,
        },
        platform::{utils::lexem, GenerateCodePlatform},
        vm::{CodeGenerationError, GenerateCode},
    },
};

use super::thread::ThreadFn;

#[derive(Debug, Clone, PartialEq)]
pub enum AllocFn {
    Append,
    Insert,
    Delete,
    Free,
    Vec,
    Map,
    Chan,
    String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AllocCasm {
    Append,
    Insert,
    Delete,
    Free,
    Vec,
    Map,
    Chan,
    String,
}

impl AllocFn {
    pub fn from(id: &String) -> Option<Self> {
        match id.as_str() {
            lexem::APPEND => Some(AllocFn::Append),
            lexem::INSERT => Some(AllocFn::Insert),
            lexem::DELETE => Some(AllocFn::Delete),
            lexem::FREE => Some(AllocFn::Free),
            lexem::VEC => Some(AllocFn::Vec),
            lexem::MAP => Some(AllocFn::Map),
            lexem::CHAN => Some(AllocFn::Chan),
            lexem::STRING => Some(AllocFn::String),
            _ => None,
        }
    }
}

impl<Scope: ScopeApi> Resolve<Scope> for AllocFn {
    type Output = ();
    type Context = Option<EType>;
    type Extra = Vec<Expression<Scope>>;
    fn resolve(
        &self,
        scope: &MutRc<Scope>,
        context: &Self::Context,
        extra: &Self::Extra,
    ) -> Result<Self::Output, SemanticError> {
        match self {
            AllocFn::Append => todo!(),
            AllocFn::Insert => todo!(),
            AllocFn::Delete => todo!(),
            AllocFn::Free => todo!(),
            AllocFn::Vec => todo!(),
            AllocFn::Map => todo!(),
            AllocFn::Chan => todo!(),
            AllocFn::String => {
                if extra.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let param = extra.first().unwrap();
                let _ = param.resolve(scope, &None, &())?;
                let param_type = param.type_of(&scope.borrow())?;
                match param_type {
                    Either::Static(value) => match value.as_ref() {
                        StaticType::StrSlice(_) => {}
                        _ => {
                            return Err(SemanticError::IncorrectArguments);
                        }
                    },
                    Either::User(_) => {
                        return Err(SemanticError::IncorrectArguments);
                    }
                }
                Ok(())
            }
        }
    }
}
impl<Scope: ScopeApi> TypeOf<Scope> for AllocFn {
    fn type_of(&self, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: ScopeApi,
        Self: Sized + Resolve<Scope>,
    {
        match self {
            AllocFn::Append => todo!(),
            AllocFn::Insert => todo!(),
            AllocFn::Delete => todo!(),
            AllocFn::Free => todo!(),
            AllocFn::Vec => todo!(),
            AllocFn::Map => todo!(),
            AllocFn::Chan => todo!(),
            AllocFn::String => Ok(Either::Static(StaticType::String(StringType()).into())),
        }
    }
}

impl<Scope: ScopeApi> GenerateCodePlatform<Scope> for AllocFn {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
        params_size: usize,
    ) -> Result<(), CodeGenerationError> {
        match self {
            AllocFn::Append => todo!(),
            AllocFn::Insert => todo!(),
            AllocFn::Delete => todo!(),
            AllocFn::Free => todo!(),
            AllocFn::Vec => gencode_vec(scope, instructions, params_size),
            AllocFn::Map => todo!(),
            AllocFn::Chan => todo!(),
            AllocFn::String => gencode_string(scope, instructions, params_size),
        }
    }
}

fn gencode_vec<Scope: ScopeApi>(
    scope: &MutRc<Scope>,
    instructions: &CasmProgram,
    params_size: usize,
) -> Result<(), CodeGenerationError> {
    todo!()
}

fn gencode_string<Scope: ScopeApi>(
    scope: &MutRc<Scope>,
    instructions: &CasmProgram,
    params_size: usize,
) -> Result<(), CodeGenerationError> {
    let len_bytes = (params_size as u64).to_le_bytes().as_slice().to_vec();
    let cap_bytes = (align(params_size) as u64)
        .to_le_bytes()
        .as_slice()
        .to_vec();

    // Push Length on stack
    instructions.push(Casm::Serialize(Serialized { data: len_bytes }));
    // Push Capacity on stack
    instructions.push(Casm::Serialize(Serialized { data: cap_bytes }));

    // Copy data on stack to heap at address
    // Alloc and push heap address on stack
    instructions.push(Casm::Alloc(Alloc::Heap {
        size: align(params_size + 16),
    }));
    // Take the address on the top of the stack
    // and copy the data on the stack in the heap at given address and given offset
    // ( removing the data from the stack ) and put back the address on top of the stack
    instructions.push(Casm::MemCopy(MemCopy::TakeToHeap {
        //offset: vec_stack_address + 8,
        size: 16,
    }));
    instructions.push(Casm::Serialize(Serialized {
        data: (16u64).to_le_bytes().to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Addition(Addition {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));

    instructions.push(Casm::MemCopy(MemCopy::TakeToHeap {
        //offset: vec_stack_address + 8,
        size: params_size,
    }));
    instructions.push(Casm::Serialize(Serialized {
        data: (16u64).to_le_bytes().to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Substraction(Substraction {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    Ok(())
}

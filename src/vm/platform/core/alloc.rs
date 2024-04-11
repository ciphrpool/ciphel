use std::{
    cell::{Cell, Ref},
    vec,
};

use nom_supreme::parser_ext::Value;
use num_traits::ToBytes;

use crate::{
    ast::expressions::Expression,
    semantic::{
        scope::{
            static_types::{self, NumberType, PrimitiveType, StaticType, StringType, VecType},
            type_traits::{GetSubTypes, TypeChecking},
            ScopeApi,
        },
        AccessLevel, EType, Either, Info, Metadata, MutRc, Resolve, SemanticError, SizeOf, TypeOf,
    },
    vm::{
        allocator::{align, stack::Offset, MemoryAddress},
        casm::{
            alloc::{Access, Alloc, Realloc},
            branch::{BranchIf, Goto, Label},
            memcopy::MemCopy,
            operation::{
                Addition, Equal, Mult, OpPrimitive, Operation, OperationKind, Substraction,
            },
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
    Append {
        item_size: Cell<usize>,
    },
    Insert,
    Delete,
    Free,
    // Alloc,
    Vec {
        with_capacity: Cell<bool>,
        item_size: Cell<usize>,
        metadata: Metadata,
    },
    Map,
    Chan,
    String,
}

impl AllocFn {
    pub fn from(id: &String) -> Option<Self> {
        match id.as_str() {
            lexem::APPEND => Some(AllocFn::Append {
                item_size: Cell::new(0),
            }),
            lexem::INSERT => Some(AllocFn::Insert),
            lexem::DELETE => Some(AllocFn::Delete),
            lexem::FREE => Some(AllocFn::Free),
            lexem::VEC => Some(AllocFn::Vec {
                with_capacity: Cell::new(false),
                item_size: Cell::new(0),
                metadata: Metadata::default(),
            }),
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
            AllocFn::Append { item_size } => {
                if extra.len() != 2 {
                    return Err(SemanticError::IncorrectArguments);
                }

                let vector = &extra[0];
                let item = &extra[1];

                let _ = vector.resolve(scope, &None, &())?;
                let vector_type = vector.type_of(&scope.borrow())?;
                if !vector_type.is_vec() {
                    return Err(SemanticError::IncorrectArguments);
                }
                let item_type = vector_type.get_item();

                let _ = item.resolve(scope, &item_type, &())?;
                let Some(item_type) = item_type else {
                    return Err(SemanticError::IncorrectArguments);
                };
                item_size.set(item_type.size_of());
                Ok(())
            }
            AllocFn::Insert => todo!(),
            AllocFn::Delete => todo!(),
            AllocFn::Free => todo!(),
            AllocFn::Vec {
                with_capacity,
                item_size,
                metadata,
            } => {
                if extra.len() > 2 || extra.len() < 1 {
                    return Err(SemanticError::IncorrectArguments);
                }
                if extra.len() == 2 {
                    with_capacity.set(true);
                }
                for param in extra {
                    let _ = param.resolve(
                        scope,
                        &Some(Either::Static(
                            StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
                        )),
                        &(),
                    )?;
                }
                if context.is_none() {
                    return Err(SemanticError::CantInferType);
                }
                match &context {
                    Some(value) => match value {
                        Either::Static(value) => match value.as_ref() {
                            StaticType::Vec(VecType(item)) => item_size.set(item.size_of()),
                            _ => return Err(SemanticError::IncompatibleTypes),
                        },
                        Either::User(_) => return Err(SemanticError::IncompatibleTypes),
                    },
                    None => unreachable!(),
                }
                let mut borrowed_metadata = metadata
                    .info
                    .as_ref()
                    .try_borrow_mut()
                    .map_err(|_| SemanticError::Default)?;
                *borrowed_metadata = Info::Resolved {
                    context: context.clone(),
                    signature: context.clone(),
                };
                Ok(())
            }
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
            AllocFn::Append { .. } => Ok(Either::Static(StaticType::Unit.into())),
            AllocFn::Insert => todo!(),
            AllocFn::Delete => todo!(),
            AllocFn::Free => todo!(),
            AllocFn::Vec { metadata, .. } => {
                metadata.signature().ok_or(SemanticError::NotResolvedYet)
            }
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
            AllocFn::Append { item_size } => gencode_append(scope, instructions, item_size.get()),
            AllocFn::Insert => todo!(),
            AllocFn::Delete => todo!(),
            AllocFn::Free => todo!(),
            AllocFn::Vec {
                with_capacity,
                item_size,
                ..
            } => gencode_vec(scope, instructions, item_size.get(), with_capacity.get()),
            AllocFn::Map => todo!(),
            AllocFn::Chan => todo!(),
            AllocFn::String => gencode_string(scope, instructions, params_size),
        }
    }
}

fn gencode_append<Scope: ScopeApi>(
    scope: &MutRc<Scope>,
    instructions: &CasmProgram,
    item_size: usize,
) -> Result<(), CodeGenerationError> {
    let else_label = Label::gen();
    let end_label = Label::gen();

    // Retrieve the heap_address of the vector
    instructions.push(Casm::Access(Access::Static {
        address: MemoryAddress::Stack {
            offset: Offset::ST(-((item_size + 8) as isize)),
            level: AccessLevel::Direct,
        },
        size: 8,
    }));
    instructions.push(Casm::MemCopy(MemCopy::Dup(8)));
    instructions.push(Casm::MemCopy(MemCopy::Dup(8)));
    // Retrieve the length of the vector
    instructions.push(Casm::Access(Access::Runtime { size: Some(16) }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Equal(Equal { left: 8, right: 8 }),
    }));
    instructions.push(Casm::If(BranchIf { else_label }));
    instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));
    instructions.push(Casm::Serialize(Serialized {
        data: (&(2u64).to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Mult(Mult {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    instructions.push(Casm::Serialize(Serialized {
        data: (&(item_size as u64).to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Mult(Mult {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    instructions.push(Casm::Serialize(Serialized {
        data: (&16u64.to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Addition(Addition {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    instructions.push(Casm::Realloc(Realloc { size: None }));
    instructions.push(Casm::Goto(Goto {
        label: Some(end_label),
    }));
    instructions.push_label_id(else_label, "append_no_realloc".into());
    instructions.push(Casm::Pop(8));
    instructions.push_label_id(end_label, "end_append".into());

    /* STACK : PREV_ADDR | ITEM | HEAP_ADDR |  */

    instructions.push(Casm::MemCopy(MemCopy::Dup(8)));
    /* STACK : PREV_ADDR | ITEM | HEAP_ADDR | HEAP_ADDR */
    instructions.push(Casm::Serialize(Serialized {
        data: (&8u64.to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Addition(Addition {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    /* STACK : PREV_ADDR | ITEM | HEAP_ADDR | HEAP_ADDR+8 */

    instructions.push(Casm::MemCopy(MemCopy::Dup(8)));

    /* STACK : PREV_ADDR | ITEM | HEAP_ADDR | HEAP_ADDR+8 | HEAP_ADDR+8 */
    instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));
    /* STACK : PREV_ADDR | ITEM | HEAP_ADDR | HEAP_ADDR+8 | Capacity */
    instructions.push(Casm::Serialize(Serialized {
        data: (&(2u64).to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Mult(Mult {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    /* update capacity */
    /* STACK : PREV_ADDR | ITEM | HEAP_ADDR | HEAP_ADDR+8 | Capacity | HEAP_ADDR+8 */
    instructions.push(Casm::Access(Access::Static {
        address: MemoryAddress::Stack {
            offset: Offset::ST(-16),
            level: AccessLevel::Direct,
        },
        size: 8,
    }));
    instructions.push(Casm::MemCopy(MemCopy::Take { size: 8 }));
    /* STACK : PREV_ADDR | ITEM | HEAP_ADDR | HEAP_ADDR+8 | HEAP_ADDR+8*/
    instructions.push(Casm::Pop(16));

    /* STACK : PREV_ADDR | ITEM | HEAP_ADDR |  */
    instructions.push(Casm::MemCopy(MemCopy::Dup(8)));
    instructions.push(Casm::MemCopy(MemCopy::Dup(8)));
    /* STACK : PREV_ADDR | ITEM | HEAP_ADDR | HEAP_ADDR | HEAP_ADDR  */
    instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));

    /* STACK : PREV_ADDR | ITEM | HEAP_ADDR | HEAP_ADDR | LENGTH */
    instructions.push(Casm::Serialize(Serialized {
        data: (&1u64.to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Addition(Addition {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    /* update length */
    instructions.push(Casm::Access(Access::Static {
        address: MemoryAddress::Stack {
            offset: Offset::ST(-16),
            level: AccessLevel::Direct,
        },
        size: 8,
    }));
    /* STACK : PREV_ADDR | ITEM | HEAP_ADDR | HEAP_ADDR | LENGTH | HEAP_ADDR */
    instructions.push(Casm::MemCopy(MemCopy::Take { size: 8 }));
    /* STACK : PREV_ADDR | ITEM | HEAP_ADDR | HEAP_ADDR | HEAP_ADDR  */
    instructions.push(Casm::Pop(8));
    /* STACK : PREV_ADDR | ITEM | HEAP_ADDR | HEAP_ADDR   */

    instructions.push(Casm::Access(Access::Runtime { size: Some(8) }));

    /* STACK : PREV_ADDR | ITEM | HEAP_ADDR  | LENGTH */
    instructions.push(Casm::Serialize(Serialized {
        data: (&1u64.to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Substraction(Substraction {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    /* STACK : PREV_ADDR | ITEM | HEAP_ADDR  | Index */
    instructions.push(Casm::Serialize(Serialized {
        data: (&(item_size as u64).to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Mult(Mult {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    instructions.push(Casm::Serialize(Serialized {
        data: (&16u64.to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Addition(Addition {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Addition(Addition {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    /* STACK : PREV_ADDR | ITEM | ITEM_HEAP_ADDR*/
    instructions.push(Casm::MemCopy(MemCopy::Take { size: item_size }));
    /* STACK : PREV_ADDR |*/
    instructions.push(Casm::Pop(8));

    Ok(())
}

fn gencode_vec<Scope: ScopeApi>(
    scope: &MutRc<Scope>,
    instructions: &CasmProgram,
    item_size: usize,
    with_capacity: bool,
) -> Result<(), CodeGenerationError> {
    if !with_capacity {
        instructions.push(Casm::MemCopy(MemCopy::Dup(8)));
        instructions.push(Casm::Operation(Operation {
            kind: OperationKind::Align,
        }))
    }
    instructions.push(Casm::MemCopy(MemCopy::Dup(8)));
    instructions.push(Casm::Serialize(Serialized {
        data: (&(item_size as u64).to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Mult(Mult {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));

    instructions.push(Casm::Serialize(Serialized {
        data: (&16u64.to_le_bytes()).to_vec(),
    }));
    instructions.push(Casm::Operation(Operation {
        kind: OperationKind::Addition(Addition {
            left: OpPrimitive::Number(NumberType::U64),
            right: OpPrimitive::Number(NumberType::U64),
        }),
    }));
    instructions.push(Casm::Alloc(Alloc::Heap { size: None }));
    instructions.push(Casm::MemCopy(MemCopy::TakeToHeap {
        //offset: vec_stack_address + 8,
        size: 16,
    }));

    Ok(())
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
        size: Some(align(params_size + 16)),
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

#[cfg(test)]
mod tests {
    use crate::{
        ast::{statements::Statement, TryParse},
        clear_stack,
        semantic::scope::scope_impl::Scope,
        vm::vm::{DeserializeFrom, Runtime},
    };

    use super::*;

    #[test]
    fn valid_string() {
        let statement = Statement::parse(
            r##"
        let x = string("Hello World");
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Resolution should have succeeded");
        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
        // Execute the instructions.
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        thread.push_instr(instructions);

        thread.run().expect("Execution should have succeeded");
        let memory = &thread.memory();
        let data = clear_stack!(memory);
        let heap_address = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;

        let data_length = memory
            .heap
            .read(heap_address, 8)
            .expect("length should be readable");
        let length = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data_length[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;

        let data = memory
            .heap
            .read(heap_address, length + 16)
            .expect("length should be readable");

        let result = <StringType as DeserializeFrom<Scope>>::deserialize_from(&StringType(), &data)
            .expect("Deserialization should have succeeded");

        assert_eq!(result.value, "Hello World");
    }

    #[test]
    fn valid_vec() {
        let statement = Statement::parse(
            r##"
        let x:Vec<u64> = vec(8);
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Resolution should have succeeded");
        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
        // Execute the instructions.
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        thread.push_instr(instructions);

        thread.run().expect("Execution should have succeeded");
        let memory = &thread.memory();
        let data = clear_stack!(memory);
        let heap_address = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;
        let data_length = memory
            .heap
            .read(heap_address, 8)
            .expect("length should be readable");
        let length = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data_length[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;
        assert_eq!(length, 8);
    }

    #[test]
    fn valid_append() {
        let statement = Statement::parse(
            r##"
            let res = {
                let x:Vec<u64> = vec[1,2,3,4,5,6,7,8];
                append(x,9);
                return x;
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Resolution should have succeeded");
        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0, "No instructions generated");
        // Execute the instructions.
        let mut runtime = Runtime::new();
        let tid = runtime
            .spawn()
            .expect("Thread spawning should have succeeded");
        let thread = runtime.get(tid).expect("Thread should exist");
        thread.push_instr(instructions);

        thread.run().expect("Execution should have succeeded");
        let memory = &thread.memory();
        let data = clear_stack!(memory);
        let heap_address = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;
        let data_length = memory
            .heap
            .read(heap_address, 8)
            .expect("length should be readable");
        let length = u64::from_le_bytes(
            TryInto::<[u8; 8]>::try_into(&data_length[0..8])
                .expect("heap address should be deserializable"),
        ) as usize;
        assert_eq!(length, 9);

        let data = memory
            .heap
            .read(heap_address as usize, 8 * 9 + 16)
            .expect("Heap Read should have succeeded");
        let data: Vec<u64> = data
            .chunks(8)
            .map(|chunk| {
                u64::from_le_bytes(
                    TryInto::<[u8; 8]>::try_into(&chunk[0..8])
                        .expect("heap address should be deserializable"),
                )
            })
            .collect();
        assert_eq!(data, vec![9, 16, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }
}

use crate::{
    ast::expressions::Expression,
    semantic::{
        scope::static_types::{MapType, StaticType, VecType},
        CompatibleWith, EType, Resolve, ResolveCore, SemanticError, SizeOf, TypeOf,
    },
    vm::{
        allocator::{heap::Heap, stack::Stack, MemoryAddress},
        asm::{
            operation::{GetNumFrom, OpPrimitive, PopNum},
            Asm,
        },
        core::{lexem, CoreAsm, ERROR_SLICE, OK_SLICE},
        runtime::RuntimeError,
        scheduler::Executable,
        stdio::StdIO,
        GenerateCode,
    },
};

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use super::{string::STRING_HEADER, vector::VEC_HEADER, PathFinder};
use crate::vm::allocator::heap::{HeapError, HEAP_SIZE};
use num_traits::ToBytes;
use rand::Rng;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum DerefHashing {
    Vec(usize),
    String,
    StrSlice,
    Default,
}
impl From<&EType> for DerefHashing {
    fn from(value: &EType) -> Self {
        match value {
            EType::Static(tmp) => match tmp {
                StaticType::String(_) => DerefHashing::String,
                StaticType::StrSlice(_) => DerefHashing::StrSlice,
                StaticType::Vec(VecType(item_subtype)) => DerefHashing::Vec(item_subtype.size_of()),
                _ => DerefHashing::Default,
            },
            EType::User { .. } => DerefHashing::Default,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MapFn {
    Map {
        with_capacity: bool,
        item_size: usize,
        key_size: usize,
    },
    Insert {
        key_size: usize,
        item_size: usize,
        ref_access: DerefHashing,
    },
    Get {
        key_size: usize,
        item_size: usize,
        ref_access: DerefHashing,
    },
    DelKey {
        key_size: usize,
        item_size: usize,
        ref_access: DerefHashing,
    },
    Clear {
        key_size: usize,
        item_size: usize,
        ref_access: DerefHashing,
    },
}

impl PathFinder for MapFn {
    fn find(path: &[String], name: &str) -> Option<Self>
    where
        Self: Sized,
    {
        if (path.len() == 1 && path[0] == lexem::MAP) || path.len() == 0 {
            return match name {
                lexem::MAP => Some(MapFn::Map {
                    with_capacity: false,
                    item_size: 0,
                    key_size: 0,
                }),
                lexem::INSERT => Some(MapFn::Insert {
                    item_size: 0,
                    key_size: 0,
                    ref_access: DerefHashing::Default,
                }),
                lexem::GET => Some(MapFn::Get {
                    item_size: 0,
                    key_size: 0,
                    ref_access: DerefHashing::Default,
                }),
                lexem::DELKEY => Some(MapFn::DelKey {
                    item_size: 0,
                    key_size: 0,
                    ref_access: DerefHashing::Default,
                }),
                lexem::CLEAR_MAP => Some(MapFn::DelKey {
                    item_size: 0,
                    key_size: 0,
                    ref_access: DerefHashing::Default,
                }),
                _ => None,
            };
        }
        None
    }
}

impl ResolveCore for MapFn {
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: Option<&EType>,
        parameters: &mut Vec<Expression>,
    ) -> Result<EType, SemanticError> {
        fn map_param<E: crate::vm::external::Engine>(
            param: &mut Expression,
            scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
            scope_id: Option<u128>,
        ) -> Result<MapType, SemanticError> {
            let _ = param.resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
            let EType::Static(StaticType::Map(map_type)) =
                param.type_of(scope_manager, scope_id)?
            else {
                return Err(SemanticError::IncorrectArguments);
            };
            Ok(map_type)
        }
        match self {
            MapFn::Map {
                with_capacity,
                item_size,
                key_size,
            } => {
                let Some(
                    context @ EType::Static(StaticType::Map(MapType {
                        keys_type,
                        values_type,
                    })),
                ) = context
                else {
                    return Err(SemanticError::CantInferType(format!(
                        "of this map allocation"
                    )));
                };
                if parameters.len() == 0 || parameters.len() > 2 {
                    return Err(SemanticError::IncorrectArguments);
                }

                for param in parameters.iter_mut() {
                    let _ = param.resolve::<E>(
                        scope_manager,
                        scope_id,
                        &Some(crate::p_num!(U64)),
                        &mut None,
                    )?;
                    let crate::p_num!(U64) = param.type_of(&scope_manager, scope_id)? else {
                        return Err(SemanticError::IncorrectArguments);
                    };
                }

                *with_capacity = parameters.len() == 2;
                *key_size = keys_type.size_of();
                *item_size = values_type.size_of();

                Ok(context.clone())
            }
            MapFn::Insert {
                key_size,
                item_size,
                ref_access,
            } => {
                if parameters.len() != 3 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let (first_part, second_part) = parameters.split_at_mut(1);
                let map = &mut first_part[0];

                let (first_part, second_part) = second_part.split_at_mut(1);
                let key = &mut first_part[0];
                let item = &mut second_part[1];

                let map_type = map_param::<E>(map, scope_manager, scope_id)?;

                let _ = item.resolve::<E>(
                    scope_manager,
                    scope_id,
                    &Some(map_type.values_type.as_ref().clone()),
                    &mut None,
                )?;
                let item_type = item.type_of(scope_manager, scope_id)?;
                let _ = item_type.compatible_with(
                    map_type.values_type.as_ref(),
                    scope_manager,
                    scope_id,
                )?;

                let _ = key.resolve::<E>(
                    scope_manager,
                    scope_id,
                    &Some(map_type.values_type.as_ref().clone()),
                    &mut None,
                )?;
                let key_type = key.type_of(scope_manager, scope_id)?;
                let _ = key_type.compatible_with(
                    map_type.values_type.as_ref(),
                    scope_manager,
                    scope_id,
                )?;

                *item_size = item_type.size_of();
                *key_size = key_type.size_of();

                // should deref the key
                match key_type {
                    EType::Static(StaticType::Vec(VecType(inner))) => {
                        *ref_access = DerefHashing::Vec(inner.as_ref().size_of())
                    }
                    EType::Static(StaticType::StrSlice(_)) => *ref_access = DerefHashing::StrSlice,
                    EType::Static(StaticType::String(_)) => *ref_access = DerefHashing::String,
                    _ => {}
                }

                Ok(EType::Static(StaticType::Map(map_type)))
            }
            MapFn::Get {
                key_size,
                item_size,
                ref_access,
            }
            | MapFn::DelKey {
                key_size,
                item_size,
                ref_access,
            } => {
                if parameters.len() != 2 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let (first_part, second_part) = parameters.split_at_mut(1);
                let map = &mut first_part[0];
                let key = &mut second_part[1];

                let map_type = map_param::<E>(map, scope_manager, scope_id)?;

                let _ = key.resolve::<E>(
                    scope_manager,
                    scope_id,
                    &Some(map_type.values_type.as_ref().clone()),
                    &mut None,
                )?;
                let key_type = key.type_of(scope_manager, scope_id)?;
                let _ = key_type.compatible_with(
                    map_type.values_type.as_ref(),
                    scope_manager,
                    scope_id,
                )?;

                *item_size = map_type.values_type.size_of();
                *key_size = key_type.size_of();

                // should deref the key
                *ref_access = (&key_type).into();
                Ok(map_type.values_type.as_ref().clone())
            }
            MapFn::Clear {
                key_size,
                item_size,
                ref_access,
            } => {
                if parameters.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let map = &mut parameters[0];
                let map_type = map_param::<E>(map, scope_manager, scope_id)?;

                Ok(EType::Static(StaticType::Map(map_type)))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MapAsm {
    Map {
        item_size: usize,
        key_size: usize,
    },
    MapWithCapacity {
        item_size: usize,
        key_size: usize,
    },
    Insert {
        item_size: usize,
        key_size: usize,
        ref_access: DerefHashing,
    },
    Get {
        item_size: usize,
        key_size: usize,
        ref_access: DerefHashing,
    },
    DelKey {
        item_size: usize,
        key_size: usize,
        ref_access: DerefHashing,
    },
    Clear {
        item_size: usize,
        key_size: usize,
    },
}

impl<E: crate::vm::external::Engine> crate::vm::AsmName<E> for MapAsm {
    fn name(&self, stdio: &mut StdIO, program: &crate::vm::program::Program<E>, engine: &mut E) {
        match self {
            MapAsm::Map { .. } => stdio.push_asm_lib(engine, "map"),
            MapAsm::MapWithCapacity { .. } => stdio.push_asm_lib(engine, "map"),
            MapAsm::Insert { .. } => stdio.push_asm_lib(engine, "insert"),
            MapAsm::DelKey { .. } => stdio.push_asm_lib(engine, "del_key"),
            MapAsm::Get { .. } => stdio.push_asm_lib(engine, "get"),
            MapAsm::Clear { .. } => stdio.push_asm_lib(engine, "clear_map"),
        }
    }
}

impl crate::vm::AsmWeight for MapAsm {
    fn weight(&self) -> crate::vm::Weight {
        match self {
            MapAsm::Map { .. } => crate::vm::Weight::MEDIUM,
            MapAsm::MapWithCapacity { .. } => crate::vm::Weight::MEDIUM,
            MapAsm::Insert { .. } => crate::vm::Weight::HIGH,
            MapAsm::DelKey { .. } => crate::vm::Weight::HIGH,
            MapAsm::Get { .. } => crate::vm::Weight::MEDIUM,
            MapAsm::Clear { .. } => crate::vm::Weight::MEDIUM,
        }
    }
}

impl GenerateCode for MapFn {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        match *self {
            MapFn::Map {
                with_capacity,
                item_size,
                key_size,
            } => {
                if with_capacity {
                    instructions.push(Asm::Core(CoreAsm::Map(MapAsm::MapWithCapacity {
                        item_size,
                        key_size,
                    })));
                } else {
                    instructions.push(Asm::Core(CoreAsm::Map(MapAsm::Map {
                        item_size,
                        key_size,
                    })));
                }
            }
            MapFn::Insert {
                key_size,
                item_size,
                ref_access,
            } => {
                instructions.push(Asm::Core(CoreAsm::Map(MapAsm::Insert {
                    item_size,
                    key_size,
                    ref_access,
                })));
            }
            MapFn::Get {
                key_size,
                item_size,
                ref_access,
            } => {
                instructions.push(Asm::Core(CoreAsm::Map(MapAsm::Get {
                    item_size,
                    key_size,
                    ref_access,
                })));
            }
            MapFn::DelKey {
                key_size,
                item_size,
                ref_access,
            } => {
                instructions.push(Asm::Core(CoreAsm::Map(MapAsm::DelKey {
                    item_size,
                    key_size,
                    ref_access,
                })));
            }
            MapFn::Clear {
                key_size,
                item_size,
                ref_access,
            } => {
                instructions.push(Asm::Core(CoreAsm::Map(MapAsm::Clear {
                    item_size,
                    key_size,
                })));
            }
        }
        Ok(())
    }
}

pub const MAP_BUCKET_SIZE: usize = 8;
enum TopHashValue {
    RestIsEmpty = 0,
    EmptyCell = 1,
    MIN = 2,
}

/*
    MAP ALLOCATION LAYOUT:
        log_cap : u64, // (called B in go) log_2 of number of buckets
        len : u64,
        hash_seed : u64,
        ptr_buckets : u64,
*/

pub const MAP_LAYOUT_SIZE: usize = 4 * 8;

#[derive(Debug)]
pub struct MapLayout {
    pub ptr_map_layout: MemoryAddress,
    pub bucket_size: usize,
    pub key_size: usize,
    pub value_size: usize,
    pub len: u64,
    pub log_cap: u8, // (called B in go) log_2 of number of buckets
    pub hash_seed: u32,
    pub ptr_buckets: MemoryAddress,
}
#[derive(Debug)]
pub struct BucketLayout {
    pub ptr_top_hash: MemoryAddress,
    pub keys_top_hash: [u8; MAP_BUCKET_SIZE],
    pub ptr_keys: MemoryAddress,
    pub key_size: usize,
    pub ptr_values: MemoryAddress,
    pub value_size: usize,
}

struct AssignResult {
    tophash_address: MemoryAddress,
    key_address: MemoryAddress,
    item_address: MemoryAddress,
    is_new_value: bool,
}

impl BucketLayout {
    fn assign(
        &self,
        top_hash: u8,
        key: &[u8],
        ref_access: DerefHashing,
        stack: &mut Stack,
        heap: &mut Heap,
    ) -> Result<Option<AssignResult>, RuntimeError> {
        let mut indexes = None;
        let mut first_empty_cell = None;
        for (idx, self_top_hash) in self.keys_top_hash.iter().enumerate() {
            if *self_top_hash == TopHashValue::EmptyCell as u8 {
                if first_empty_cell.is_none() {
                    first_empty_cell = Some(idx);
                }
                continue;
            }
            if *self_top_hash == top_hash {
                // Read key
                match ref_access {
                    DerefHashing::Vec(item_size) => {
                        // found_key is a pointer to a vec
                        let key_address: MemoryAddress = OpPrimitive::get_num_from::<u64>(
                            self.ptr_keys.add(idx * self.key_size),
                            stack,
                            heap,
                        )?
                        .try_into()?;

                        let len =
                            OpPrimitive::get_num_from::<u64>(key_address.add(8), stack, heap)?;

                        let items_bytes = heap
                            .read_slice(key_address.add(STRING_HEADER), len as usize * item_size)?;

                        if items_bytes == key {
                            indexes = Some(AssignResult {
                                tophash_address: self.ptr_top_hash.add(idx),
                                key_address: self.ptr_keys.add(idx * self.key_size),
                                item_address: self.ptr_values.add(idx * self.value_size),
                                is_new_value: false,
                            })
                        }
                    }
                    DerefHashing::String => {
                        // found_key is a pointer to a string
                        let key_address: MemoryAddress = OpPrimitive::get_num_from::<u64>(
                            self.ptr_keys.add(idx * self.key_size),
                            stack,
                            heap,
                        )?
                        .try_into()?;

                        let len =
                            OpPrimitive::get_num_from::<u64>(key_address.add(8), stack, heap)?;

                        let items_bytes =
                            heap.read_slice(key_address.add(VEC_HEADER), len as usize)?;

                        if items_bytes == key {
                            indexes = Some(AssignResult {
                                tophash_address: self.ptr_top_hash.add(idx),
                                key_address: self.ptr_keys.add(idx * self.key_size),
                                item_address: self.ptr_values.add(idx * self.value_size),
                                is_new_value: false,
                            })
                        }
                    }
                    DerefHashing::StrSlice => {
                        // found_key is a pointer to a str slice
                        let key_address: MemoryAddress = OpPrimitive::get_num_from::<u64>(
                            self.ptr_keys.add(idx * self.key_size),
                            stack,
                            heap,
                        )?
                        .try_into()?;

                        let len = OpPrimitive::get_num_from::<u64>(key_address, stack, heap)?;

                        let items_bytes = heap.read_slice(key_address.add(8), len as usize)?;

                        if items_bytes == key {
                            indexes = Some(AssignResult {
                                tophash_address: self.ptr_top_hash.add(idx),
                                key_address: self.ptr_keys.add(idx * self.key_size),
                                item_address: self.ptr_values.add(idx * self.value_size),
                                is_new_value: false,
                            })
                        }
                    }
                    DerefHashing::Default => {
                        let found_key =
                            heap.read_slice(self.ptr_keys.add(idx * self.key_size), self.key_size)?;

                        if found_key == key {
                            indexes = Some(AssignResult {
                                tophash_address: self.ptr_top_hash.add(idx),
                                key_address: self.ptr_keys.add(idx * self.key_size),
                                item_address: self.ptr_values.add(idx * self.value_size),
                                is_new_value: false,
                            })
                        }
                    }
                }
            }
            if *self_top_hash == TopHashValue::RestIsEmpty as u8 {
                if first_empty_cell.is_none() {
                    first_empty_cell = Some(idx);
                }
                break;
            }
        }
        if indexes.is_none() {
            if let Some(idx) = first_empty_cell {
                return Ok(Some(AssignResult {
                    tophash_address: self.ptr_top_hash.add(idx),
                    key_address: self.ptr_keys.add(idx * self.key_size),
                    item_address: self.ptr_values.add(idx * self.value_size),
                    is_new_value: true,
                }));
            } else {
                return Ok(None);
            }
        } else {
            return Ok(indexes);
        }
    }
    pub fn get(
        &self,
        top_hash: u8,
        key: &[u8],
        ref_access: DerefHashing,
        stack: &mut Stack,
        heap: &mut Heap,
    ) -> Result<Option<MemoryAddress>, RuntimeError> {
        for (idx, self_top_hash) in self.keys_top_hash.iter().enumerate() {
            if *self_top_hash == TopHashValue::EmptyCell as u8 {
                continue;
            }
            if *self_top_hash == top_hash {
                // Read key
                match ref_access {
                    DerefHashing::Vec(item_size) => {
                        // found_key is a pointer to a vec
                        let key_address: MemoryAddress = OpPrimitive::get_num_from::<u64>(
                            self.ptr_keys.add(idx * self.key_size),
                            stack,
                            heap,
                        )?
                        .try_into()?;

                        let len =
                            OpPrimitive::get_num_from::<u64>(key_address.add(8), stack, heap)?;

                        let items_bytes = heap
                            .read_slice(key_address.add(STRING_HEADER), len as usize * item_size)?;

                        if items_bytes == key {
                            return Ok(Some(self.ptr_keys.add(idx * self.value_size)));
                        }
                    }
                    DerefHashing::String => {
                        // found_key is a pointer to a string
                        let key_address: MemoryAddress = OpPrimitive::get_num_from::<u64>(
                            self.ptr_keys.add(idx * self.key_size),
                            stack,
                            heap,
                        )?
                        .try_into()?;

                        let len =
                            OpPrimitive::get_num_from::<u64>(key_address.add(8), stack, heap)?;

                        let items_bytes =
                            heap.read_slice(key_address.add(VEC_HEADER), len as usize)?;

                        if items_bytes == key {
                            return Ok(Some(self.ptr_keys.add(idx * self.value_size)));
                        }
                    }
                    DerefHashing::StrSlice => {
                        // found_key is a pointer to a str slice
                        let key_address: MemoryAddress = OpPrimitive::get_num_from::<u64>(
                            self.ptr_keys.add(idx * self.key_size),
                            stack,
                            heap,
                        )?
                        .try_into()?;

                        let len = OpPrimitive::get_num_from::<u64>(key_address, stack, heap)?;

                        let items_bytes = heap.read_slice(key_address.add(8), len as usize)?;

                        if items_bytes == key {
                            return Ok(Some(self.ptr_keys.add(idx * self.value_size)));
                        }
                    }
                    DerefHashing::Default => {
                        let found_key =
                            heap.read_slice(self.ptr_keys.add(idx * self.key_size), self.key_size)?;

                        if found_key == key {
                            return Ok(Some(self.ptr_keys.add(idx * self.value_size)));
                        }
                    }
                }
            }
            if *self_top_hash == TopHashValue::RestIsEmpty as u8 {
                break;
            }
        }
        return Ok(None);
    }
    pub fn delete(
        &self,
        top_hash: u8,
        key: &[u8],
        ref_access: DerefHashing,
        stack: &mut Stack,
        heap: &mut Heap,
    ) -> Result<Option<MemoryAddress>, RuntimeError> {
        let mut found_idx = None;
        for (idx, self_top_hash) in self.keys_top_hash.iter().enumerate() {
            if *self_top_hash == TopHashValue::EmptyCell as u8 {
                continue;
            }
            if *self_top_hash == top_hash {
                // Read key
                match ref_access {
                    DerefHashing::Vec(item_size) => {
                        // found_key is a pointer to a vec
                        let key_address: MemoryAddress = OpPrimitive::get_num_from::<u64>(
                            self.ptr_keys.add(idx * self.key_size),
                            stack,
                            heap,
                        )?
                        .try_into()?;

                        let len =
                            OpPrimitive::get_num_from::<u64>(key_address.add(8), stack, heap)?;

                        let items_bytes = heap
                            .read_slice(key_address.add(STRING_HEADER), len as usize * item_size)?;

                        if items_bytes == key {
                            found_idx = Some(idx);
                        }
                    }
                    DerefHashing::String => {
                        // found_key is a pointer to a string
                        let key_address: MemoryAddress = OpPrimitive::get_num_from::<u64>(
                            self.ptr_keys.add(idx * self.key_size),
                            stack,
                            heap,
                        )?
                        .try_into()?;

                        let len =
                            OpPrimitive::get_num_from::<u64>(key_address.add(8), stack, heap)?;

                        let items_bytes =
                            heap.read_slice(key_address.add(VEC_HEADER), len as usize)?;

                        if items_bytes == key {
                            found_idx = Some(idx);
                        }
                    }
                    DerefHashing::StrSlice => {
                        // found_key is a pointer to a str slice
                        let key_address: MemoryAddress = OpPrimitive::get_num_from::<u64>(
                            self.ptr_keys.add(idx * self.key_size),
                            stack,
                            heap,
                        )?
                        .try_into()?;

                        let len = OpPrimitive::get_num_from::<u64>(key_address, stack, heap)?;

                        let items_bytes = heap.read_slice(key_address.add(8), len as usize)?;

                        if items_bytes == key {
                            found_idx = Some(idx);
                        }
                    }
                    DerefHashing::Default => {
                        let found_key =
                            heap.read_slice(self.ptr_keys.add(idx * self.key_size), self.key_size)?;

                        if found_key == key {
                            found_idx = Some(idx);
                        }
                    }
                }
            }
            if *self_top_hash == TopHashValue::RestIsEmpty as u8 {
                break;
            }
        }
        // Update Cell
        if let Some(idx) = found_idx {
            if idx < MAP_BUCKET_SIZE - 1 {
                // Get next cell
                let next_top_hash = self.keys_top_hash[idx + 1];
                if next_top_hash == TopHashValue::RestIsEmpty as u8 {
                    // Write RestIsEmpty
                    heap.write(
                        self.ptr_top_hash.add(idx),
                        &vec![TopHashValue::RestIsEmpty as u8],
                    )?;
                } else {
                    // Write EmptyCell
                    heap.write(
                        self.ptr_top_hash.add(idx),
                        &vec![TopHashValue::EmptyCell as u8],
                    )?;
                }
            } else {
                // Write RestIsEmpty
                heap.write(
                    self.ptr_top_hash.add(idx),
                    &vec![TopHashValue::RestIsEmpty as u8],
                )?;
            }
            Ok(Some(self.ptr_values.add(idx * self.value_size)))
        } else {
            Ok(None)
        }
    }
}

impl MapLayout {
    fn len_offset() -> usize {
        8
    }

    fn log_cap_offset() -> usize {
        0
    }

    fn hash_seed_offset() -> usize {
        8 * 2
    }

    pub fn ptr_buckets_offset() -> usize {
        8 * 3
    }

    pub fn new(key_size: usize, value_size: usize, log_cap: u8) -> Self {
        Self {
            ptr_map_layout: MemoryAddress::default(),
            bucket_size: MAP_BUCKET_SIZE
                + MAP_BUCKET_SIZE * key_size
                + MAP_BUCKET_SIZE * value_size,
            key_size,
            value_size,
            len: 0,
            log_cap,
            hash_seed: gen_seed(),
            ptr_buckets: MemoryAddress::default(),
        }
    }

    pub fn init_in_mem(
        &self,
        stack: &Stack,
        heap: &mut Heap,
    ) -> Result<MemoryAddress, RuntimeError> {
        // alloc map layout
        let map_ptr = heap.alloc(MAP_LAYOUT_SIZE)?;

        let mut data = [0; MAP_LAYOUT_SIZE];
        // write log_cap
        data[0..8].copy_from_slice(&(self.log_cap as u64).to_le_bytes());
        // write len
        data[8..16].copy_from_slice(&self.len.to_le_bytes());
        // write seed
        data[16..24].copy_from_slice(&(self.hash_seed as u64).to_le_bytes());

        // alloc buckets
        let buckets_ptr = heap.alloc((1 << self.log_cap) * self.bucket_size)?;
        // clean buckets
        let _ = heap.write(
            buckets_ptr,
            &vec![0u8; (1 << self.log_cap) * self.bucket_size],
        )?;

        let buckets_ptr: u64 = buckets_ptr.into(stack);

        // write buckets_ptr
        data[24..32].copy_from_slice(&buckets_ptr.to_le_bytes());

        // write map layout in mem
        let _ = heap.write(map_ptr, &data.to_vec())?;

        Ok(map_ptr)
    }

    fn update_log_cap(&self, new_log_cap: u8, heap: &mut Heap) -> Result<(), RuntimeError> {
        let _ = heap.write(
            self.ptr_map_layout.add(MapLayout::ptr_buckets_offset()),
            &[new_log_cap],
        )?;
        Ok(())
    }

    fn update_buckets_ptr(
        &self,
        bucket_ptr: MemoryAddress,
        stack: &Stack,
        heap: &mut Heap,
    ) -> Result<(), RuntimeError> {
        let bucket_ptr: u64 = bucket_ptr.into(stack);
        let _ = heap.write(
            self.ptr_map_layout.add(MapLayout::ptr_buckets_offset()),
            &bucket_ptr.to_le_bytes(),
        )?;
        Ok(())
    }

    pub fn resize(&self, stack: &Stack, heap: &mut Heap) -> Result<(), RuntimeError> {
        let mut new_log_cap = self.log_cap + 1;

        let previous_ptr_bucket = self.ptr_buckets;
        // get all buckets
        let bytes_buckets = heap.read(self.ptr_buckets, (1 << self.log_cap) * self.bucket_size)?;
        let mut resizing_is_over = false;
        let mut new_bytes_buckets = Vec::new();
        'again: while !resizing_is_over {
            let alloc_size = (1 << new_log_cap) * self.bucket_size;
            if alloc_size > HEAP_SIZE {
                return Err(RuntimeError::HeapError(HeapError::AllocationError));
            }
            new_bytes_buckets = vec![0u8; (1 << new_log_cap) * self.bucket_size];

            for bucket in bytes_buckets.chunks_exact(self.bucket_size) {
                let key_value_pair = bucket
                    [MAP_BUCKET_SIZE..MAP_BUCKET_SIZE + MAP_BUCKET_SIZE * self.key_size]
                    .chunks_exact(self.key_size)
                    .zip(
                        bucket[MAP_BUCKET_SIZE + MAP_BUCKET_SIZE * self.key_size
                            ..MAP_BUCKET_SIZE
                                + MAP_BUCKET_SIZE * self.key_size
                                + MAP_BUCKET_SIZE * self.value_size]
                            .chunks_exact(self.value_size),
                    )
                    .enumerate();
                for (idx_key_value, (key, value)) in key_value_pair {
                    if bucket[idx_key_value] <= TopHashValue::MIN as u8 {
                        // Skip empty cell
                        continue;
                    }
                    // Compute hash
                    let hash = hash_of(key, self.hash_seed);
                    let new_bucket_idx = bucket_idx(hash, new_log_cap) as usize;
                    // index of key_top_hash in new bucket
                    let ptr_new_bucket = new_bucket_idx * self.bucket_size;
                    let mut idx_in_new_bucket = None;
                    for idx_top_hash in 0..MAP_BUCKET_SIZE {
                        if new_bytes_buckets[ptr_new_bucket + idx_top_hash]
                            <= TopHashValue::MIN as u8
                        {
                            idx_in_new_bucket = Some(idx_top_hash);
                            break;
                        }
                    }
                    if idx_in_new_bucket.is_none() {
                        new_log_cap += 1;
                        continue 'again;
                    }
                    let idx_in_new_bucket = idx_in_new_bucket.unwrap();
                    // Update top_hash, key and value in the found idx
                    // update top_hash
                    new_bytes_buckets[ptr_new_bucket + idx_in_new_bucket] = top_hash(hash);
                    // update key
                    new_bytes_buckets[ptr_new_bucket
                        + MAP_BUCKET_SIZE
                        + idx_in_new_bucket * self.key_size
                        ..ptr_new_bucket
                            + MAP_BUCKET_SIZE
                            + idx_in_new_bucket * self.key_size
                            + self.key_size]
                        .copy_from_slice(key);

                    // update value
                    new_bytes_buckets[ptr_new_bucket
                        + MAP_BUCKET_SIZE
                        + MAP_BUCKET_SIZE * self.key_size
                        + idx_in_new_bucket * self.value_size
                        ..ptr_new_bucket
                            + MAP_BUCKET_SIZE
                            + MAP_BUCKET_SIZE * self.key_size
                            + idx_in_new_bucket * self.value_size
                            + self.value_size]
                        .copy_from_slice(value);
                }
            }
            resizing_is_over = true;
        }

        // Update log_cap in memory: log_cap += 1
        let _ = self.update_log_cap(new_log_cap, heap)?;

        // free previous_buckets
        let _ = heap.free(previous_ptr_bucket)?;

        // alloc new buckets
        let new_buckets_ptr = heap.alloc(new_bytes_buckets.len())?;

        // copy new buckets in memory
        let _ = heap.write(new_buckets_ptr, &new_bytes_buckets)?;

        // update buckets ptr
        let _ = self.update_buckets_ptr(new_buckets_ptr, stack, heap)?;

        Ok(())
    }

    pub fn clear_buckets(&self, heap: &mut Heap) -> Result<(), RuntimeError> {
        let _ = heap.write(
            self.ptr_buckets,
            &vec![0; (1 << self.log_cap) * self.bucket_size],
        )?;
        Ok(())
    }

    pub fn retrieve_vec_values(&self, heap: &mut Heap) -> Result<Vec<MemoryAddress>, RuntimeError> {
        // get all buckets
        let bytes_buckets = heap.read(self.ptr_buckets, (1 << self.log_cap) * self.bucket_size)?;

        let mut items_ptr = Vec::with_capacity(self.len as usize);

        for (idx, bucket) in bytes_buckets.chunks_exact(self.bucket_size).enumerate() {
            for idx_top_hash in 0..MAP_BUCKET_SIZE {
                if bucket[idx_top_hash] > TopHashValue::MIN as u8 {
                    items_ptr.push(self.ptr_buckets.add(
                        idx * self.bucket_size
                            + MAP_BUCKET_SIZE
                            + MAP_BUCKET_SIZE * self.key_size
                            + idx_top_hash * self.value_size,
                    ))
                }
            }
        }
        Ok(items_ptr)
    }
    pub fn retrieve_vec_keys(&self, heap: &mut Heap) -> Result<Vec<MemoryAddress>, RuntimeError> {
        // get all buckets
        let bytes_buckets = heap.read(self.ptr_buckets, (1 << self.log_cap) * self.bucket_size)?;

        let mut items_ptr = Vec::with_capacity(self.len as usize);

        for (idx, bucket) in bytes_buckets.chunks_exact(self.bucket_size).enumerate() {
            for idx_top_hash in 0..MAP_BUCKET_SIZE {
                if bucket[idx_top_hash] > TopHashValue::MIN as u8 {
                    items_ptr.push(self.ptr_buckets.add(
                        idx * self.bucket_size + MAP_BUCKET_SIZE + idx_top_hash * self.key_size,
                    ))
                }
            }
        }
        Ok(items_ptr)
    }
    pub fn retrieve_vec_items(
        &self,
        heap: &mut Heap,
    ) -> Result<Vec<(MemoryAddress, MemoryAddress)>, RuntimeError> {
        // get all buckets
        let bytes_buckets = heap.read(self.ptr_buckets, (1 << self.log_cap) * self.bucket_size)?;

        let mut items_ptr = Vec::with_capacity(self.len as usize);

        for (idx, bucket) in bytes_buckets.chunks_exact(self.bucket_size).enumerate() {
            for idx_top_hash in 0..MAP_BUCKET_SIZE {
                if bucket[idx_top_hash] > TopHashValue::MIN as u8 {
                    items_ptr.push((
                        self.ptr_buckets.add(
                            idx * self.bucket_size + MAP_BUCKET_SIZE + idx_top_hash * self.key_size,
                        ),
                        self.ptr_buckets.add(
                            idx * self.bucket_size
                                + MAP_BUCKET_SIZE
                                + MAP_BUCKET_SIZE * self.key_size
                                + idx_top_hash * self.value_size,
                        ),
                    ))
                }
            }
        }
        Ok(items_ptr)
    }
}

pub fn over_load_factor(size: u64, log_cap: u8) -> bool {
    size > MAP_BUCKET_SIZE as u64 && size > ((3 * (1 << log_cap) * MAP_BUCKET_SIZE as u64) / 4)
}

pub fn bucket_layout(
    address: MemoryAddress,
    key_size: usize,
    value_size: usize,
    heap: &mut Heap,
) -> Result<BucketLayout, RuntimeError> {
    let data = heap.read(address, MAP_BUCKET_SIZE)?;
    if data.len() != MAP_BUCKET_SIZE {
        return Err(RuntimeError::CodeSegmentation);
    }
    let keys_top_hash: [u8; MAP_BUCKET_SIZE] =
        data.try_into().map_err(|_| RuntimeError::Deserialization)?;

    Ok(BucketLayout {
        ptr_top_hash: address,
        keys_top_hash,
        ptr_keys: address.add(MAP_BUCKET_SIZE),
        key_size,
        ptr_values: address.add(MAP_BUCKET_SIZE + MAP_BUCKET_SIZE * key_size),
        value_size,
    })
}

pub fn map_layout(
    address: MemoryAddress,
    key_size: usize,
    value_size: usize,
    heap: &mut Heap,
) -> Result<MapLayout, RuntimeError> {
    let data = heap.read(address, MAP_LAYOUT_SIZE)?;
    if data.len() != MAP_LAYOUT_SIZE {
        return Err(RuntimeError::CodeSegmentation);
    }
    let log_cap: u8 = u64::from_le_bytes(
        data[0..8]
            .try_into()
            .map_err(|_| RuntimeError::Deserialization)?,
    )
    .try_into()
    .map_err(|_| RuntimeError::Deserialization)?;

    let len = u64::from_le_bytes(
        data[8..16]
            .try_into()
            .map_err(|_| RuntimeError::Deserialization)?,
    );
    let hash_seed: u32 = u64::from_le_bytes(
        data[16..24]
            .try_into()
            .map_err(|_| RuntimeError::Deserialization)?,
    )
    .try_into()
    .map_err(|_| RuntimeError::Deserialization)?;

    let ptr_buckets: MemoryAddress = u64::from_le_bytes(
        data[24..32]
            .try_into()
            .map_err(|_| RuntimeError::Deserialization)?,
    )
    .try_into()?;

    Ok(MapLayout {
        ptr_map_layout: address,
        bucket_size: MAP_BUCKET_SIZE + 8 * key_size + 8 * value_size,
        key_size,
        value_size,
        len,
        log_cap,
        hash_seed,
        ptr_buckets,
    })
}

pub fn top_hash(hash: u64) -> u8 {
    let mut top = (hash >> 48) as u8;
    if top <= TopHashValue::MIN as u8 {
        top += TopHashValue::MIN as u8;
    }
    return top;
}

pub fn bucket_idx(hash: u64, log_cap: u8) -> u64 {
    hash & ((1 << log_cap) - 1)
}

pub fn hash_of(bytes: &[u8], seed: u32) -> u64 {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    seed.hash(&mut hasher);
    hasher.finish()
}

fn gen_seed() -> u32 {
    let mut rng = rand::thread_rng();
    let seed: u32 = rng.gen();
    seed
}

fn retrieve_key(
    key_size: usize,
    ref_access: DerefHashing,
    stack: &mut Stack,
    heap: &mut Heap,
) -> Result<(Option<MemoryAddress>, Vec<u8>), RuntimeError> {
    match ref_access {
        DerefHashing::Vec(item_size) => {
            let address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;

            let len = OpPrimitive::get_num_from::<u64>(address.add(8), stack, heap)?;

            let items_bytes = heap.read(
                address.add(super::vector::VEC_HEADER),
                len as usize * item_size,
            )?;
            Ok((Some(address), items_bytes))
        }
        DerefHashing::String => {
            let address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;

            let len = OpPrimitive::get_num_from::<u64>(address.add(8), stack, heap)?;

            let items_bytes = heap.read(address.add(super::string::STRING_HEADER), len as usize)?;
            Ok((Some(address), items_bytes))
        }
        DerefHashing::Default => Ok((None, stack.pop(key_size)?.to_vec())),
        DerefHashing::StrSlice => {
            let address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;

            let len = OpPrimitive::get_num_from::<u64>(address, stack, heap)?;

            let items_bytes = heap.read(address.add(8), len as usize)?;
            Ok((Some(address), items_bytes))
        }
    }
}

impl<E: crate::vm::external::Engine> Executable<E> for MapAsm {
    fn execute<P: crate::vm::scheduler::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler::Scheduler<P>,
        signal_handler: &mut crate::vm::signal::SignalHandler<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::TID>,
    ) -> Result<(), RuntimeError> {
        match *self {
            MapAsm::Map {
                item_size,
                key_size,
            } => {
                let log_cap: u8 = 0;
                let map = MapLayout::new(key_size, item_size, log_cap);

                let address = map.init_in_mem(&stack, heap)?;
                let address: u64 = address.into(stack);
                stack.push_with(&address.to_le_bytes())?;
            }
            MapAsm::MapWithCapacity {
                item_size,
                key_size,
            } => {
                let mut log_cap: u8 = 0;
                let cap = OpPrimitive::pop_num::<u64>(stack)?;

                // to reduce cap size and therefore number of created bucket -> the map will try to fill up buckets in priority rather than reallocating
                if cap <= MAP_BUCKET_SIZE as u64 {
                    log_cap = 0;
                } else {
                    log_cap = ((cap as f64 / MAP_BUCKET_SIZE as f64).log2().ceil() as u64)
                        .try_into()
                        .map_err(|_| RuntimeError::Deserialization)?;
                }

                let map = MapLayout::new(key_size, item_size, log_cap);

                let address = map.init_in_mem(&stack, heap)?;
                let address: u64 = address.into(stack);
                stack.push_with(&address.to_le_bytes())?;
            }
            MapAsm::Insert {
                item_size,
                key_size,
                ref_access,
            } => {
                let item_data = stack.pop(item_size)?.to_owned();
                let (key_address, key_data) = retrieve_key(key_size, ref_access, stack, heap)?;

                let map_address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;

                let mut insertion_successful = false;

                while !insertion_successful {
                    let map_layout = map_layout(map_address, key_size, item_size, heap)?;

                    let hash = hash_of(&key_data, map_layout.hash_seed);
                    let top_hash = top_hash(hash);
                    let bucket_idx = bucket_idx(hash, map_layout.log_cap) as u64;

                    // get address of the bucket
                    let bucket_address = map_layout
                        .ptr_buckets
                        .add(bucket_idx as usize * map_layout.bucket_size);

                    let bucket_layout = bucket_layout(bucket_address, key_size, item_size, heap)?;

                    let opt_ptr_key_value =
                        bucket_layout.assign(top_hash, &key_data, ref_access, stack, heap)?;
                    match opt_ptr_key_value {
                        Some(AssignResult {
                            tophash_address,
                            key_address: assigned_key_address,
                            item_address,
                            is_new_value,
                        }) => {
                            // trigger resizing if overload
                            if is_new_value {
                                if over_load_factor(map_layout.len + 1, map_layout.log_cap) {
                                    let _ = map_layout.resize(stack, heap)?;
                                    // resizing invalidates everything so perform the whole operation again
                                    continue;
                                }
                            }
                            // insert in found place
                            let _ = heap.write(tophash_address, &vec![top_hash])?;
                            match key_address {
                                Some(key_address) => {
                                    let key_address: u64 = key_address.into(stack);

                                    let _ = heap
                                        .write(assigned_key_address, &key_address.to_le_bytes())?;
                                }
                                None => {
                                    let _ = heap.write(assigned_key_address, &key_data)?;
                                }
                            }
                            let _ = heap.write(item_address, &item_data)?;
                            if is_new_value {
                                // update len
                                let _ = heap.write(
                                    map_address.add(8),
                                    &(map_layout.len + 1).to_le_bytes().to_vec(),
                                )?;
                            }
                            insertion_successful = true;
                        }
                        None => {
                            // resize and retry
                            let _ = map_layout.resize(&stack, heap)?;
                        }
                    }

                    let map_address: u64 = map_address.into(stack);
                    let _ = stack.push_with(&map_address.to_le_bytes())?;
                }
            }
            MapAsm::DelKey {
                item_size,
                key_size,
                ref_access,
            } => {
                let (key_address, key_data) = retrieve_key(key_size, ref_access, stack, heap)?;

                let map_address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                let map_layout = map_layout(map_address, key_size, item_size, heap)?;

                let hash = hash_of(&key_data, map_layout.hash_seed);
                let top_hash = top_hash(hash);
                let bucket_idx = bucket_idx(hash, map_layout.log_cap) as u64;

                // get address of the bucket
                let bucket_address = map_layout
                    .ptr_buckets
                    .add(bucket_idx as usize * map_layout.bucket_size);

                let bucket_layout = bucket_layout(bucket_address, key_size, item_size, heap)?;

                match bucket_layout.delete(top_hash, &key_data, ref_access, stack, heap)? {
                    Some(item_address) => {
                        // update len
                        let _ =
                            heap.write(map_address, &(map_layout.len - 1).to_le_bytes().to_vec())?;
                        // read in found place
                        let value_data = heap.read(item_address, item_size)?;

                        let _ = stack.push_with(&value_data)?;
                        // push NO_ERROR
                        let _ = stack.push_with(&OK_SLICE)?;
                    }
                    None => {
                        let _ = stack.push_with(&vec![0u8; item_size])?;
                        // push ERROR
                        let _ = stack.push_with(&ERROR_SLICE)?;
                    }
                }
            }
            MapAsm::Get {
                item_size,
                key_size,
                ref_access,
            } => {
                let (key_address, key_data) = retrieve_key(key_size, ref_access, stack, heap)?;
                let map_address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                let map_layout = map_layout(map_address, key_size, item_size, heap)?;

                let hash = hash_of(&key_data, map_layout.hash_seed);
                let top_hash = top_hash(hash);
                let bucket_idx = bucket_idx(hash, map_layout.log_cap) as u64;

                // get address of the bucket
                let bucket_address = map_layout
                    .ptr_buckets
                    .add(bucket_idx as usize * map_layout.bucket_size);

                let bucket_layout = bucket_layout(bucket_address, key_size, item_size, heap)?;

                match bucket_layout.get(top_hash, &key_data, ref_access, stack, heap)? {
                    Some(item_address) => {
                        let value_data = heap.read(item_address, item_size)?;

                        let _ = stack.push_with(&value_data)?;
                        // push NO_ERROR
                        let _ = stack.push_with(&OK_SLICE)?;
                    }
                    None => {
                        let _ = stack.push_with(&vec![0u8; item_size])?;
                        // push ERROR
                        let _ = stack.push_with(&ERROR_SLICE)?;
                    }
                }
            }
            MapAsm::Clear {
                item_size,
                key_size,
            } => {
                let map_address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                let map_layout = map_layout(map_address, key_size, item_size, heap)?;
                let _ = map_layout.clear_buckets(heap)?;
                // update len
                let _ = heap.write(map_address.add(8), &(0u64).to_le_bytes().to_vec())?;
            }
        }
        scheduler.next();
        Ok(())
    }
}

use proc_macro_utils::*;

pub mod vm {
    pub mod casm {
        pub struct CasmProgram {}
        pub mod operation {
            use crate::vm::allocator::heap::Heap;
            use crate::vm::allocator::stack::Stack;
            use crate::vm::vm::RuntimeError;
            use num_traits::FromBytes;
            pub enum OpPrimitive {}

            impl OpPrimitive {
                pub fn get_num16<N: FromBytes<Bytes = [u8; 16]>>(
                    memory: &mut Stack,
                ) -> Result<N, RuntimeError> {
                    todo!()
                }
                pub fn get_num8<N: FromBytes<Bytes = [u8; 8]>>(
                    memory: &mut Stack,
                ) -> Result<N, RuntimeError> {
                    todo!()
                }
                pub fn get_num4<N: FromBytes<Bytes = [u8; 4]>>(
                    memory: &mut Stack,
                ) -> Result<N, RuntimeError> {
                    todo!()
                }
                pub fn get_num2<N: FromBytes<Bytes = [u8; 2]>>(
                    memory: &mut Stack,
                ) -> Result<N, RuntimeError> {
                    todo!()
                }
                pub fn get_num1<N: FromBytes<Bytes = [u8; 1]>>(
                    memory: &mut Stack,
                ) -> Result<N, RuntimeError> {
                    todo!()
                }

                pub fn get_bool(memory: &mut Stack) -> Result<bool, RuntimeError> {
                    todo!()
                }
                pub fn get_char(memory: &mut Stack) -> Result<char, RuntimeError> {
                    todo!()
                }
                pub fn get_str_slice(memory: &mut Stack) -> Result<String, RuntimeError> {
                    todo!()
                }
                pub fn get_string(
                    stack: &mut Stack,
                    heap: &mut Heap,
                ) -> Result<String, RuntimeError> {
                    todo!()
                }
            }
        }
    }
    pub mod allocator {
        pub mod stack {
            use crate::vm::vm::RuntimeError;
            pub struct Stack {}

            impl Stack {
                pub fn push_with(&mut self, data: &[u8]) -> Result<(), RuntimeError> {
                    todo!()
                }
            }
        }
        pub mod heap {
            pub struct Heap {}
        }
    }
    pub mod stdio {
        pub struct StdIO {}
        impl StdIO {
            pub fn push_casm_lib(&self, data: &str) {
                todo!()
            }
        }
    }
    pub mod vm {
        use crate::vm::allocator::heap::Heap;
        use crate::vm::allocator::stack::Stack;
        use crate::vm::casm::CasmProgram;
        use crate::vm::stdio::StdIO;
        pub enum RuntimeError {}

        pub trait Executable {
            fn execute(
                &self,
                program: &CasmProgram,
                stack: &mut Stack,
                heap: &mut Heap,
                stdio: &mut StdIO<G>,
            ) -> Result<(), RuntimeError>;
        }

        pub trait CasmMetadata {
            fn name(&self, stdio: &mut StdIO<G>, program: &CasmProgram, engine: &mut G);
            fn weight(&self) -> usize {
                1
            }
        }
    }
}

// #[shared_fn]
// fn func_cell(arg1: i32, arg2: String) -> u64 {
//     // Function body
//     8u64
// }

fn main() {}

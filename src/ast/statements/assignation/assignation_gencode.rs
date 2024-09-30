use crate::{
    ast::{expressions::locate::Locatable, statements::assignation::AssignValue},
    semantic::SizeOf,
    vm::{
        asm::{mem::Mem, Asm},
        CodeGenerationError, GenerateCode,
    },
};

use super::Assignation;

impl GenerateCode for Assignation {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        let _ = &self
            .right
            .gencode::<E>(scope_manager, scope_id, instructions, context)?;

        let Some(var_type) = self.left.signature() else {
            return Err(CodeGenerationError::UnresolvedError);
        };

        let var_size = var_type.size_of();

        if var_size == 0 {
            return Ok(());
        }
        match self.left.locate(scope_manager, scope_id, instructions)? {
            Some(address) => {
                instructions.push(Asm::Mem(Mem::Store {
                    size: var_size,
                    address,
                }));
            }
            None => {
                // The address was push on stack
                instructions.push(Asm::Mem(Mem::Take { size: var_size }));
            }
        }

        Ok(())
    }
}

impl GenerateCode for AssignValue {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        match self {
            AssignValue::Block(value) => {
                let _ = value.gencode::<E>(scope_manager, scope_id, instructions, context)?;
            }
            AssignValue::Expr(value) => {
                value.gencode::<E>(scope_manager, scope_id, instructions, context)?
            }
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {

    use crate::{
        test_extract_variable_with, test_statements,
        vm::{
            allocator::MemoryAddress,
            asm::operation::{GetNumFrom, OpPrimitive},
        },
    };

    #[test]
    fn valid_assignation() {
        let mut engine = crate::vm::external::test::NoopEngine {};

        fn assert_fn(
            scope_manager: &crate::semantic::scope::scope::ScopeManager,
            stack: &crate::vm::allocator::stack::Stack,
            heap: &crate::vm::allocator::heap::Heap,
        ) -> bool {
            test_extract_variable_with(
                "point",
                |address, stack, heap| {
                    let x = OpPrimitive::get_num_from::<u64>(address, stack, heap)
                        .expect("Deserialization should have succeeded");
                    let y = OpPrimitive::get_num_from::<u64>(address.add(8), stack, heap)
                        .expect("Deserialization should have succeeded");

                    assert_eq!(x, 5);
                    assert_eq!(y, 6);
                },
                scope_manager,
                stack,
                heap,
            );
            test_extract_variable_with(
                "arr",
                |address, stack, heap| {
                    let address: MemoryAddress =
                        OpPrimitive::get_num_from::<u64>(address, stack, heap)
                            .expect("Deserialization should have succeeded")
                            .try_into()
                            .unwrap();
                    let x = OpPrimitive::get_num_from::<u64>(address, stack, heap)
                        .expect("Deserialization should have succeeded");
                    let y = OpPrimitive::get_num_from::<u64>(address.add(8), stack, heap)
                        .expect("Deserialization should have succeeded");
                    let z = OpPrimitive::get_num_from::<u64>(address.add(16), stack, heap)
                        .expect("Deserialization should have succeeded");
                    let w = OpPrimitive::get_num_from::<u64>(address.add(24), stack, heap)
                        .expect("Deserialization should have succeeded");
                    assert_eq!(x, 5);
                    assert_eq!(y, 6);
                    assert_eq!(z, 7);
                    assert_eq!(w, 8);
                },
                scope_manager,
                stack,
                heap,
            );

            test_extract_variable_with(
                "tuple",
                |address, stack, heap| {
                    let x = OpPrimitive::get_num_from::<u64>(address, stack, heap)
                        .expect("Deserialization should have succeeded");
                    let y = OpPrimitive::get_num_from::<u64>(address.add(8), stack, heap)
                        .expect("Deserialization should have succeeded");
                    let z = OpPrimitive::get_num_from::<u64>(address.add(16), stack, heap)
                        .expect("Deserialization should have succeeded");
                    assert_eq!(x, 5);
                    assert_eq!(y, 6);
                    assert_eq!(z, 7);
                },
                scope_manager,
                stack,
                heap,
            );
            true
        }

        test_statements(
            r##"
        struct Point {
            x : i64,
            y : i64,
        }

        let point = Point { x:1, y:2 };

        point.x = 5;
        point.y = 6;

        let arr = [1,2,3,4];
        arr[0] = 5;
        arr[1] = 6;
        arr[2] = 7;
        arr[3] = 8;

        let tuple = (1,2,3);
        tuple.0 = 5;
        tuple.1 = 6;
        tuple.2 = 7;
        "##,
            &mut engine,
            assert_fn,
        );
    }

    #[test]
    fn valid_complex_assignation() {
        let mut engine = crate::vm::external::test::NoopEngine {};

        fn assert_fn(
            scope_manager: &crate::semantic::scope::scope::ScopeManager,
            stack: &crate::vm::allocator::stack::Stack,
            heap: &crate::vm::allocator::heap::Heap,
        ) -> bool {
            test_extract_variable_with(
                "t",
                |address, stack, heap| {
                    let address: MemoryAddress =
                        OpPrimitive::get_num_from::<u64>(address, stack, heap)
                            .expect("Deserialization should have succeeded")
                            .try_into()
                            .unwrap();
                    let res = OpPrimitive::get_num_from::<u64>(address.add(16), stack, heap)
                        .expect("Deserialization should have succeeded");
                    assert_eq!(res, 69);
                },
                scope_manager,
                stack,
                heap,
            );
            test_extract_variable_with(
                "arr",
                |address, stack, heap| {
                    let address: MemoryAddress =
                        OpPrimitive::get_num_from::<u64>(address, stack, heap)
                            .expect("Deserialization should have succeeded")
                            .try_into()
                            .unwrap();
                    let address = address.add(16);
                    let res = OpPrimitive::get_num_from::<u64>(address.add(16), stack, heap)
                        .expect("Deserialization should have succeeded");
                    assert_eq!(res, 69);
                },
                scope_manager,
                stack,
                heap,
            );
            true
        }

        test_statements(
            r##"
        struct Point {
            x :i64,
            y :i64,
            z :i64,
        }

        struct Test {
            tuple : ([4]i64,i64,Point)
        }

        let t = Test{
            tuple : ([1,2,3,4],2,Point{x:1,y:2,z:3})
        };
        
        t.tuple.0[2] = 69;

        let arr = vec[1,2,3,4];
        arr[2] = 69;

        "##,
            &mut engine,
            assert_fn,
        );
    }
}

use crate::ast::statements::assignation::AssignValue;
use crate::semantic::scope::scope::VariableInfo;
use crate::vm::{CodeGenerationError, GenerateCode};
use crate::{
    ast::statements::declaration::{DeclaredVar, PatternVar},
    semantic::SizeOf,
    vm::asm::{mem::Mem, Asm},
};

use super::{Declaration, TypedVar};

impl GenerateCode for Declaration {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        fn store_right_side<E: crate::vm::external::Engine>(
            left: &DeclaredVar,
            right: &AssignValue,
            scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
            scope_id: Option<u128>,
            instructions: &mut crate::vm::program::Program<E>,
            context: &crate::vm::CodeGenerationContext,
        ) -> Result<(), crate::vm::CodeGenerationError> {
            let Some(right_type) = (match right {
                AssignValue::Block(value) => value.metadata.signature(),
                AssignValue::Expr(value) => value.signature(),
            }) else {
                return Err(CodeGenerationError::UnresolvedError);
            };

            let _ = right.gencode::<E>(scope_manager, scope_id, instructions, context)?;

            match left {
                DeclaredVar::Id { id: Some(id), .. }
                | DeclaredVar::Typed(TypedVar { id: Some(id), .. }) => {
                    let Some(VariableInfo { address, .. }) = scope_manager.find_var_by_id(*id).ok()
                    else {
                        return Err(CodeGenerationError::UnresolvedError);
                    };
                    instructions.push(Asm::Mem(Mem::Store {
                        size: right_type.size_of(),
                        address: (*address)
                            .try_into()
                            .map_err(|_| CodeGenerationError::UnresolvedError)?,
                    }));
                }
                DeclaredVar::Pattern(PatternVar::Tuple { ids: Some(ids), .. })
                | DeclaredVar::Pattern(PatternVar::StructFields { ids: Some(ids), .. }) => {
                    for id in ids.iter().rev() {
                        let Some(VariableInfo { address, ctype, .. }) =
                            scope_manager.find_var_by_id(*id).ok()
                        else {
                            return Err(CodeGenerationError::UnresolvedError);
                        };
                        instructions.push(Asm::Mem(Mem::Store {
                            size: ctype.size_of(),
                            address: (*address)
                                .try_into()
                                .map_err(|_| CodeGenerationError::UnresolvedError)?,
                        }));
                    }
                }
                _ => {
                    return Err(CodeGenerationError::UnresolvedError);
                }
            }
            Ok(())
        }

        match self {
            Declaration::Declared(TypedVar { id, .. }) => {
                let Some(id) = id else {
                    return Err(CodeGenerationError::UnresolvedError);
                };

                if scope_manager.is_var_global(*id) {
                    let _ = scope_manager.alloc_global_var_by_id(*id)?;
                }
                Ok(())
            }
            Declaration::Assigned { left, right } => {
                // Alloc the variables
                match left {
                    DeclaredVar::Id { id: Some(id), .. }
                    | DeclaredVar::Typed(TypedVar { id: Some(id), .. }) => {
                        if scope_manager.is_var_global(*id) {
                            let _ = scope_manager.alloc_global_var_by_id(*id)?;
                        }
                    }
                    DeclaredVar::Pattern(PatternVar::StructFields { ids: Some(ids), .. })
                    | DeclaredVar::Pattern(PatternVar::Tuple { ids: Some(ids), .. }) => {
                        for id in ids {
                            if scope_manager.is_var_global(*id) {
                                let _ = scope_manager.alloc_global_var_by_id(*id)?;
                            }
                        }
                    }
                    _ => {
                        return Err(CodeGenerationError::UnresolvedError);
                    }
                }
                store_right_side(left, right, scope_manager, None, instructions, context)
            }
            Declaration::RecClosure {
                name,
                id,
                signature,
                right,
            } => {
                let Some(id) = id else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(right_type) = right.metadata.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                if scope_manager.is_var_global(*id) {
                    let _ = scope_manager.alloc_global_var_by_id(*id)?;
                }

                let _ = right.gencode::<E>(scope_manager, scope_id, instructions, context)?;
                let Some(VariableInfo { address, .. }) = scope_manager.find_var_by_id(*id).ok()
                else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                instructions.push(Asm::Mem(Mem::Store {
                    size: right_type.size_of(),
                    address: (*address)
                        .try_into()
                        .map_err(|_| CodeGenerationError::UnresolvedError)?,
                }));
                Ok(())
            }
            Declaration::RecLambda {
                name,
                id,
                signature,
                right,
            } => {
                let Some(id) = id else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                let Some(right_type) = right.metadata.signature() else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                if scope_manager.is_var_global(*id) {
                    let _ = scope_manager.alloc_global_var_by_id(*id)?;
                }

                let _ = right.gencode::<E>(scope_manager, scope_id, instructions, context)?;
                let Some(VariableInfo { address, .. }) = scope_manager.find_var_by_id(*id).ok()
                else {
                    return Err(CodeGenerationError::UnresolvedError);
                };
                instructions.push(Asm::Mem(Mem::Store {
                    size: right_type.size_of(),
                    address: (*address)
                        .try_into()
                        .map_err(|_| CodeGenerationError::UnresolvedError)?,
                }));
                Ok(())
            } // }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{test_extract_variable, test_statements};

    #[test]
    fn valid_declaration() {
        let mut engine = crate::vm::external::test::NoopEngine {};

        fn assert_fn(
            scope_manager: &crate::semantic::scope::scope::ScopeManager,
            stack: &crate::vm::allocator::stack::Stack,
            heap: &crate::vm::allocator::heap::Heap,
        ) -> bool {
            let res = test_extract_variable::<i64>("res1", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 1);
            let res = test_extract_variable::<u32>("res2", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 2);
            let res = test_extract_variable::<i64>("res3", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 3);
            let res = test_extract_variable::<u8>("res4", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 4);

            let res = test_extract_variable::<i64>("res5", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 5);

            let res = test_extract_variable::<i64>("res6", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 6);

            let res = test_extract_variable::<i64>("x", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 7);

            let res = test_extract_variable::<i64>("y", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 8);

            let res = test_extract_variable::<i64>("a", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 9);

            let res = test_extract_variable::<u32>("b", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 10);

            let res = test_extract_variable::<i64>("c", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 11);
            true
        }

        test_statements(
            r##"

        let res1 = 1;
        let res2:u32 = 2;
        let res3 = {
            let x = 1;
            x + 2
        };
        let res4 : u8 = {
            let x = 3u8;
            x + 1
        };

        let (res5,res6) = (5,6);

        struct Point {
            x : i64,
            y : i64,
        }

        let Point {x,y} = Point {x:7,y:8};

        struct Test {
            a : i64,
            b : u32,
            c : i64,
        }
        
        let Test {a,b,c} = Test {a:9,b:10,c:11};
        
        "##,
            &mut engine,
            assert_fn,
        );
    }

    #[test]
    fn valid_rec_functions() {
        let mut engine = crate::vm::external::test::NoopEngine {};

        fn assert_fn(
            scope_manager: &crate::semantic::scope::scope::ScopeManager,
            stack: &crate::vm::allocator::stack::Stack,
            heap: &crate::vm::allocator::heap::Heap,
        ) -> bool {
            let res = test_extract_variable::<i64>("res1", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 55);

            let res = test_extract_variable::<i64>("res2", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 15);

            let res = test_extract_variable::<i64>("res3", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 15);
            true
        }

        test_statements(
            r##"
        fn fibonacci(x:u64) -> u64 {
            if x == 0u64 {
                return 0;
            } else if x == 1u64 or x == 2u64 {
                return 1;
            }
            return fibonacci(x-1) + fibonacci(x-2);
        }
        let res1 = fibonacci(10);

        let rec lambda1 : (u64) -> u64 = (x) -> {
            if x == 0 {
                return 0;
            }
            return x + lambda1(x - 1);
        };

        let res2 = lambda1(5);

        let rec closure1 : (u64) -> u64 = (x) -> {
            if x == 0 {
                return 0;
            }
            return x + closure1(x - 1);
        };

        let res3 = closure1(5);
        "##,
            &mut engine,
            assert_fn,
        );
    }
}

use super::{Definition, FnDef, TypeDef};
use crate::semantic::scope::scope::{ScopeManager, Variable, VariableInfo};
use crate::semantic::SizeOf;
use crate::vm::allocator::MemoryAddress;
use crate::vm::vm::CodeGenerationContext;

use crate::vm::casm::alloc::Alloc;
use crate::vm::casm::locate::Locate;

use crate::vm::{
    casm::{
        branch::{Goto, Label},
        mem::Mem,
        Casm, CasmProgram,
    },
    vm::{CodeGenerationError, GenerateCode},
};

impl GenerateCode for Definition {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Definition::Type(value) => {
                value.gencode(scope_manager, scope_id, instructions, context)
            }
            Definition::Fn(value) => value.gencode(scope_manager, scope_id, instructions, context),
        }
    }
}

impl GenerateCode for TypeDef {
    fn gencode(
        &self,
        _scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        _instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        Ok(())
    }
}

impl GenerateCode for FnDef {
    fn gencode(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut CasmProgram,
        context: &crate::vm::vm::CodeGenerationContext,
    ) -> Result<(), CodeGenerationError> {
        let function_label = Label::gen();
        let store_label = Label::gen();

        instructions.push(Casm::Goto(Goto {
            label: Some(store_label),
        }));
        instructions.push_label_id(function_label, format!("fn_{0}", self.name));
        self.scope
            .gencode(scope_manager, scope_id, instructions, context)?;

        instructions.push_label_id(store_label, format!("store_fn_{0}", self.name));

        instructions.push(Casm::Mem(Mem::Label(function_label)));

        if let Some(scope_id) = scope_id {
            // LOCAL FUNCTION
            // store the function label as it is considered a variable
            let Some(id) = self.id else {
                return Err(CodeGenerationError::UnresolvedError);
            };
            let Ok(VariableInfo { address, .. }) = scope_manager.find_var_by_id(id) else {
                return Err(CodeGenerationError::UnresolvedError);
            };
            instructions.push(Casm::Mem(Mem::Store {
                size: 8,
                address: (*address)
                    .try_into()
                    .map_err(|_| CodeGenerationError::UnresolvedError)?,
            }));
        } else {
            // GLOBAL FUNCTION
            // allocate the function
            let Some(id) = self.id else {
                return Err(CodeGenerationError::UnresolvedError);
            };
            // store the function label as it is considered a variable
            let address = scope_manager.alloc_global_var_by_id(id)?;
            instructions.push(Casm::Mem(Mem::Store {
                size: 8,
                address: (address)
                    .try_into()
                    .map_err(|_| CodeGenerationError::UnresolvedError)?,
            }));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use crate::ast::TryParse;
    use crate::semantic::scope::static_types::{NumberType, PrimitiveType};
    use crate::semantic::Resolve;
    use crate::vm::vm::Runtime;
    use crate::{ast::statements::Statement, semantic::scope::scope::ScopeManager};
    use crate::{clear_stack, test_extract_variable, test_statements, v_num};

    #[test]
    fn valid_fn() {
        let mut engine = crate::vm::vm::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
            stack: &mut crate::vm::allocator::stack::Stack,
            heap: &mut crate::vm::allocator::heap::Heap,
        ) -> bool {
            let res = test_extract_variable::<i64>("res2", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 5);
            let res = test_extract_variable::<i64>("res3", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 5);
            let res = test_extract_variable::<i64>("res5", scope_manager, stack, heap)
                .expect("Deserialization should have succeeded");
            assert_eq!(res, 5);
            true
        }

        test_statements(
            r##"
        fn test1() {
            let x = 5;
        }

        test1();

        fn test2() -> i64 {
            let x = 5;
            return x;
        }

        let res2 = test2();

        fn test3(x:i64) -> i64 {
            return x + 1;
        }
        let res3 = test3(4);

        fn test4(x:i64) {
            let y = x;
        }
        test4(5);

        fn test5(x:i64,y:i64) -> i64 {
            let z = x + y;
            return z;
        }
        let res5 = test5(2,3);

        "##,
            &mut engine,
            assert_fn,
        );
    }

    // #[test]
    // fn valid_function() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = {
    //         fn f(x:u64) -> u64 {
    //             return x+1;
    //         }
    //         return f(68);
    //     };

    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let data = compile_statement!(statement);

    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::U64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(U64, 69));
    // }

    // #[test]
    // fn valid_function_general() {
    //     let mut statement = Statement::parse(
    //         r##"
    //         fn f(x:u64) -> u64 {
    //             return x+1;
    //         }
    //     };

    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let data = compile_statement!(statement);
    //     assert!(data.len() != 0);
    // }

    // #[test]
    // fn valid_function_with_stack_env() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = {
    //         let env:u64 = 31;
    //         fn f(x:u64) -> u64 {
    //             if true {
    //                 return x + env ;
    //             }else {
    //                 return env + x;
    //             }
    //         }
    //         env = 50;
    //         return f(38);
    //     };

    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let data = compile_statement!(statement);

    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::U64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(U64, 88));
    // }

    // #[test]
    // fn valid_function_with_heap_env() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = {
    //         let env : Vec<u64> = vec[2,5];

    //         fn f(x:u64) -> u64 {
    //             return env[1] + x;
    //         }
    //         env[1] = 31;
    //         return f(38);
    //     };

    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let data = compile_statement!(statement);

    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::U64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(U64, 69));
    // }

    // #[test]
    // fn valid_function_rec() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = {
    //         fn recursive(x:u64) -> u64 {
    //             if x == 0u64 {
    //                 return 0;
    //             }
    //             return 1u64 + recursive(x-1);
    //         }
    //         return recursive(3);
    //     };

    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
    //     let _ = statement
    //         .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
    //         .expect("Semantic resolution should have succeeded");

    //     // Code generation.
    //     let mut instructions = CasmProgram::default();
    //     statement
    //         .gencode(
    //             &mut scope_manager,
    //             None,
    //             &mut instructions,
    //             &crate::vm::vm::CodeGenerationContext::default(),
    //         )
    //         .expect("Code generation should have succeeded");

    //     assert!(instructions.len() > 0);

    //     let (mut runtime, mut heap, mut stdio) = Runtime::new();
    //     let tid = runtime
    //         .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
    //         .expect("Thread spawn_with_scopeing should have succeeded");
    //     let (_, stack, program) = runtime
    //         .get_mut(crate::vm::vm::Player::P1, tid)
    //         .expect("Thread should exist");
    //     program.merge(instructions);
    //     let mut engine = crate::vm::vm::NoopGameEngine {};

    //     program
    //         .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
    //         .expect("Execution should have succeeded");
    //     let memory = stack;
    //     let data = clear_stack!(memory);

    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::U64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(U64, 3));
    // }

    // #[test]
    // fn valid_function_fibonacci() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = {
    //         fn fibonacci(x:u64) -> u64 {
    //             if x == 0u64 {
    //                 return 0;
    //             } else if x == 1u64 or x == 2u64 {
    //                 return 1;
    //             }
    //             return fibonacci(x-1) + fibonacci(x-2);
    //         }
    //         return fibonacci(10);
    //     };

    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let mut scope_manager = crate::semantic::scope::scope::ScopeManager::default();
    //     let _ = statement
    //         .resolve::<crate::vm::vm::NoopGameEngine>(&mut scope_manager, None, &None, &mut ())
    //         .expect("Semantic resolution should have succeeded");

    //     // Code generation.
    //     let mut instructions = CasmProgram::default();
    //     statement
    //         .gencode(
    //             &mut scope_manager,
    //             None,
    //             &mut instructions,
    //             &crate::vm::vm::CodeGenerationContext::default(),
    //         )
    //         .expect("Code generation should have succeeded");

    //     assert!(instructions.len() > 0);

    //     let (mut runtime, mut heap, mut stdio) = Runtime::new();
    //     let tid = runtime
    //         .spawn_with_scope(crate::vm::vm::Player::P1, scope_manager)
    //         .expect("Thread spawn_with_scopeing should have succeeded");
    //     let (_, stack, program) = runtime
    //         .get_mut(crate::vm::vm::Player::P1, tid)
    //         .expect("Thread should exist");
    //     program.merge(instructions);
    //     let mut engine = crate::vm::vm::NoopGameEngine {};

    //     program
    //         .execute(stack, &mut heap, &mut stdio, &mut engine, tid)
    //         .expect("Execution should have succeeded");
    //     let memory = stack;
    //     let data = clear_stack!(memory);
    //     let engine = crate::vm::vm::NoopGameEngine {};

    //     let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
    //         &PrimitiveType::Number(NumberType::U64),
    //         &data,
    //     )
    //     .expect("Deserialization should have succeeded");
    //     assert_eq!(result, v_num!(U64, 55));
    // }
}

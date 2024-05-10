use super::{Definition, FnDef, TypeDef};
use crate::semantic::scope::scope::Scope;
use crate::semantic::SizeOf;

use crate::vm::allocator::stack::Offset;
use crate::vm::allocator::MemoryAddress;

use crate::vm::casm::alloc::Alloc;
use crate::vm::casm::locate::Locate;

use crate::{
    semantic::MutRc,
    vm::{
        casm::{
            branch::{Goto, Label},
            mem::Mem,
            Casm, CasmProgram,
        },
        vm::{CodeGenerationError, GenerateCode},
    },
};

impl GenerateCode for Definition {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Definition::Type(value) => value.gencode(scope, instructions),
            Definition::Fn(value) => value.gencode(scope, instructions),
        }
    }
}

impl GenerateCode for TypeDef {
    fn gencode(
        &self,
        _scope: &MutRc<Scope>,
        _instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        Ok(())
    }
}

impl GenerateCode for FnDef {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        let end_closure = Label::gen();

        // If the scope is the general scope, update the address, the offset and the scope stack top
        if let Some(stack_top) = scope.borrow().stack_top() {
            let borrow = scope.as_ref().borrow();
            let mut size = 8;
            for (v, o) in borrow.vars() {
                if **v.id == *self.id {
                    o.set(Offset::SB(stack_top));
                    size = v.type_sig.size_of();
                    break;
                }
            }

            instructions.push(Casm::Alloc(Alloc::Stack { size }));
            let _ = scope
                .borrow()
                .update_stack_top(stack_top + size)
                .map_err(|_| CodeGenerationError::UnresolvedError)?;
        }

        instructions.push(Casm::Goto(Goto {
            label: Some(end_closure),
        }));

        let closure_label = instructions.push_label(format!("fn_{}", self.id).into());
        let _ = self.scope.gencode(scope, instructions);
        instructions.push_label_id(end_closure, "end_closure".to_string().into());

        instructions.push(Casm::Mem(Mem::LabelOffset(closure_label)));

        let (var, address, level) = scope.as_ref().borrow().access_var(&self.id)?;
        let var_type = &var.as_ref().type_sig;

        let var_size = var_type.size_of();

        instructions.push(Casm::Locate(Locate {
            address: MemoryAddress::Stack {
                offset: address,
                level,
            },
        }));
        instructions.push(Casm::Mem(Mem::TakeToStack { size: var_size }));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;
    use crate::ast::expressions::data::{Number, Primitive};
    use crate::ast::TryParse;
    use crate::semantic::scope::static_types::{NumberType, PrimitiveType};
    use crate::semantic::Resolve;
    use crate::vm::vm::{DeserializeFrom, Runtime};
    use crate::{ast::statements::Statement, semantic::scope::scope::Scope};
    use crate::{clear_stack, compile_statement, v_num};

    #[test]
    fn valid_function() {
        let statement = Statement::parse(
            r##"
        let x = {
            fn f(x:u64) -> u64 {
                return x+1;
            }
            return f(68); 
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let data = compile_statement!(statement);

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 69));
    }

    #[test]
    fn valid_function_general() {
        let statement = Statement::parse(
            r##"
            fn f(x:u64) -> u64 {
                return x+1;
            }
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let data = compile_statement!(statement);
        assert!(data.len() != 0);
    }

    #[test]
    fn valid_function_with_stack_env() {
        let statement = Statement::parse(
            r##"
        let x = {
            let env:u64 = 31;
            fn f(x:u64) -> u64 {
                if true {
                    return x + env ;
                }else {
                    return env + x;
                }
            }
            env = 50;
            return f(38); 
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let data = compile_statement!(statement);

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 88));
    }

    #[test]
    fn valid_function_with_heap_env() {
        let statement = Statement::parse(
            r##"
        let x = {
            let env : Vec<u64> = vec[2,5];

            fn f(x:u64) -> u64 {
                return env[1] + x;
            }
            env[1] = 31;
            return f(38); 
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let data = compile_statement!(statement);

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 69));
    }

    #[test]
    fn valid_function_rec() {
        let statement = Statement::parse(
            r##"
        let x = {
            fn recursive(x:u64) -> u64 {
                if x == 0u64 {
                    return 0;
                }
                return 1u64 + recursive(x-1);
            }
            return recursive(3); 
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        // dbg!(&instructions);
        assert!(instructions.len() > 0);

        let (mut runtime, mut heap, mut stdio) = Runtime::<crate::vm::vm::NoopGameEngine>::new();
        let tid = runtime
            .spawn_with_scope(scope)
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, mut stack, mut program) = runtime.get_mut(tid).expect("Thread should exist");
        program.merge(instructions);
        let mut engine = crate::vm::vm::NoopGameEngine {};

        program
            .execute(stack, &mut heap, &mut stdio, &mut engine)
            .expect("Execution should have succeeded");
        let memory = stack;
        let data = clear_stack!(memory);

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 3));
    }

    #[test]
    fn valid_function_fibonacci() {
        let statement = Statement::parse(
            r##"
        let x = {
            fn fibonacci(x:u64) -> u64 {
                if x == 0u64 {
                    return 0;
                } else if x == 1u64 or x == 2u64 {
                    return 1;
                }
                return fibonacci(x-1) + fibonacci(x-2);
            }
            return fibonacci(10);
        };

        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;

        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        // dbg!(&instructions);
        assert!(instructions.len() > 0);

        let (mut runtime, mut heap, mut stdio) = Runtime::<crate::vm::vm::NoopGameEngine>::new();
        let tid = runtime
            .spawn_with_scope(scope)
            .expect("Thread spawn_with_scopeing should have succeeded");
        let (_, mut stack, mut program) = runtime.get_mut(tid).expect("Thread should exist");
        program.merge(instructions);
        let mut engine = crate::vm::vm::NoopGameEngine {};

        program
            .execute(stack, &mut heap, &mut stdio, &mut engine)
            .expect("Execution should have succeeded");
        let memory = stack;
        let data = clear_stack!(memory);
        let mut engine = crate::vm::vm::NoopGameEngine {};

        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(U64, 55));
    }
}

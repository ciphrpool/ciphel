use crate::ast::utils::lexem;
use crate::semantic::scope::scope::Scope;
use crate::{arw_read, arw_write};
use crate::{
    ast::statements::declaration::{DeclaredVar, PatternVar},
    semantic::{ArcMutex, SizeOf},
    vm::{
        allocator::{stack::Offset, MemoryAddress},
        casm::{alloc::Alloc, locate::Locate, mem::Mem, Casm, CasmProgram},
        vm::{CodeGenerationError, GenerateCode},
    },
};

use super::{Declaration, TypedVar};

impl GenerateCode for Declaration {
    fn gencode(
        &self,
        scope: &crate::semantic::ArcRwLock<Scope>,
        instructions: &CasmProgram,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Declaration::Declared(TypedVar { id, .. }) => {
                // When the variable is created in the general block,
                // the block can't assign a stackpointer to the variable
                // therefore the variable have to live at the current offset
                let borrow = crate::arw_read!(scope, CodeGenerationError::ConcurrencyError)?;
                let mut new_stack_top = borrow.stack_top();
                for (v, o) in borrow.vars() {
                    if v.id == *id && !v.is_declared.get() {
                        let var = v;

                        var.is_declared.set(true);

                        if let Some(ref mut stack_top) = new_stack_top {
                            o.set(Offset::SB(*stack_top));
                            if var.type_sig.size_of() == 0 {
                                continue;
                            }
                            instructions.push(Casm::Alloc(Alloc::Stack {
                                size: var.type_sig.size_of(),
                            }));
                            *stack_top = *stack_top + var.type_sig.size_of();
                        }
                        break;
                    }
                }
                drop(borrow);
                if let Some(stack_top) = new_stack_top {
                    let _ = arw_write!(scope, CodeGenerationError::ConcurrencyError)?
                        .update_stack_top(stack_top)
                        .map_err(|_| CodeGenerationError::UnresolvedError)?;
                }

                // let (address, level) = block
                //     .borrow()
                //     .address_of(id)
                //     .map_err(|_| CodeGenerationError::UnresolvedError)?;

                Ok(())
                // match address.get() {
                //     Some(_) => Ok(()),
                //     None => {
                //         // Update the stack pointer of the variable
                //         var.as_ref().address.set(Some(Offset::FZ(0)));
                //         let var_size = var.type_sig.size_of();
                //         instructions.push(Casm::Alloc(Alloc::Stack { size: var_size }));
                //         Ok(())
                //     }
                // }
            }
            Declaration::Assigned { left, right } => {
                // retrieve all named variables and alloc them if needed
                let mut vars = match left {
                    DeclaredVar::Id(id) => vec![id.clone()],
                    DeclaredVar::Typed(TypedVar { id, .. }) => vec![id.clone()],
                    // DeclaredVar::Pattern(PatternVar::UnionFields { vars, .. }) => vars.to_vec(),
                    DeclaredVar::Pattern(PatternVar::StructFields { vars, .. }) => vars.to_vec(),
                    DeclaredVar::Pattern(PatternVar::Tuple(ids)) => ids.to_vec(),
                };

                for id in &vars {
                    let borrow = crate::arw_read!(scope, CodeGenerationError::ConcurrencyError)?;
                    for (v, o) in borrow.vars() {
                        if v.id == *id && !v.is_declared.get() {
                            let var = v;
                            let var_size = var.type_sig.size_of();
                            if var_size == 0 {
                                continue;
                            }
                            if let Some(stack_top) =
                                arw_read!(scope, CodeGenerationError::ConcurrencyError)?.stack_top()
                            {
                                o.set(Offset::SB(stack_top));
                                instructions.push(Casm::Alloc(Alloc::Stack { size: var_size }));
                                let _ = arw_read!(scope, CodeGenerationError::ConcurrencyError)?
                                    .update_stack_top(stack_top + var_size)?;
                            }
                            break;
                        }
                    }
                }

                // Generate right side code
                let _ = right.gencode(scope, instructions)?;

                for id in &vars {
                    let borrow = crate::arw_read!(scope, CodeGenerationError::ConcurrencyError)?;
                    for (v, _o) in borrow.vars() {
                        if v.id == *id && !v.is_declared.get() {
                            v.is_declared.set(true);
                            break;
                        }
                    }
                }

                // Generate the left side code : the variable declaration
                // reverse the variables in order to pop the stack and assign in order of stack push
                vars.reverse();

                for id in &vars {
                    let (var, address, level) =
                        crate::arw_read!(scope, CodeGenerationError::ConcurrencyError)?
                            .access_var(id)?;

                    let var_size = var.type_sig.size_of();
                    if var_size == 0 {
                        continue;
                    }
                    if **id != lexem::UNDERSCORE {
                        instructions.push(Casm::Locate(Locate {
                            address: MemoryAddress::Stack {
                                offset: address,
                                level,
                            },
                        }));
                        instructions.push(Casm::Mem(Mem::TakeToStack { size: var_size }))
                    } else {
                        instructions.push(Casm::Pop(var_size));
                    }
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use num_traits::Zero;

    use crate::{
        ast::{
            expressions::data::{Number, Primitive},
            statements::Statement,
            TryParse,
        },
        clear_stack, compile_statement, p_num,
        semantic::{
            scope::{
                scope::Scope,
                static_types::{NumberType, PrimitiveType, StaticType},
                user_type_impl::{self, UserType},
            },
            Either, Resolve,
        },
        v_num,
        vm::{
            allocator::Memory,
            casm::CasmProgram,
            vm::{DeserializeFrom, Executable, Runtime},
        },
    };

    use super::*;

    #[test]
    fn valid_declaration_inplace_in_scope() {
        let mut statement = Statement::parse(
            r##"
        let x = {
            let x:u64 = 420;
            return x;
        };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(I64, 420));
    }

    #[test]
    fn valid_declaration_underscore() {
        let mut statement = Statement::parse(
            r##"
        let x = {
            let _ = 420;
            return 69;
        };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let result = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, v_num!(I64, 69));
    }

    #[test]
    fn valid_declaration_inplace_tuple_in_scope() {
        let mut statement = Statement::parse(
            r##"
        let (x,y) = {
            let (x,y) = (420,69);
            return (x,y);
        };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let x = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data[0..8],
        )
        .expect("Deserialization should have succeeded");
        let y = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data[8..16],
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(x, v_num!(I64, 420));
        assert_eq!(y, v_num!(I64, 69));
    }

    #[test]
    fn valid_declaration_tuple_underscore() {
        let mut statement = Statement::parse(
            r##"
        let (x,y) = {
            let (x,_) = (420,69);
            return (x,70);
        };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let x = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data[0..8],
        )
        .expect("Deserialization should have succeeded");
        let y = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data[8..16],
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(x, v_num!(I64, 420));
        assert_eq!(y, v_num!(I64, 70));
    }

    #[test]
    fn valid_declaration_inplace_tuple_general_scope() {
        let mut statement = Statement::parse(
            r##"
            let (x,y) = (420,69);
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let data = compile_statement!(statement);
        let x = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data[0..8],
        )
        .expect("Deserialization should have succeeded");
        let y = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data[8..16],
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(x, v_num!(I64, 420));
        assert_eq!(y, v_num!(I64, 69));
    }

    #[test]
    fn valid_declaration_inplace_struct_in_scope() {
        let user_type = user_type_impl::Struct {
            id: "Point".to_string().into(),
            fields: {
                let mut res = Vec::new();
                res.push(("x".to_string().into(), p_num!(I64)));
                res.push(("y".to_string().into(), p_num!(I64)));
                res
            },
        };
        let mut statement = Statement::parse(
            r##"
        let (x,y) = {
            let Point {x,y} = Point {
                x : 420,
                y : 69,
            };
            return (x,y);
        };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = crate::arw_write!(scope, CodeGenerationError::ConcurrencyError)
            .unwrap()
            .register_type(&"Point".to_string().into(), UserType::Struct(user_type))
            .expect("Registering of user type should have succeeded");
        let _ = statement
            .resolve(&scope, &None, &mut ())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0);
        // Execute the instructions.

        let (mut runtime, mut heap, mut stdio) = Runtime::new();
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
        let x = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data[0..8],
        )
        .expect("Deserialization should have succeeded");
        let y = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data[8..16],
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(x, v_num!(I64, 420));
        assert_eq!(y, v_num!(I64, 69));
    }

    #[test]
    fn valid_declaration_inplace_struct_general_scope() {
        let user_type = user_type_impl::Struct {
            id: "Point".to_string().into(),
            fields: {
                let mut res = Vec::new();
                res.push(("x".to_string().into(), p_num!(I64)));
                res.push(("y".to_string().into(), p_num!(I64)));
                res
            },
        };
        let mut statement = Statement::parse(
            r##"
            let Point {x,y} = Point {
                x : 420,
                y : 69,
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();
        let _ = crate::arw_write!(scope, CodeGenerationError::ConcurrencyError)
            .unwrap()
            .register_type(&"Point".to_string().into(), UserType::Struct(user_type))
            .expect("Registering of user type should have succeeded");
        let _ = statement
            .resolve(&scope, &None, &mut ())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0);
        // Execute the instructions.

        let (mut runtime, mut heap, mut stdio) = Runtime::new();
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

        let x = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data[0..8],
        )
        .expect("Deserialization should have succeeded");
        let y = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data[8..16],
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(x, v_num!(I64, 420));
        assert_eq!(y, v_num!(I64, 69));
    }

    #[test]
    fn valid_shadowing_same_type() {
        let mut statement = Statement::parse(
            r##"
            let x = {
                let var = 5;
                var = 6;
                let var = var + 4;
                return var + 10; 
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();

        let _ = statement
            .resolve(&scope, &None, &mut ())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0);
        // Execute the instructions.

        let (mut runtime, mut heap, mut stdio) = Runtime::new();
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

        let value = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data[0..8],
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(value, v_num!(I64, 20));
    }

    #[test]
    fn valid_shadowing_different_type() {
        let mut statement = Statement::parse(
            r##"
            let x = {
                let var = 5u8;
                var = 6u8;
                let var = var as i64 + 4;
                return var + 10; 
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();

        let _ = statement
            .resolve(&scope, &None, &mut ())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0);
        // Execute the instructions.

        let (mut runtime, mut heap, mut stdio) = Runtime::new();
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

        let value = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data[0..8],
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(value, v_num!(I64, 20));
    }

    #[test]
    fn valid_shadowing_outer_scope() {
        let mut statement = Statement::parse(
            r##"
            let x = {
                let var = 5u8;
                let outer = {
                    let var = 10;
                    return var;
                };
                return outer + 10; 
            };
        "##
            .into(),
        )
        .expect("Parsing should have succeeded")
        .1;
        let scope = Scope::new();

        let _ = statement
            .resolve(&scope, &None, &mut ())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = CasmProgram::default();
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        assert!(instructions.len() > 0);
        // Execute the instructions.

        let (mut runtime, mut heap, mut stdio) = Runtime::new();
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

        let value = <PrimitiveType as DeserializeFrom>::deserialize_from(
            &PrimitiveType::Number(NumberType::I64),
            &data[0..8],
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(value, v_num!(I64, 20));
    }
}

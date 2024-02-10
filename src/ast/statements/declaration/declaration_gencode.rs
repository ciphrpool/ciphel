use std::{cell::RefCell, rc::Rc};

use crate::{
    ast::statements::{
        assignation::AssignValue,
        declaration::{DeclaredVar, PatternVar},
    },
    semantic::{scope::ScopeApi, MutRc, SizeOf},
    vm::{
        allocator::{stack::Offset, MemoryAddress},
        casm::{alloc::Alloc, locate::Locate, memcopy::MemCopy, Casm, CasmProgram},
        vm::{CodeGenerationError, GenerateCode},
    },
};

use super::{Declaration, TypedVar};

impl<Scope: ScopeApi> GenerateCode<Scope> for Declaration<Scope> {
    fn gencode(
        &self,
        scope: &MutRc<Scope>,
        instructions: &MutRc<CasmProgram>,
    ) -> Result<(), CodeGenerationError> {
        match self {
            Declaration::Declared(TypedVar { id, .. }) => {
                // When the variable is created in the general scope,
                // the scope can't assign a stackpointer to the variable
                // therefore the variable have to live at the current offset
                let var = scope
                    .borrow()
                    .find_var(id)
                    .map_err(|_| CodeGenerationError::UnresolvedError)?;

                let address = &var.as_ref().address;
                match address.get() {
                    Some(_) => Ok(()),
                    None => {
                        // Update the stack pointer of the variable
                        var.as_ref().address.set(Some(Offset::FZ(0)));
                        let var_size = var.type_sig.size_of();
                        let mut borrowed_instructions = instructions
                            .as_ref()
                            .try_borrow_mut()
                            .map_err(|_| CodeGenerationError::Default)?;
                        borrowed_instructions.push(Casm::Alloc(Alloc::Stack { size: var_size }));
                        Ok(())
                    }
                }
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
                let mut borrowed_instructions = instructions
                    .as_ref()
                    .try_borrow_mut()
                    .map_err(|_| CodeGenerationError::Default)?;
                let mut var_offset_idx = 0;
                for id in &vars {
                    let var = scope
                        .borrow()
                        .find_var(&id)
                        .map_err(|_| CodeGenerationError::UnresolvedError)?;
                    let address = &var.as_ref().address;
                    let var_size = var.type_sig.size_of();
                    let address = match address.get() {
                        Some(addr) => addr,
                        None => {
                            // Update the stack pointer of the variable
                            var.as_ref().address.set(Some(Offset::FZ(var_offset_idx)));
                            borrowed_instructions
                                .push(Casm::Alloc(Alloc::Stack { size: var_size }));
                            Offset::FZ(var_offset_idx)
                        }
                    };
                    var_offset_idx += var_size as isize;
                }
                drop(borrowed_instructions);

                // Generate right side code
                let _ = right.gencode(scope, instructions)?;

                // Generate the left side code : the variable declaration
                // reverse the variables in order to pop the stack and assign in order of stack push
                vars.reverse();
                let mut borrowed_instructions = instructions
                    .as_ref()
                    .try_borrow_mut()
                    .map_err(|_| CodeGenerationError::Default)?;
                for id in vars {
                    let var = scope
                        .borrow()
                        .find_var(&id)
                        .map_err(|_| CodeGenerationError::UnresolvedError)?;
                    let Some(address) = var.as_ref().address.get() else {
                        return Err(CodeGenerationError::UnresolvedError);
                    };
                    let var_size = var.type_sig.size_of();
                    borrowed_instructions.push(Casm::Locate(Locate {
                        address: MemoryAddress::Stack { offset: address },
                    }));
                    borrowed_instructions
                        .push(Casm::MemCopy(MemCopy::TakeToStack { size: var_size }))
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use num_traits::Zero;

    use crate::{
        ast::{
            expressions::data::{Number, Primitive},
            statements::Statement,
            TryParse,
        },
        clear_stack,
        semantic::{
            scope::{
                scope_impl::Scope,
                static_types::{NumberType, PrimitiveType, StaticType},
                user_type_impl::{self, UserType},
            },
            Either, Resolve,
        },
        vm::{
            allocator::Memory,
            casm::CasmProgram,
            vm::{DeserializeFrom, Executable},
        },
    };

    use super::*;

    #[test]
    fn valid_declaration_inplace_in_scope() {
        let statement = Statement::parse(
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
        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = Rc::new(RefCell::new(CasmProgram::default()));
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        let instructions = instructions.as_ref().take();
        assert!(instructions.len() > 0);
        // Execute the instructions.
        let memory = Memory::new();
        instructions
            .execute(&memory)
            .expect("Execution should have succeeded");

        let data = clear_stack!(memory);
        let result = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data,
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(result, Primitive::Number(Number::U64(420)));
    }

    #[test]
    fn valid_declaration_inplace_tuple_in_scope() {
        let statement = Statement::parse(
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
        let scope = Scope::new();
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = Rc::new(RefCell::new(CasmProgram::default()));
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        let instructions = instructions.as_ref().take();
        assert!(instructions.len() > 0);
        // Execute the instructions.
        let memory = Memory::new();
        instructions
            .execute(&memory)
            .expect("Execution should have succeeded");
        let data = clear_stack!(memory);
        let x = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data[0..8],
        )
        .expect("Deserialization should have succeeded");
        let y = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data[8..16],
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(x, Primitive::Number(Number::U64(420)));
        assert_eq!(y, Primitive::Number(Number::U64(69)));
    }

    #[test]
    fn valid_declaration_inplace_tuple_general_scope() {
        let statement = Statement::parse(
            r##"
            let (x,y) = (420,69);
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
        let instructions = Rc::new(RefCell::new(CasmProgram::default()));
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        let instructions = instructions.as_ref().take();
        assert!(instructions.len() > 0);
        // Execute the instructions.
        let memory = Memory::new();
        instructions
            .execute(&memory)
            .expect("Execution should have succeeded");
        let data = clear_stack!(memory);
        let x = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data[0..8],
        )
        .expect("Deserialization should have succeeded");
        let y = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data[8..16],
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(x, Primitive::Number(Number::U64(420)));
        assert_eq!(y, Primitive::Number(Number::U64(69)));
    }

    #[test]
    fn valid_declaration_inplace_struct_in_scope() {
        let user_type = user_type_impl::Struct {
            id: "Point".into(),
            fields: {
                let mut res = Vec::new();
                res.push((
                    "x".into(),
                    Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
                    ),
                ));
                res.push((
                    "y".into(),
                    Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
                    ),
                ));
                res
            },
        };
        let statement = Statement::parse(
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
        let _ = scope
            .borrow_mut()
            .register_type(&"Point".into(), UserType::Struct(user_type))
            .expect("Registering of user type should have succeeded");
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = Rc::new(RefCell::new(CasmProgram::default()));
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        let instructions = instructions.as_ref().take();
        assert!(instructions.len() > 0);
        // Execute the instructions.
        let memory = Memory::new();
        instructions
            .execute(&memory)
            .expect("Execution should have succeeded");
        let data = clear_stack!(memory);
        let x = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data[0..8],
        )
        .expect("Deserialization should have succeeded");
        let y = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data[8..16],
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(x, Primitive::Number(Number::U64(420)));
        assert_eq!(y, Primitive::Number(Number::U64(69)));
    }

    #[test]
    fn valid_declaration_inplace_struct_general_scope() {
        let user_type = user_type_impl::Struct {
            id: "Point".into(),
            fields: {
                let mut res = Vec::new();
                res.push((
                    "x".into(),
                    Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
                    ),
                ));
                res.push((
                    "y".into(),
                    Either::Static(
                        StaticType::Primitive(PrimitiveType::Number(NumberType::U64)).into(),
                    ),
                ));
                res
            },
        };
        let statement = Statement::parse(
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
        let _ = scope
            .borrow_mut()
            .register_type(&"Point".into(), UserType::Struct(user_type))
            .expect("Registering of user type should have succeeded");
        let _ = statement
            .resolve(&scope, &None, &())
            .expect("Semantic resolution should have succeeded");

        // Code generation.
        let instructions = Rc::new(RefCell::new(CasmProgram::default()));
        statement
            .gencode(&scope, &instructions)
            .expect("Code generation should have succeeded");

        let instructions = instructions.as_ref().take();
        assert!(instructions.len() > 0);
        // Execute the instructions.
        let memory = Memory::new();
        instructions
            .execute(&memory)
            .expect("Execution should have succeeded");
        let data = clear_stack!(memory);
        let x = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data[0..8],
        )
        .expect("Deserialization should have succeeded");
        let y = <PrimitiveType as DeserializeFrom<Scope>>::deserialize_from(
            &PrimitiveType::Number(NumberType::U64),
            &data[8..16],
        )
        .expect("Deserialization should have succeeded");
        assert_eq!(x, Primitive::Number(Number::U64(420)));
        assert_eq!(y, Primitive::Number(Number::U64(69)));
    }
}

use crate::ast::utils::strings::ID;
use crate::e_static;
use crate::semantic::scope::scope::ScopeManager;
use crate::semantic::scope::static_types::{
    NumberType, PrimitiveType, StaticType, StrSliceType, StringType,
};
use crate::semantic::{EType, ResolveCore, TypeOf};
use crate::vm::allocator::heap::Heap;
use crate::vm::allocator::stack::Stack;
use crate::vm::allocator::{align, MemoryAddress};
use crate::vm::asm::operation::{OpPrimitive, PopNum};
use crate::vm::asm::Asm;
use crate::vm::core::lexem;
use crate::vm::core::CoreAsm;

use crate::vm::program::Program;
use crate::vm::runtime::RuntimeError;
use crate::vm::scheduler::Executable;
use crate::vm::stdio::StdIO;
use crate::vm::{CodeGenerationError, GenerateCode};
use crate::{
    ast::expressions::Expression,
    semantic::{Resolve, SemanticError},
};

use super::PathFinder;

#[derive(Debug, Clone, PartialEq)]
pub enum FormatFn {
    ITOA { number_type: NumberType },
    ATOI { number_type: NumberType },
    FTOA,
    ATOF,
    BTOA,
    ATOB,
    CTOA,
    ATOC,
}

impl PathFinder for FormatFn {
    fn find(path: &[String], name: &str) -> Option<Self>
    where
        Self: Sized,
    {
        if (path.len() == 1 && path[0] == lexem::FORMAT) || path.len() == 0 {
            return match name {
                lexem::ITOA => Some(FormatFn::ITOA {
                    number_type: NumberType::I64,
                }),
                lexem::ATOI => Some(FormatFn::ATOI {
                    number_type: NumberType::I64,
                }),
                lexem::FTOA => Some(FormatFn::FTOA),
                lexem::ATOF => Some(FormatFn::ATOF),
                lexem::BTOA => Some(FormatFn::BTOA),
                lexem::ATOB => Some(FormatFn::ATOB),
                lexem::CTOA => Some(FormatFn::CTOA),
                lexem::ATOC => Some(FormatFn::ATOC),
                _ => None,
            };
        }
        None
    }
}

impl ResolveCore for FormatFn {
    fn resolve<E: crate::vm::external::Engine>(
        &mut self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        context: Option<&EType>,
        parameters: &mut Vec<Expression>,
    ) -> Result<EType, SemanticError> {
        match self {
            FormatFn::ITOA { number_type } => {
                if parameters.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let _ = parameters[0].resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
                match parameters[0].type_of(scope_manager, scope_id)? {
                    EType::Static(StaticType::Primitive(PrimitiveType::Number(nt))) => {
                        *number_type = nt;
                    }
                    _ => return Err(SemanticError::IncorrectArguments),
                }
                Ok(EType::Static(StaticType::String(StringType())))
            }
            FormatFn::ATOI { number_type } => {
                if parameters.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let _ = parameters[0].resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
                match parameters[0].type_of(scope_manager, scope_id)? {
                    EType::Static(StaticType::StrSlice(StrSliceType())) => {}
                    _ => return Err(SemanticError::IncorrectArguments),
                }

                match context {
                    Some(EType::Static(StaticType::Primitive(PrimitiveType::Number(nt)))) => {
                        *number_type = nt.clone();
                    }
                    None => *number_type = NumberType::I64,
                    _ => return Err(SemanticError::IncorrectArguments),
                }
                Ok(EType::Static(StaticType::Primitive(PrimitiveType::Number(
                    number_type.to_owned(),
                ))))
            }
            FormatFn::FTOA => {
                if parameters.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let _ = parameters[0].resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
                match parameters[0].type_of(scope_manager, scope_id)? {
                    EType::Static(StaticType::Primitive(PrimitiveType::Number(
                        NumberType::F64,
                    ))) => {}
                    _ => return Err(SemanticError::IncorrectArguments),
                }
                Ok(EType::Static(StaticType::String(StringType())))
            }
            FormatFn::ATOF => {
                if parameters.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let _ = parameters[0].resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
                match parameters[0].type_of(scope_manager, scope_id)? {
                    EType::Static(StaticType::StrSlice(StrSliceType())) => {}
                    _ => return Err(SemanticError::IncorrectArguments),
                }
                Ok(EType::Static(StaticType::Primitive(PrimitiveType::Number(
                    NumberType::F64,
                ))))
            }
            FormatFn::BTOA => {
                if parameters.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let _ = parameters[0].resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
                match parameters[0].type_of(scope_manager, scope_id)? {
                    EType::Static(StaticType::Primitive(PrimitiveType::Bool)) => {}
                    _ => return Err(SemanticError::IncorrectArguments),
                }
                Ok(EType::Static(StaticType::String(StringType())))
            }
            FormatFn::ATOB => {
                if parameters.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let _ = parameters[0].resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
                match parameters[0].type_of(scope_manager, scope_id)? {
                    EType::Static(StaticType::StrSlice(StrSliceType())) => {}
                    _ => return Err(SemanticError::IncorrectArguments),
                }
                Ok(EType::Static(StaticType::Primitive(PrimitiveType::Bool)))
            }
            FormatFn::CTOA => {
                if parameters.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let _ = parameters[0].resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
                match parameters[0].type_of(scope_manager, scope_id)? {
                    EType::Static(StaticType::Primitive(PrimitiveType::Char)) => {}
                    _ => return Err(SemanticError::IncorrectArguments),
                }
                Ok(EType::Static(StaticType::String(StringType())))
            }
            FormatFn::ATOC => {
                if parameters.len() != 1 {
                    return Err(SemanticError::IncorrectArguments);
                }
                let _ = parameters[0].resolve::<E>(scope_manager, scope_id, &None, &mut None)?;
                match parameters[0].type_of(scope_manager, scope_id)? {
                    EType::Static(StaticType::StrSlice(StrSliceType())) => {}
                    _ => return Err(SemanticError::IncorrectArguments),
                }
                Ok(EType::Static(StaticType::Primitive(PrimitiveType::Char)))
            }
        }
    }
}

impl GenerateCode for FormatFn {
    fn gencode<E: crate::vm::external::Engine>(
        &self,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
        context: &crate::vm::CodeGenerationContext,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        match self {
            FormatFn::ITOA { number_type } => match number_type {
                NumberType::U8 => {
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::FormatStart)));
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::U8TOA)));
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::FormatEnd)));
                }
                NumberType::U16 => {
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::FormatStart)));
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::U16TOA)));
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::FormatEnd)));
                }
                NumberType::U32 => {
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::FormatStart)));
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::U32TOA)));
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::FormatEnd)));
                }
                NumberType::U64 => {
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::FormatStart)));
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::U64TOA)));
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::FormatEnd)));
                }
                NumberType::U128 => {
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::FormatStart)));
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::U128TOA)));
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::FormatEnd)));
                }
                NumberType::I8 => {
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::FormatStart)));
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::I8TOA)));
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::FormatEnd)));
                }
                NumberType::I16 => {
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::FormatStart)));
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::I16TOA)));
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::FormatEnd)));
                }
                NumberType::I32 => {
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::FormatStart)));
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::I32TOA)));
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::FormatEnd)));
                }
                NumberType::I64 => {
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::FormatStart)));
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::I64TOA)));
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::FormatEnd)));
                }
                NumberType::I128 => {
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::FormatStart)));
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::I128TOA)));
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::FormatEnd)));
                }
                NumberType::F64 => {
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::FormatStart)));
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::FTOA)));
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::FormatEnd)));
                }
            },
            FormatFn::ATOI { number_type } => match number_type {
                NumberType::U8 => {
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::ATOU8)));
                }
                NumberType::U16 => {
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::ATOU16)));
                }
                NumberType::U32 => {
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::ATOU32)));
                }
                NumberType::U64 => {
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::ATOU64)));
                }
                NumberType::U128 => {
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::ATOU128)));
                }
                NumberType::I8 => {
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::ATOI8)));
                }
                NumberType::I16 => {
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::ATOI16)));
                }
                NumberType::I32 => {
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::ATOI32)));
                }
                NumberType::I64 => {
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::ATOI64)));
                }
                NumberType::I128 => {
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::ATOI128)));
                }
                NumberType::F64 => {
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::ATOF)));
                }
            },
            FormatFn::FTOA => {
                instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::FormatStart)));
                instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::FTOA)));
                instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::FormatEnd)));
            }
            FormatFn::ATOF => instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::ATOF))),
            FormatFn::BTOA => {
                instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::FormatStart)));
                instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::BTOA)));
                instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::FormatEnd)));
            }
            FormatFn::ATOB => instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::ATOB))),
            FormatFn::CTOA => {
                instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::FormatStart)));
                instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::CTOA)));
                instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::FormatEnd)));
            }
            FormatFn::ATOC => instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::ATOC))),
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FormatAsm {
    U128TOA,
    ATOU128,

    U64TOA,
    U64TOH,
    ATOU64,

    U32TOA,
    ATOU32,

    U16TOA,
    ATOU16,

    U8TOA,
    ATOU8,

    I128TOA,
    ATOI128,

    I64TOA,
    ATOI64,

    I32TOA,
    ATOI32,

    I16TOA,
    ATOI16,

    I8TOA,
    ATOI8,

    FTOA,
    ATOF,
    BTOA,
    ATOB,
    CTOA,
    ATOC,

    STOA,
    ETOA,

    PrintfStart,
    FormatStart,
    PushStr(Box<[u8]>),
    PushStrBefore(Box<[u8]>),
    Push,
    InsertBefore(usize),
    Merge(usize),
    Wrap { left: Box<[u8]>, right: Box<[u8]> },
    PrintfEnd,
    FormatEnd,
}

pub mod type_printer {
    use crate::{
        semantic::{
            scope::{
                static_types::{NumberType, PrimitiveType, StaticType, POINTER_SIZE},
                user_types::{Enum, Struct, Union, UserType},
            },
            EType, SizeOf,
        },
        vm::{
            allocator::MemoryAddress,
            asm::{
                branch::{BranchIf, Goto, Label},
                data::Data,
                locate::LocateOffset,
                mem::Mem,
                operation::{Equal, Operation},
                Asm,
            },
            core::CoreAsm,
        },
    };

    use super::FormatAsm;

    fn build_struct<E: crate::vm::external::Engine>(
        Struct { id, fields }: &Struct,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        match fields.len() {
            0 => instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::PushStr(
                format!("{} {{}}", id).as_bytes().into(),
            )))),
            1 => {
                let _ = build(&fields[0].1, scope_manager, scope_id, instructions)?;
                instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::PushStrBefore(
                    format!("{0}: ", fields[0].0).as_bytes().into(),
                ))));
                instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::Merge(2))));

                instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::Wrap {
                    left: format!("{0} {{", id).as_bytes().into(),
                    right: "}".as_bytes().into(),
                })));
            }
            2.. => {
                for i in (1..fields.len()).rev() {
                    let _ = build(&fields[i].1, scope_manager, scope_id, instructions)?;
                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::PushStrBefore(
                        format!("{0}: ", fields[i].0).as_bytes().into(),
                    ))));
                    if i != fields.len() - 1 {
                        instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::PushStr(
                            ", ".as_bytes().into(),
                        ))));
                    }

                    instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::InsertBefore(
                        fields[i - 1].1.size_of(),
                    ))));
                }

                let _ = build(&fields[0].1, scope_manager, scope_id, instructions)?;
                instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::PushStrBefore(
                    format!("{0}: ", fields[0].0).as_bytes().into(),
                ))));
                instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::PushStr(
                    ", ".as_bytes().into(),
                ))));

                instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::Merge(fields.len()))));

                instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::Wrap {
                    left: format!("{0} {{ ", id).as_bytes().into(),
                    right: " }".as_bytes().into(),
                })));
            }
        }
        Ok(())
    }

    pub fn build<E: crate::vm::external::Engine>(
        ctype: &crate::semantic::EType,
        scope_manager: &mut crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
        instructions: &mut crate::vm::program::Program<E>,
    ) -> Result<(), crate::vm::CodeGenerationError> {
        match ctype {
            crate::semantic::EType::Static(static_type) => {
                match static_type {
                    StaticType::Primitive(primitive_type) => {
                        match primitive_type {
                            PrimitiveType::Number(number_type) => {
                                match number_type {
                                    NumberType::U8 => instructions
                                        .push(Asm::Core(CoreAsm::Format(FormatAsm::U8TOA))),
                                    NumberType::U16 => instructions
                                        .push(Asm::Core(CoreAsm::Format(FormatAsm::U16TOA))),
                                    NumberType::U32 => instructions
                                        .push(Asm::Core(CoreAsm::Format(FormatAsm::U32TOA))),
                                    NumberType::U64 => instructions
                                        .push(Asm::Core(CoreAsm::Format(FormatAsm::U64TOA))),
                                    NumberType::U128 => instructions
                                        .push(Asm::Core(CoreAsm::Format(FormatAsm::U128TOA))),
                                    NumberType::I8 => instructions
                                        .push(Asm::Core(CoreAsm::Format(FormatAsm::I8TOA))),
                                    NumberType::I16 => instructions
                                        .push(Asm::Core(CoreAsm::Format(FormatAsm::I16TOA))),
                                    NumberType::I32 => instructions
                                        .push(Asm::Core(CoreAsm::Format(FormatAsm::I32TOA))),
                                    NumberType::I64 => instructions
                                        .push(Asm::Core(CoreAsm::Format(FormatAsm::I64TOA))),
                                    NumberType::I128 => instructions
                                        .push(Asm::Core(CoreAsm::Format(FormatAsm::I128TOA))),
                                    NumberType::F64 => instructions
                                        .push(Asm::Core(CoreAsm::Format(FormatAsm::FTOA))),
                                }
                            }
                            PrimitiveType::Char => {
                                instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::CTOA)))
                            }
                            PrimitiveType::Bool => {
                                instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::BTOA)))
                            }
                        }
                    }
                    StaticType::StrSlice(_) => {
                        instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::STOA)));
                        instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::Wrap {
                            left: "\"".as_bytes().into(),
                            right: "\"".as_bytes().into(),
                        })));
                    }
                    StaticType::String(_) => {
                        instructions.push(Asm::Offset(LocateOffset {
                            offset: POINTER_SIZE,
                        }));
                        instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::STOA)));
                        instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::Wrap {
                            left: "\"".as_bytes().into(),
                            right: "\"".as_bytes().into(),
                        })));
                    }
                    StaticType::Tuple(tuple_type) => match tuple_type.0.len() {
                        0 => instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::PushStr(
                            "()".as_bytes().into(),
                        )))),
                        1 => {
                            let _ = build(&tuple_type.0[0], scope_manager, scope_id, instructions)?;
                            instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::Wrap {
                                left: "(".as_bytes().into(),
                                right: ")".as_bytes().into(),
                            })));
                        }
                        2.. => {
                            for i in (1..tuple_type.0.len()).rev() {
                                let _ =
                                    build(&tuple_type.0[i], scope_manager, scope_id, instructions)?;
                                if i != tuple_type.0.len() - 1 {
                                    instructions.push(Asm::Core(CoreAsm::Format(
                                        FormatAsm::PushStr(", ".as_bytes().into()),
                                    )));
                                }
                                instructions.push(Asm::Core(CoreAsm::Format(
                                    FormatAsm::InsertBefore(tuple_type.0[i - 1].size_of()),
                                )));
                            }

                            let _ = build(&tuple_type.0[0], scope_manager, scope_id, instructions)?;
                            instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::PushStr(
                                ", ".as_bytes().into(),
                            ))));
                            instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::Merge(
                                tuple_type.0.len(),
                            ))));
                            instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::Wrap {
                                left: "(".as_bytes().into(),
                                right: ")".as_bytes().into(),
                            })));
                        }
                    },
                    StaticType::Slice(_)
                    | StaticType::Vec(_)
                    | StaticType::Map(_)
                    | StaticType::Address(_) => {
                        instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::U64TOH)));
                        instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::PushStrBefore(
                            "@".as_bytes().into(),
                        ))));
                        instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::PushStrBefore(
                            ctype.name(scope_manager, scope_id)?.as_bytes().into(),
                        ))));
                    }
                    StaticType::Closure(_) | StaticType::Lambda(_) | StaticType::Function(_) => {
                        instructions.push(Asm::Pop(POINTER_SIZE));

                        instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::PushStr(
                            ctype.name(scope_manager, scope_id)?.as_bytes().into(),
                        ))));
                        instructions.push(Asm::Data(Data::Serialized {
                            data: 0u64.to_le_bytes().into(),
                        }));
                    }
                    StaticType::Unit => {
                        instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::PushStr(
                            "unit".as_bytes().into(),
                        ))));
                        instructions.push(Asm::Data(Data::Serialized {
                            data: 0u64.to_le_bytes().into(),
                        }));
                    }
                    StaticType::Any => {
                        instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::PushStr(
                            "any".as_bytes().into(),
                        ))));
                        instructions.push(Asm::Data(Data::Serialized {
                            data: 0u64.to_le_bytes().into(),
                        }));
                    }
                    StaticType::Error => {
                        instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::ETOA)))
                    }
                }
            }
            crate::semantic::EType::User { id, size } => {
                let Ok(utype) = scope_manager.find_type_by_id(*id, scope_id) else {
                    return Err(crate::vm::CodeGenerationError::UnresolvedError);
                };
                match utype {
                    UserType::Struct(struct_type) => {
                        let _ = build_struct(&struct_type, scope_manager, scope_id, instructions)?;
                    }
                    UserType::Enum(Enum { id, values }) => {
                        let mut else_label = Label::gen();
                        let end_label = Label::gen();
                        instructions.push(Asm::Data(Data::Serialized {
                            data: 0u64.to_le_bytes().into(),
                        }));
                        instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::PushStr(
                            format!("{0}::", id).as_bytes().into(),
                        ))));
                        instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::InsertBefore(
                            POINTER_SIZE,
                        ))));
                        for (i, variant_name) in values.iter().enumerate() {
                            instructions.push_label_by_id(
                                else_label,
                                format!("format_{variant_name}").into(),
                            );
                            instructions.push(Asm::Mem(Mem::Dup(POINTER_SIZE)));

                            instructions.push(Asm::Data(Data::Serialized {
                                data: i.to_le_bytes().into(),
                            }));
                            instructions.push(Asm::Operation(Operation {
                                kind: crate::vm::asm::operation::OperationKind::Equal(Equal {
                                    left: 8,
                                    right: 8,
                                }),
                            }));
                            else_label = Label::gen();
                            instructions.push(Asm::If(BranchIf { else_label }));
                            instructions.push(Asm::Pop(POINTER_SIZE));
                            instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::PushStr(
                                format!("{0}", variant_name).as_bytes().into(),
                            ))));
                            instructions.push(Asm::Goto(Goto {
                                label: Some(end_label),
                            }));
                        }
                        instructions.push_label_by_id(else_label, format!("format_union").into());
                        instructions
                            .push_label_by_id(end_label, format!("end_format_union").into());
                    }
                    UserType::Union(Union { id, variants }) => {
                        let mut else_label = Label::gen();
                        let end_label = Label::gen();

                        for (i, (variant_name, struct_type)) in variants.iter().enumerate() {
                            instructions.push_label_by_id(
                                else_label,
                                format!("format_{variant_name}").into(),
                            );
                            instructions.push(Asm::Mem(Mem::Dup(POINTER_SIZE)));

                            instructions.push(Asm::Data(Data::Serialized {
                                data: i.to_le_bytes().into(),
                            }));
                            instructions.push(Asm::Operation(Operation {
                                kind: crate::vm::asm::operation::OperationKind::Equal(Equal {
                                    left: 8,
                                    right: 8,
                                }),
                            }));
                            else_label = Label::gen();
                            instructions.push(Asm::If(BranchIf { else_label }));
                            instructions.push(Asm::Pop(POINTER_SIZE));
                            let _ =
                                build_struct(&struct_type, scope_manager, scope_id, instructions)?;
                            instructions.push(Asm::Goto(Goto {
                                label: Some(end_label),
                            }));
                        }
                        instructions.push_label_by_id(else_label, format!("format_union").into());
                        instructions
                            .push_label_by_id(end_label, format!("end_format_union").into());
                        instructions.push(Asm::Core(CoreAsm::Format(FormatAsm::PushStrBefore(
                            format!("{0}::", id).as_bytes().into(),
                        ))));
                    }
                }
            }
        }
        Ok(())
    }
}

impl<E: crate::vm::external::Engine> crate::vm::AsmName<E> for FormatAsm {
    fn name(&self, stdio: &mut StdIO, program: &crate::vm::program::Program<E>, engine: &mut E) {
        match self {
            FormatAsm::U128TOA => stdio.push_asm_lib(engine, "u128toa"),
            FormatAsm::ATOU128 => stdio.push_asm_lib(engine, "atou128"),
            FormatAsm::U64TOA => stdio.push_asm_lib(engine, "u64toa"),
            FormatAsm::U64TOH => stdio.push_asm_lib(engine, "u64toh"),
            FormatAsm::ATOU64 => stdio.push_asm_lib(engine, "atou64"),
            FormatAsm::U32TOA => stdio.push_asm_lib(engine, "u32toa"),
            FormatAsm::ATOU32 => stdio.push_asm_lib(engine, "atou32"),
            FormatAsm::U16TOA => stdio.push_asm_lib(engine, "u16toa"),
            FormatAsm::ATOU16 => stdio.push_asm_lib(engine, "atou16"),
            FormatAsm::U8TOA => stdio.push_asm_lib(engine, "u8toa"),
            FormatAsm::ATOU8 => stdio.push_asm_lib(engine, "atou8"),
            FormatAsm::I128TOA => stdio.push_asm_lib(engine, "i128toa"),
            FormatAsm::ATOI128 => stdio.push_asm_lib(engine, "atoi128"),
            FormatAsm::I64TOA => stdio.push_asm_lib(engine, "i64toa"),
            FormatAsm::ATOI64 => stdio.push_asm_lib(engine, "atoi64"),
            FormatAsm::I32TOA => stdio.push_asm_lib(engine, "i32toa"),
            FormatAsm::ATOI32 => stdio.push_asm_lib(engine, "atoi32"),
            FormatAsm::I16TOA => stdio.push_asm_lib(engine, "i16toa"),
            FormatAsm::ATOI16 => stdio.push_asm_lib(engine, "atoi16"),
            FormatAsm::I8TOA => stdio.push_asm_lib(engine, "i8toa"),
            FormatAsm::ATOI8 => stdio.push_asm_lib(engine, "atoi8"),
            FormatAsm::FTOA => stdio.push_asm_lib(engine, "ftoa"),
            FormatAsm::ATOF => stdio.push_asm_lib(engine, "atof"),
            FormatAsm::BTOA => stdio.push_asm_lib(engine, "btoa"),
            FormatAsm::ATOB => stdio.push_asm_lib(engine, "atob"),
            FormatAsm::CTOA => stdio.push_asm_lib(engine, "ctoa"),
            FormatAsm::ATOC => stdio.push_asm_lib(engine, "atoc"),
            FormatAsm::STOA => stdio.push_asm_lib(engine, "stoa"),
            FormatAsm::ETOA => stdio.push_asm_lib(engine, "etoa"),
            FormatAsm::PrintfStart => stdio.push_asm_lib(engine, "printfstart"),
            FormatAsm::FormatStart => stdio.push_asm_lib(engine, "formatstart"),
            FormatAsm::PushStr(_) => stdio.push_asm_lib(engine, "pushstr"),
            FormatAsm::PushStrBefore(_) => stdio.push_asm_lib(engine, "pushstr_before"),
            FormatAsm::Push => stdio.push_asm_lib(engine, "push"),
            FormatAsm::InsertBefore(size) => stdio.push_asm_lib(engine, &format!("insert -{size}")),
            FormatAsm::Merge(count) => stdio.push_asm_lib(engine, &format!("merge {count}")),
            FormatAsm::Wrap { left, right } => stdio.push_asm_lib(engine, "wrap"),
            FormatAsm::PrintfEnd => stdio.push_asm_lib(engine, "printfend"),
            FormatAsm::FormatEnd => stdio.push_asm_lib(engine, "formatend"),
        }
    }
}

impl crate::vm::AsmWeight for FormatAsm {
    fn weight(&self) -> crate::vm::Weight {
        match self {
            FormatAsm::U128TOA
            | FormatAsm::ATOU128
            | FormatAsm::U64TOA
            | FormatAsm::U64TOH
            | FormatAsm::ATOU64
            | FormatAsm::U32TOA
            | FormatAsm::ATOU32
            | FormatAsm::U16TOA
            | FormatAsm::ATOU16
            | FormatAsm::U8TOA
            | FormatAsm::ATOU8
            | FormatAsm::I128TOA
            | FormatAsm::ATOI128
            | FormatAsm::I64TOA
            | FormatAsm::ATOI64
            | FormatAsm::I32TOA
            | FormatAsm::ATOI32
            | FormatAsm::I16TOA
            | FormatAsm::ATOI16
            | FormatAsm::I8TOA
            | FormatAsm::ATOI8
            | FormatAsm::FTOA
            | FormatAsm::ATOF
            | FormatAsm::BTOA
            | FormatAsm::ATOB
            | FormatAsm::CTOA
            | FormatAsm::ATOC
            | FormatAsm::STOA
            | FormatAsm::ETOA => crate::vm::Weight::LOW,
            FormatAsm::PrintfStart => crate::vm::Weight::ZERO,
            FormatAsm::FormatStart => crate::vm::Weight::ZERO,
            FormatAsm::PushStr(_) => crate::vm::Weight::LOW,
            FormatAsm::PushStrBefore(_) => crate::vm::Weight::LOW,
            FormatAsm::Push => crate::vm::Weight::ZERO,
            FormatAsm::InsertBefore(_) => crate::vm::Weight::ZERO,
            FormatAsm::Merge(_) => crate::vm::Weight::ZERO,
            FormatAsm::Wrap { left, right } => crate::vm::Weight::ZERO,
            FormatAsm::PrintfEnd => crate::vm::Weight::EXTREME,
            FormatAsm::FormatEnd => crate::vm::Weight::HIGH,
        }
    }
}

impl<E: crate::vm::external::Engine> Executable<E> for FormatAsm {
    fn execute<P: crate::vm::scheduler::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler::Scheduler<P>,
        signal_handler: &mut crate::vm::runtime::SignalHandler<E>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler::ExecutionContext<E::FunctionContext, E::TID>,
    ) -> Result<(), RuntimeError> {
        match self {
            FormatAsm::U128TOA => {
                let data = OpPrimitive::pop_num::<u128>(stack)?;
                let words = data.to_string();
                let _ = stack.push_with(words.as_bytes())?;
                let _ = stack.push_with(&((words.len() as u64).to_le_bytes()))?;
            }
            FormatAsm::ATOU128 => {
                let address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                let string = OpPrimitive::get_string_from(address, stack, heap)?;
                let number: u128 = string.parse().map_err(|_| RuntimeError::Deserialization)?;
                let _ = stack.push_with(&number.to_le_bytes())?;
            }
            FormatAsm::U64TOA => {
                let data = OpPrimitive::pop_num::<u64>(stack)?;
                let words = data.to_string();
                let _ = stack.push_with(words.as_bytes())?;
                let _ = stack.push_with(&((words.len() as u64).to_le_bytes()))?;
            }
            FormatAsm::U64TOH => {
                let data = OpPrimitive::pop_num::<u64>(stack)?;
                let words = format!("0x{:X}", data);
                let _ = stack.push_with(words.as_bytes())?;
                let _ = stack.push_with(&((words.len() as u64).to_le_bytes()))?;
            }
            FormatAsm::ATOU64 => {
                let address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                let string = OpPrimitive::get_string_from(address, stack, heap)?;
                let number: u64 = string.parse().map_err(|_| RuntimeError::Deserialization)?;
                let _ = stack.push_with(&number.to_le_bytes())?;
            }
            FormatAsm::U32TOA => {
                let data = OpPrimitive::pop_num::<u32>(stack)?;
                let words = data.to_string();
                let _ = stack.push_with(words.as_bytes())?;
                let _ = stack.push_with(&((words.len() as u64).to_le_bytes()))?;
            }
            FormatAsm::ATOU32 => {
                let address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                let string = OpPrimitive::get_string_from(address, stack, heap)?;
                let number: u32 = string.parse().map_err(|_| RuntimeError::Deserialization)?;
                let _ = stack.push_with(&number.to_le_bytes())?;
            }
            FormatAsm::U16TOA => {
                let data = OpPrimitive::pop_num::<u16>(stack)?;
                let words = data.to_string();
                let _ = stack.push_with(words.as_bytes())?;
                let _ = stack.push_with(&((words.len() as u64).to_le_bytes()))?;
            }
            FormatAsm::ATOU16 => {
                let address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                let string = OpPrimitive::get_string_from(address, stack, heap)?;
                let number: u16 = string.parse().map_err(|_| RuntimeError::Deserialization)?;
                let _ = stack.push_with(&number.to_le_bytes())?;
            }
            FormatAsm::U8TOA => {
                let data = OpPrimitive::pop_num::<u8>(stack)?;
                let words = data.to_string();
                let _ = stack.push_with(words.as_bytes())?;
                let _ = stack.push_with(&((words.len() as u64).to_le_bytes()))?;
            }
            FormatAsm::ATOU8 => {
                let address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                let string = OpPrimitive::get_string_from(address, stack, heap)?;
                let number: u8 = string.parse().map_err(|_| RuntimeError::Deserialization)?;
                let _ = stack.push_with(&number.to_le_bytes())?;
            }
            FormatAsm::I128TOA => {
                let data = OpPrimitive::pop_num::<i128>(stack)?;
                let words = data.to_string();
                let _ = stack.push_with(words.as_bytes())?;
                let _ = stack.push_with(&((words.len() as u64).to_le_bytes()))?;
            }
            FormatAsm::ATOI128 => {
                let address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                let string = OpPrimitive::get_string_from(address, stack, heap)?;
                let number: i128 = string.parse().map_err(|_| RuntimeError::Deserialization)?;
                let _ = stack.push_with(&number.to_le_bytes())?;
            }
            FormatAsm::I64TOA => {
                let data = OpPrimitive::pop_num::<i64>(stack)?;
                let words = data.to_string();
                let _ = stack.push_with(words.as_bytes())?;
                let _ = stack.push_with(&((words.len() as u64).to_le_bytes()))?;
            }
            FormatAsm::ATOI64 => {
                let address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                let string = OpPrimitive::get_string_from(address, stack, heap)?;
                let number: i64 = string.parse().map_err(|_| RuntimeError::Deserialization)?;
                let _ = stack.push_with(&number.to_le_bytes())?;
            }
            FormatAsm::I32TOA => {
                let data = OpPrimitive::pop_num::<i32>(stack)?;
                let words = data.to_string();
                let _ = stack.push_with(words.as_bytes())?;
                let _ = stack.push_with(&((words.len() as u64).to_le_bytes()))?;
            }
            FormatAsm::ATOI32 => {
                let address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                let string = OpPrimitive::get_string_from(address, stack, heap)?;
                let number: i32 = string.parse().map_err(|_| RuntimeError::Deserialization)?;
                let _ = stack.push_with(&number.to_le_bytes())?;
            }
            FormatAsm::I16TOA => {
                let data = OpPrimitive::pop_num::<i16>(stack)?;
                let words = data.to_string();
                let _ = stack.push_with(words.as_bytes())?;
                let _ = stack.push_with(&((words.len() as u64).to_le_bytes()))?;
            }
            FormatAsm::ATOI16 => {
                let address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                let string = OpPrimitive::get_string_from(address, stack, heap)?;
                let number: i16 = string.parse().map_err(|_| RuntimeError::Deserialization)?;
                let _ = stack.push_with(&number.to_le_bytes())?;
            }
            FormatAsm::I8TOA => {
                let data = OpPrimitive::pop_num::<i8>(stack)?;
                let words = data.to_string();
                let _ = stack.push_with(words.as_bytes())?;
                let _ = stack.push_with(&((words.len() as u64).to_le_bytes()))?;
            }
            FormatAsm::ATOI8 => {
                let address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                let string = OpPrimitive::get_string_from(address, stack, heap)?;
                let number: i8 = string.parse().map_err(|_| RuntimeError::Deserialization)?;
                let _ = stack.push_with(&number.to_le_bytes())?;
            }
            FormatAsm::FTOA => {
                let data = OpPrimitive::pop_float(stack)?;
                let words = data.to_string();
                let _ = stack.push_with(words.as_bytes())?;
                let _ = stack.push_with(&((words.len() as u64).to_le_bytes()))?;
            }
            FormatAsm::ATOF => {
                let address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                let string = OpPrimitive::get_string_from(address, stack, heap)?;
                let number: f64 = string.parse().map_err(|_| RuntimeError::Deserialization)?;
                let _ = stack.push_with(&number.to_le_bytes())?;
            }
            FormatAsm::BTOA => {
                let data = OpPrimitive::pop_num::<u8>(stack)?;
                let words = if data >= 1 { "true" } else { "false" }.to_string();
                let _ = stack.push_with(words.as_bytes())?;
                let _ = stack.push_with(&((words.len() as u64).to_le_bytes()))?;
            }
            FormatAsm::ATOB => {
                let address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                let string = OpPrimitive::get_string_from(address, stack, heap)?;
                let value: bool = string.parse().map_err(|_| RuntimeError::Deserialization)?;
                if value {
                    let _ = stack.push_with(&(1u8).to_le_bytes())?;
                } else {
                    let _ = stack.push_with(&(0u8).to_le_bytes())?;
                }
            }
            FormatAsm::CTOA => {
                let data = OpPrimitive::pop_char(stack)?;
                let words = format!("'{0}'", data.to_string());
                let _ = stack.push_with(words.as_bytes())?;
                let _ = stack.push_with(&((words.len() as u64).to_le_bytes()))?;
            }
            FormatAsm::ATOC => {
                let address: MemoryAddress = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                let string = OpPrimitive::get_string_from(address, stack, heap)?;
                let value: char = string.chars().next().ok_or(RuntimeError::Deserialization)?;
                let mut buffer = [0u8; 4];
                let _ = value.encode_utf8(&mut buffer);
                let _ = stack.push_with(&buffer)?;
            }
            FormatAsm::STOA => {
                let address = OpPrimitive::pop_num::<u64>(stack)?.try_into()?;
                let words = OpPrimitive::get_string_from(address, stack, heap)?;
                let _ = stack.push_with(words.as_bytes())?;
                let _ = stack.push_with(&((words.len() as u64).to_le_bytes()))?;
            }
            FormatAsm::ETOA => {
                let data = OpPrimitive::pop_num::<u8>(stack)?;
                let words = if data >= super::ERROR_VALUE {
                    "ERROR".to_string()
                } else {
                    "OK".to_string()
                };
                let _ = stack.push_with(words.as_bytes())?;
                let _ = stack.push_with(&((words.len() as u64).to_le_bytes()))?;
            }
            FormatAsm::PrintfStart => {
                let _ = stack.push_with(&((0u64).to_le_bytes()))?;
            }
            FormatAsm::FormatStart => {
                let _ = stack.push_with(&((0u64).to_le_bytes()))?;
            }
            FormatAsm::PushStr(words) => {
                let size = OpPrimitive::pop_num::<u64>(stack)?;
                let mut data = stack.pop(size as usize)?.to_vec();
                data.extend_from_slice(&words);

                let _ = stack.push_with(&data)?;
                let _ = stack.push_with(&((data.len() as u64).to_le_bytes()))?;
            }
            FormatAsm::PushStrBefore(words) => {
                let size = OpPrimitive::pop_num::<u64>(stack)?;
                let data = stack.pop(size as usize)?.to_vec();
                let mut result = words.to_vec();
                result.extend_from_slice(&data);

                let _ = stack.push_with(&result)?;
                let _ = stack.push_with(&((result.len() as u64).to_le_bytes()))?;
            }
            FormatAsm::Push => {
                let size = OpPrimitive::pop_num::<u64>(stack)?;
                let right = stack.pop(size as usize)?.to_vec();
                let size = OpPrimitive::pop_num::<u64>(stack)?;
                let mut left = stack.pop(size as usize)?.to_vec();
                left.extend_from_slice(&right);

                let _ = stack.push_with(&left)?;
                let _ = stack.push_with(&((left.len() as u64).to_le_bytes()))?;
            }
            FormatAsm::InsertBefore(before) => {
                let size = OpPrimitive::pop_num::<u64>(stack)?;
                let data = stack.pop(size as usize)?.to_vec();

                let real = stack.pop(*before)?.to_vec();

                let _ = stack.push_with(&data)?;
                let _ = stack.push_with(&((data.len() as u64).to_le_bytes()))?;
                let _ = stack.push_with(&real)?;
            }
            FormatAsm::Merge(count) => {
                let mut buffer = Vec::with_capacity(*count);
                for _ in 0..*count {
                    let size = OpPrimitive::pop_num::<u64>(stack)?;
                    let data = stack.pop(size as usize)?.to_vec();
                    buffer.push(data);
                }
                let rev_buffer: Vec<u8> = buffer.into_iter().flatten().collect();

                let _ = stack.push_with(&rev_buffer)?;
                let _ = stack.push_with(&((rev_buffer.len() as u64).to_le_bytes()))?;
            }
            FormatAsm::Wrap { left, right } => {
                let size = OpPrimitive::pop_num::<u64>(stack)?;
                let data = stack.pop(size as usize)?.to_vec();
                let mut words = Vec::with_capacity(left.len() + data.len() + right.len());
                words.extend_from_slice(left);
                words.extend_from_slice(&data);
                words.extend_from_slice(right);

                let _ = stack.push_with(&words)?;
                let _ = stack.push_with(&((words.len() as u64).to_le_bytes()))?;
            }
            FormatAsm::PrintfEnd => {
                let size = OpPrimitive::pop_num::<u64>(stack)?;
                let data = stack.pop(size as usize)?.to_vec();
                stdio.stdout.push(&String::from_utf8_lossy(&data));
                stdio.stdout.flushln(engine);
            }
            FormatAsm::FormatEnd => {
                let size = OpPrimitive::pop_num::<u64>(stack)?;
                let data = stack.pop(size as usize)?.to_vec();

                let len = data.len();
                let cap = len * 2;
                let address = heap.alloc(cap)?;

                /* Write capacity */
                let _ = heap.write(address, &(cap as u64).to_le_bytes())?;
                /* Write len */
                let _ = heap.write(address.add(8), &(len as u64).to_le_bytes())?;

                /* Write slice */
                let _ = heap.write(address.add(super::string::STRING_HEADER), &data)?;

                /* Push vec address */
                let address: u64 = address.into(stack);
                stack.push_with(&address.to_le_bytes())?;
            }
        }
        scheduler.next();
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        test_extract_variable, test_extract_variable_with, test_statements,
        vm::asm::operation::GetNumFrom,
    };

    use super::*;

    #[test]
    fn valid_format() {
        let mut engine = crate::vm::external::test::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &crate::semantic::scope::scope::ScopeManager,
            stack: &crate::vm::allocator::stack::Stack,
            heap: &crate::vm::allocator::heap::Heap,
        ) -> bool {
            test_extract_variable_with(
                "text",
                |address, stack, heap| {
                    let address: MemoryAddress =
                        OpPrimitive::get_num_from::<u64>(address, stack, heap)
                            .expect("Deserialization should have succeeded")
                            .try_into()
                            .unwrap();
                    let address = address.add(8);

                    let text = OpPrimitive::get_string_from(address, stack, heap)
                        .expect("Deserialization should have succeeded");

                    assert_eq!(text, "Hello World from 69");
                },
                scope_manager,
                stack,
                heap,
            );
            true
        }

        test_statements(
            r##"
        let text = format("Hello World from {60 + 9}");
        
        "##,
            &mut engine,
            assert_fn,
        );
    }

    pub fn test_extract_variable_bool(
        variable_name: &str,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        stack: &crate::vm::allocator::stack::Stack,
        heap: &crate::vm::allocator::heap::Heap,
    ) -> Option<bool> {
        let crate::semantic::scope::scope::Variable { id, .. } = scope_manager
            .find_var_by_name(variable_name, None)
            .expect("The variable should have been found");

        let crate::semantic::scope::scope::VariableInfo { address, ctype, .. } = scope_manager
            .find_var_by_id(id)
            .expect("The variable should have been found");

        let address: crate::vm::allocator::MemoryAddress = (*address)
            .try_into()
            .expect("the address should have been known");
        let res = crate::vm::asm::operation::OpPrimitive::get_bool_from(address, stack, heap)
            .expect("Deserialization should have succeeded");

        return Some(res);
    }

    pub fn test_extract_variable_char(
        variable_name: &str,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        stack: &crate::vm::allocator::stack::Stack,
        heap: &crate::vm::allocator::heap::Heap,
    ) -> Option<char> {
        let crate::semantic::scope::scope::Variable { id, .. } = scope_manager
            .find_var_by_name(variable_name, None)
            .expect("The variable should have been found");

        let crate::semantic::scope::scope::VariableInfo { address, ctype, .. } = scope_manager
            .find_var_by_id(id)
            .expect("The variable should have been found");

        let address: crate::vm::allocator::MemoryAddress = (*address)
            .try_into()
            .expect("the address should have been known");
        let res = crate::vm::asm::operation::OpPrimitive::get_char_from(address, stack, heap)
            .expect("Deserialization should have succeeded");

        return Some(res);
    }

    pub fn test_extract_variable_float(
        variable_name: &str,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        stack: &crate::vm::allocator::stack::Stack,
        heap: &crate::vm::allocator::heap::Heap,
    ) -> Option<f64> {
        let crate::semantic::scope::scope::Variable { id, .. } = scope_manager
            .find_var_by_name(variable_name, None)
            .expect("The variable should have been found");

        let crate::semantic::scope::scope::VariableInfo { address, ctype, .. } = scope_manager
            .find_var_by_id(id)
            .expect("The variable should have been found");

        let address: crate::vm::allocator::MemoryAddress = (*address)
            .try_into()
            .expect("the address should have been known");
        let res = crate::vm::asm::operation::OpPrimitive::get_float_from(address, stack, heap)
            .expect("Deserialization should have succeeded");

        return Some(res);
    }

    #[test]
    fn valid_ato_x() {
        let mut engine = crate::vm::external::test::NoopGameEngine {};

        fn assert_fn(
            scope_manager: &crate::semantic::scope::scope::ScopeManager,
            stack: &crate::vm::allocator::stack::Stack,
            heap: &crate::vm::allocator::heap::Heap,
        ) -> bool {
            let res = test_extract_variable::<i64>("res1", scope_manager, stack, heap)
                .expect("Deserialiresation should have succeeded");
            assert_eq!(res, 1);
            let res = test_extract_variable::<u128>("res2", scope_manager, stack, heap)
                .expect("Deserialiresation should have succeeded");
            assert_eq!(res, 2);
            let res = test_extract_variable::<u64>("res3", scope_manager, stack, heap)
                .expect("Deserialiresation should have succeeded");
            assert_eq!(res, 3);
            let res = test_extract_variable::<u32>("res4", scope_manager, stack, heap)
                .expect("Deserialiresation should have succeeded");
            assert_eq!(res, 4);
            let res = test_extract_variable::<u16>("res5", scope_manager, stack, heap)
                .expect("Deserialiresation should have succeeded");
            assert_eq!(res, 5);
            let res = test_extract_variable::<u8>("res6", scope_manager, stack, heap)
                .expect("Deserialiresation should have succeeded");
            assert_eq!(res, 6);
            let res = test_extract_variable::<i128>("res7", scope_manager, stack, heap)
                .expect("Deserialiresation should have succeeded");
            assert_eq!(res, 7);
            let res = test_extract_variable::<i32>("res8", scope_manager, stack, heap)
                .expect("Deserialiresation should have succeeded");
            assert_eq!(res, 8);
            let res = test_extract_variable::<i16>("res9", scope_manager, stack, heap)
                .expect("Deserialiresation should have succeeded");
            assert_eq!(res, 9);
            let res = test_extract_variable::<i8>("res10", scope_manager, stack, heap)
                .expect("Deserialiresation should have succeeded");
            assert_eq!(res, 10);
            let res = test_extract_variable_bool("res11", scope_manager, stack, heap)
                .expect("Deserialiresation should have succeeded");
            assert_eq!(res, true);
            let res = test_extract_variable_bool("res12", scope_manager, stack, heap)
                .expect("Deserialiresation should have succeeded");
            assert_eq!(res, false);
            let res = test_extract_variable_char("res13", scope_manager, stack, heap)
                .expect("Deserialiresation should have succeeded");
            assert_eq!(res, 'a');
            let res = test_extract_variable_float("res14", scope_manager, stack, heap)
                .expect("Deserialiresation should have succeeded");
            assert_eq!(res, 69.420);
            true
        }

        test_statements(
            r##"
        let res1 = atoi("1");
        let res2 : u128 = atoi("2");
        let res3 = atoi("3") as u64;
        let res4 : u32 = atoi("4");
        let res5 : u16 = atoi("5");
        let res6 : u8 = atoi("6");
        let res7 : i128 = atoi("7");
        let res8 : i32 = atoi("8");
        let res9 : i16 = atoi("9");
        let res10 : i8 = atoi("10");
        let res11 : bool = atob("true");
        let res12 = atob("false");
        let res13 = atoc("a");
        let res14 = atof("69.420");
        "##,
            &mut engine,
            assert_fn,
        );
    }
}

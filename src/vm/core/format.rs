use crate::ast::utils::strings::ID;
use crate::e_static;
use crate::semantic::scope::scope::ScopeManager;
use crate::semantic::scope::static_types::{
    NumberType, PrimitiveType, StaticType, StrSliceType, StringType,
};
use crate::semantic::{EType, ResolveCore, TypeOf};
use crate::vm::allocator::align;
use crate::vm::allocator::heap::Heap;
use crate::vm::allocator::stack::Stack;
use crate::vm::asm::operation::{OpPrimitive, PopNum};
use crate::vm::asm::Asm;
use crate::vm::core::lexem;
use crate::vm::core::CoreAsm;

use crate::vm::program::Program;
use crate::vm::runtime::RuntimeError;
use crate::vm::scheduler_v2::Executable;
use crate::vm::stdio::StdIO;
use crate::vm::{CodeGenerationError, GenerateCode};
use crate::{
    ast::expressions::Expression,
    semantic::{Resolve, SemanticError},
};

use super::PathFinder;

#[derive(Debug, Clone, PartialEq)]
pub enum FormatFn {
    Format,
    ITOA,
    ATOI,
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
                lexem::FORMAT => Some(FormatFn::Format),
                lexem::ITOA => Some(FormatFn::ITOA),
                lexem::ATOI => Some(FormatFn::ATOI),
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
            FormatFn::Format => {
                //

                Ok(EType::Static(StaticType::String(StringType())))
            }
            FormatFn::ITOA => todo!(),
            FormatFn::ATOI => todo!(),
            FormatFn::FTOA => todo!(),
            FormatFn::ATOF => todo!(),
            FormatFn::BTOA => todo!(),
            FormatFn::ATOB => todo!(),
            FormatFn::CTOA => todo!(),
            FormatFn::ATOC => todo!(),
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
            FormatFn::Format => {
                //
            }
            FormatFn::ITOA => todo!(),
            FormatFn::ATOI => todo!(),
            FormatFn::FTOA => todo!(),
            FormatFn::ATOF => todo!(),
            FormatFn::BTOA => todo!(),
            FormatFn::ATOB => todo!(),
            FormatFn::CTOA => todo!(),
            FormatFn::ATOC => todo!(),
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
                for i in fields.len() - 1..=1 {
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
                            for i in tuple_type.0.len() - 1..=1 {
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
                    StaticType::Range(range_type) => todo!(),
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

impl<E: crate::vm::external::Engine> Executable<E>
    for FormatAsm
{
    fn execute<P: crate::vm::scheduler_v2::SchedulingPolicy>(
        &self,
        program: &crate::vm::program::Program<E>,
        scheduler: &mut crate::vm::scheduler_v2::Scheduler<P>,
        stack: &mut crate::vm::allocator::stack::Stack,
        heap: &mut crate::vm::allocator::heap::Heap,
        stdio: &mut crate::vm::stdio::StdIO,
        engine: &mut E,
        context: &crate::vm::scheduler_v2::ExecutionContext,
    ) -> Result<(), RuntimeError> {
        match self {
            FormatAsm::U128TOA => {
                let data = OpPrimitive::pop_num::<u128>(stack)?;
                let words = data.to_string();
                let _ = stack.push_with(words.as_bytes())?;
                let _ = stack.push_with(&((words.len() as u64).to_le_bytes()))?;
            }
            FormatAsm::ATOU128 => todo!(),
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
            FormatAsm::ATOU64 => todo!(),
            FormatAsm::U32TOA => {
                let data = OpPrimitive::pop_num::<u32>(stack)?;
                let words = data.to_string();
                let _ = stack.push_with(words.as_bytes())?;
                let _ = stack.push_with(&((words.len() as u64).to_le_bytes()))?;
            }
            FormatAsm::ATOU32 => todo!(),
            FormatAsm::U16TOA => {
                let data = OpPrimitive::pop_num::<u16>(stack)?;
                let words = data.to_string();
                let _ = stack.push_with(words.as_bytes())?;
                let _ = stack.push_with(&((words.len() as u64).to_le_bytes()))?;
            }
            FormatAsm::ATOU16 => todo!(),
            FormatAsm::U8TOA => {
                let data = OpPrimitive::pop_num::<u8>(stack)?;
                let words = data.to_string();
                let _ = stack.push_with(words.as_bytes())?;
                let _ = stack.push_with(&((words.len() as u64).to_le_bytes()))?;
            }
            FormatAsm::ATOU8 => todo!(),
            FormatAsm::I128TOA => {
                let data = OpPrimitive::pop_num::<i128>(stack)?;
                let words = data.to_string();
                let _ = stack.push_with(words.as_bytes())?;
                let _ = stack.push_with(&((words.len() as u64).to_le_bytes()))?;
            }
            FormatAsm::ATOI128 => todo!(),
            FormatAsm::I64TOA => {
                let data = OpPrimitive::pop_num::<i64>(stack)?;
                let words = data.to_string();
                let _ = stack.push_with(words.as_bytes())?;
                let _ = stack.push_with(&((words.len() as u64).to_le_bytes()))?;
            }
            FormatAsm::ATOI64 => todo!(),
            FormatAsm::I32TOA => {
                let data = OpPrimitive::pop_num::<i32>(stack)?;
                let words = data.to_string();
                let _ = stack.push_with(words.as_bytes())?;
                let _ = stack.push_with(&((words.len() as u64).to_le_bytes()))?;
            }
            FormatAsm::ATOI32 => todo!(),
            FormatAsm::I16TOA => {
                let data = OpPrimitive::pop_num::<i16>(stack)?;
                let words = data.to_string();
                let _ = stack.push_with(words.as_bytes())?;
                let _ = stack.push_with(&((words.len() as u64).to_le_bytes()))?;
            }
            FormatAsm::ATOI16 => todo!(),
            FormatAsm::I8TOA => {
                let data = OpPrimitive::pop_num::<i8>(stack)?;
                let words = data.to_string();
                let _ = stack.push_with(words.as_bytes())?;
                let _ = stack.push_with(&((words.len() as u64).to_le_bytes()))?;
            }
            FormatAsm::ATOI8 => todo!(),
            FormatAsm::FTOA => {
                let data = OpPrimitive::pop_float(stack)?;
                let words = data.to_string();
                let _ = stack.push_with(words.as_bytes())?;
                let _ = stack.push_with(&((words.len() as u64).to_le_bytes()))?;
            }
            FormatAsm::ATOF => todo!(),
            FormatAsm::BTOA => {
                let data = OpPrimitive::pop_num::<u8>(stack)?;
                let words = if data >= 1 { "true" } else { "false" }.to_string();
                let _ = stack.push_with(words.as_bytes())?;
                let _ = stack.push_with(&((words.len() as u64).to_le_bytes()))?;
            }
            FormatAsm::ATOB => todo!(),
            FormatAsm::CTOA => {
                let data = OpPrimitive::pop_char(stack)?;
                let words = format!("'{0}'", data.to_string());
                let _ = stack.push_with(words.as_bytes())?;
                let _ = stack.push_with(&((words.len() as u64).to_le_bytes()))?;
            }
            FormatAsm::ATOC => todo!(),
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

    use super::*;

    // #[test]
    // fn valid_format_i64() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = f"Hello {10}";
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let result = compile_statement_for_string!(statement);
    //     assert_eq!(result, "Hello 10");
    // }

    // #[test]
    // fn valid_format_u64() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = f"Hello {10u64}";
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let result = compile_statement_for_string!(statement);
    //     assert_eq!(result, "Hello 10");
    // }

    // #[test]
    // fn valid_format_float() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = f"Hello {20.5}";
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let result = compile_statement_for_string!(statement);
    //     assert_eq!(result, "Hello 20.5");
    // }

    // #[test]
    // fn valid_format_bool() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = f"Hello {true}";
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let result = compile_statement_for_string!(statement);
    //     assert_eq!(result, "Hello true");

    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = f"Hello {false}";
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let result = compile_statement_for_string!(statement);
    //     assert_eq!(result, "Hello false");

    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = f"Hello {false} {true}";
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let result = compile_statement_for_string!(statement);
    //     assert_eq!(result, "Hello false true");
    // }

    // #[test]
    // fn valid_format_char() {
    //     let mut statement = Statement::parse(
    //         r##"
    //     let x = f"Hello {'a'}";
    //     "##
    //         .into(),
    //     )
    //     .expect("Parsing should have succeeded")
    //     .1;

    //     let result = compile_statement_for_string!(statement);
    //     assert_eq!(result, "Hello a");
    // }
}

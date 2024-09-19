use std::{arch::x86_64, sync::Arc};

use crate::{semantic::scope::scope::ScopeManager, vm::casm};
use ulid::Ulid;

use crate::{
    ast::{
        expressions::{
            data::{self, Data},
            Atomic, Expression,
        },
        statements::definition,
        utils::{lexem, strings::ID},
    },
    e_static, e_user,
    semantic::{CompatibleWith, EType, Info, MergeType, Metadata, SemanticError, SizeOf, TypeOf},
    vm::{
        allocator::align,
        casm::{
            branch::{Goto, Label},
            Casm, CasmProgram,
        },
        platform::{
            stdlib::{
                io::{IOCasm, PrintCasm},
                StdCasm,
            },
            LibCasm,
        },
        vm::{CodeGenerationError, Printer, RuntimeError},
    },
};

use super::{
    static_types::{
        st_deserialize::{extract_end_u64, extract_u64},
        StaticType,
    },
    type_traits::{IsEnum, OperandMerging, TypeChecking},
    BuildUserType,
};

#[derive(Debug, Clone, PartialEq)]
pub enum UserType {
    Struct(Struct),
    Enum(Enum),
    Union(Union),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    pub id: ID,
    pub fields: Vec<(ID, EType)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    pub id: ID,
    pub values: Vec<ID>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Union {
    pub id: ID,
    pub variants: Vec<(ID, Struct)>,
}

impl CompatibleWith for UserType {
    fn compatible_with(
        &self,
        other: &Self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<(), SemanticError> {
        match (self, other) {
            (UserType::Struct(x), UserType::Struct(y)) => {
                x.compatible_with(y, scope_manager, scope_id)
            }
            (UserType::Enum(x), UserType::Enum(y)) => x.compatible_with(y, scope_manager, scope_id),
            (UserType::Union(x), UserType::Union(y)) => {
                x.compatible_with(y, scope_manager, scope_id)
            }
            _ => Err(SemanticError::IncompatibleTypes),
        }
    }
}

impl BuildUserType for UserType {
    fn build_usertype(
        type_sig: &crate::ast::statements::definition::TypeDef,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Self, SemanticError> {
        match type_sig {
            definition::TypeDef::Struct(value) => Ok(UserType::Struct(Struct::build(
                value,
                scope_manager,
                scope_id,
            )?)),
            definition::TypeDef::Union(value) => Ok(UserType::Union(Union::build(
                value,
                scope_manager,
                scope_id,
            )?)),
            definition::TypeDef::Enum(value) => {
                Ok(UserType::Enum(Enum::build(value, scope_manager, scope_id)?))
            }
        }
    }
}

impl TypeChecking for UserType {}

impl OperandMerging for UserType {}

impl IsEnum for UserType {
    fn is_enum(&self) -> bool {
        match self {
            UserType::Struct(_) => false,
            UserType::Enum(_) => true,
            UserType::Union(_) => false,
        }
    }
}

impl MergeType for UserType {
    fn merge(
        &self,
        other: &Self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError> {
        match (self, other) {
            (UserType::Struct(x), UserType::Struct(y)) => x.merge(y).map(|res| todo!()),
            (UserType::Enum(x), UserType::Enum(y)) => x.merge(y).map(|res| todo!()),
            (UserType::Union(x), UserType::Union(y)) => x.merge(y).map(|res| todo!()),
            _ => Err(SemanticError::IncompatibleTypes),
        }
    }
}

impl SizeOf for UserType {
    fn size_of(&self) -> usize {
        match self {
            UserType::Struct(value) => value.size_of(),
            UserType::Enum(value) => value.size_of(),
            UserType::Union(value) => value.size_of(),
        }
    }
}

impl Printer for UserType {
    fn build_printer(&self, instructions: &mut CasmProgram) -> Result<(), CodeGenerationError> {
        match self {
            UserType::Struct(value) => value.build_printer(instructions),
            UserType::Enum(value) => value.build_printer(instructions),
            UserType::Union(value) => value.build_printer(instructions),
        }
    }
}

impl SizeOf for Struct {
    fn size_of(&self) -> usize {
        self.fields
            .iter()
            .map(|(_, field)| align(field.size_of()))
            .sum()
    }
}
impl SizeOf for Union {
    fn size_of(&self) -> usize {
        8 + self
            .variants
            .iter()
            .map(|(_, variant)| variant.size_of())
            .max()
            .unwrap_or(0)
    }
}
impl SizeOf for Enum {
    fn size_of(&self) -> usize {
        8
    }
}

impl CompatibleWith for Struct {
    fn compatible_with(
        &self,
        other: &Self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<(), SemanticError> {
        if self.id != other.id || self.fields.len() != other.fields.len() {
            return Err(SemanticError::IncompatibleTypes);
        }
        for (self_key, self_field) in self.fields.iter() {
            if let Some((_, other_field)) = other.fields.iter().find(|(id, _)| id == self_key) {
                let _ = self_field.compatible_with(other_field, scope_manager, scope_id)?;
            } else {
                return Err(SemanticError::IncompatibleTypes);
            }
        }
        Ok(())
    }
}

impl CompatibleWith for Union {
    fn compatible_with(
        &self,
        other: &Self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<(), SemanticError> {
        if self.id != other.id || self.variants.len() != other.variants.len() {
            return Err(SemanticError::IncompatibleTypes);
        }
        for (self_variant_key, self_variant) in self.variants.iter() {
            if let Some(other_variant) = other
                .variants
                .iter()
                .find(|(id, _)| id == self_variant_key)
                .map(|(_, field)| field)
            {
                if self_variant.fields.len() != other_variant.fields.len() {
                    return Err(SemanticError::IncompatibleTypes);
                }
                for (self_key, self_field) in self_variant.fields.iter() {
                    if let Some((_, other_field)) =
                        other_variant.fields.iter().find(|(id, _)| id == self_key)
                    {
                        let _ = self_field.compatible_with(other_field, scope_manager, scope_id)?;
                    } else {
                        return Err(SemanticError::IncompatibleTypes);
                    }
                }
            } else {
                return Err(SemanticError::IncompatibleTypes);
            }
        }
        Ok(())
    }
}
impl CompatibleWith for Enum {
    fn compatible_with(
        &self,
        other: &Self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<(), SemanticError> {
        if self.id != other.id || self.values.len() != other.values.len() {
            return Err(SemanticError::IncompatibleTypes);
        }
        for id in &self.values {
            if !other.values.contains(id) {
                return Err(SemanticError::IncompatibleTypes);
            }
        }
        Ok(())
    }
}

impl Struct {
    pub fn build(
        from: &definition::StructDef,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Self, SemanticError> {
        let mut fields = Vec::with_capacity(from.fields.len());
        for (id, field) in &from.fields {
            let field_type = field.type_of(&scope_manager, scope_id)?;
            fields.push((id.clone(), field_type));
        }
        Ok(Self {
            id: from.id.clone(),
            fields,
        })
    }

    pub fn merge(&self, _other: &Self) -> Result<Self, SemanticError> {
        Ok(self.clone())
    }
}

impl Union {
    pub fn build(
        from: &definition::UnionDef,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Self, SemanticError> {
        let mut variants = Vec::new();
        for (id, variant) in &from.variants {
            let mut fields = Vec::with_capacity(variant.len());
            for (id, field) in variant {
                let field_type = field.type_of(&scope_manager, scope_id)?;
                fields.push((id.clone(), field_type));
            }
            variants.push((
                id.clone(),
                Struct {
                    id: id.clone(),
                    fields,
                },
            ));
        }
        Ok(Self {
            id: from.id.clone(),
            variants,
        })
    }
    pub fn merge(&self, _other: &Self) -> Result<Self, SemanticError> {
        Ok(self.clone())
    }
}

impl Enum {
    pub fn build(
        from: &definition::EnumDef,
        _scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<Self, SemanticError> {
        let mut values = Vec::new();
        for id in &from.values {
            values.push(id.clone());
        }
        Ok(Self {
            id: from.id.clone(),
            values,
        })
    }
    pub fn merge(&self, _other: &Self) -> Result<Self, SemanticError> {
        Ok(self.clone())
    }
}

impl Printer for Struct {
    fn build_printer(&self, instructions: &mut CasmProgram) -> Result<(), CodeGenerationError> {
        todo!();
        // instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
        //     PrintCasm::PrintID(self.id.clone()),
        // )))));
        // instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
        //     PrintCasm::StdOutBufOpen,
        // )))));
        // instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
        //     PrintCasm::PrintLexem(lexem::BRA_C),
        // )))));
        // for (idx, (field_name, field_type)) in self.fields.iter().enumerate().rev() {
        //     if idx != self.fields.len() - 1 {
        //         instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
        //             PrintCasm::PrintLexem(lexem::COMA),
        //         )))));
        //     }
        //     let _ = field_type.build_printer(instructions)?;
        //     instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
        //         PrintCasm::PrintLexem(lexem::COLON),
        //     )))));
        //     instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
        //         PrintCasm::PrintID(field_name.clone()),
        //     )))));
        // }
        // instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
        //     PrintCasm::PrintLexem(lexem::BRA_O),
        // )))));
        // instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
        //     PrintCasm::StdOutBufRevFlush,
        // )))));
        Ok(())
    }
}

impl Printer for Union {
    fn build_printer(&self, instructions: &mut CasmProgram) -> Result<(), CodeGenerationError> {
        todo!();
        // let mut cases: Vec<Ulid> = Vec::with_capacity(self.variants.len());
        // let mut dump_data: Vec<Box<[u8]>> = Vec::with_capacity(self.variants.len());

        // let end_label = Label::gen();

        // for (idx, _) in self.variants.iter().enumerate() {
        //     let label: Ulid = Label::gen();
        //     dump_data.push((idx as u64).to_le_bytes().into());
        //     cases.push(label);
        // }

        // let dump_data_label = instructions.push_data(casm::data::Data::Dump {
        //     data: dump_data.into(),
        // });
        // let table_data_label = instructions.push_data(casm::data::Data::Table {
        //     data: cases.clone().into(),
        // });
        // instructions.push(Casm::Switch(BranchTable::Swith {
        //     size: Some(8),
        //     data_label: Some(dump_data_label),
        //     else_label: None,
        // }));
        // instructions.push(Casm::Switch(BranchTable::Table {
        //     table_label: Some(table_data_label),
        //     else_label: None,
        // }));

        // for ((name, value), label) in self.variants.iter().zip(cases) {
        //     instructions.push_label_id(label, format!("print_{}", name).into());
        //     instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
        //         PrintCasm::PrintID(format!("{}::", self.id.clone()).into()),
        //     )))));
        //     let _ = value.build_printer(instructions)?;
        //     instructions.push(Casm::Goto(Goto {
        //         label: Some(end_label),
        //     }));
        // }
        // instructions.push_label_id(end_label, "end_print_enum".to_string().into());
        Ok(())
    }
}

impl Printer for Enum {
    fn build_printer(&self, instructions: &mut CasmProgram) -> Result<(), CodeGenerationError> {
        todo!();
        // let mut cases: Vec<Ulid> = Vec::with_capacity(self.values.len());
        // let mut dump_data: Vec<Box<[u8]>> = Vec::with_capacity(self.values.len());
        // let end_label = Label::gen();

        // for (idx, _) in self.values.iter().enumerate() {
        //     let label: Ulid = Label::gen();
        //     dump_data.push((idx as u64).to_le_bytes().into());
        //     cases.push(label);
        // }

        // let dump_data_label = instructions.push_data(casm::data::Data::Dump {
        //     data: dump_data.into(),
        // });
        // let table_data_label = instructions.push_data(casm::data::Data::Table {
        //     data: cases.clone().into(),
        // });
        // instructions.push(Casm::Switch(BranchTable::Swith {
        //     size: Some(8),
        //     data_label: Some(dump_data_label),
        //     else_label: None,
        // }));
        // instructions.push(Casm::Switch(BranchTable::Table {
        //     table_label: Some(table_data_label),
        //     else_label: None,
        // }));

        // for (name, label) in self.values.iter().zip(cases) {
        //     instructions.push_label_id(label, format!("print_{}", name).into());
        //     instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
        //         PrintCasm::PrintID(format!("{}::{}", self.id.clone(), name.clone()).into()),
        //     )))));
        //     instructions.push(Casm::Goto(Goto {
        //         label: Some(end_label),
        //     }));
        // }

        // instructions.push_label_id(end_label, "end_print_enum".to_string().into());
        Ok(())
    }
}

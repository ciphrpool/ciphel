use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    fmt::format,
    rc::Rc,
};

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
    semantic::{
        CompatibleWith, EType, Either, Info, MergeType, Metadata, SemanticError, SizeOf, TypeOf,
    },
    vm::{
        allocator::align,
        casm::{
            branch::{BranchTable, Goto, Label},
            operation::OpPrimitive,
            Casm, CasmProgram,
        },
        platform::{
            stdlib::{
                io::{IOCasm, PrintCasm},
                StdCasm,
            },
            LibCasm,
        },
        vm::{CodeGenerationError, DeserializeFrom, Printer, RuntimeError},
    },
};

use super::{
    static_types::{
        st_deserialize::{extract_end_u64, extract_u64},
        StaticType,
    },
    type_traits::{GetSubTypes, IsEnum, OperandMerging, TypeChecking},
    BuildUserType, ScopeApi,
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

impl<Scope: ScopeApi> TypeOf<Scope> for Rc<UserType> {
    fn type_of(&self, _scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Scope: super::ScopeApi,
        Self: Sized,
    {
        Ok(Either::User(self.clone()))
    }
}

impl<Scope: ScopeApi> TypeOf<Scope> for UserType {
    fn type_of(&self, _scope: &Ref<Scope>) -> Result<Either<Self, StaticType>, SemanticError>
    where
        Scope: super::ScopeApi,
        Self: Sized,
    {
        Ok(e_user!(self.clone()))
    }
}

impl<Scope: ScopeApi> CompatibleWith<Scope> for UserType {
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        match self {
            UserType::Struct(value) => value.compatible_with(other, scope),
            UserType::Enum(value) => value.compatible_with(other, scope),
            UserType::Union(value) => value.compatible_with(other, scope),
        }
    }
}

impl<Scope: ScopeApi> BuildUserType<Scope> for UserType {
    fn build_usertype(
        type_sig: &crate::ast::statements::definition::TypeDef,
        scope: &Ref<Scope>,
    ) -> Result<Self, SemanticError> {
        match type_sig {
            definition::TypeDef::Struct(value) => {
                Ok(UserType::Struct(Struct::build(value, scope)?))
            }
            definition::TypeDef::Union(value) => Ok(UserType::Union(Union::build(value, scope)?)),
            definition::TypeDef::Enum(value) => Ok(UserType::Enum(Enum::build(value, scope)?)),
        }
    }
}

impl GetSubTypes for UserType {
    fn get_field(&self, field_id: &ID) -> Option<EType> {
        match self {
            UserType::Struct(value) => value
                .fields
                .iter()
                .find(|(id, _)| id == field_id)
                .map(|(_, field)| field.clone()),
            UserType::Enum(_) => None,
            UserType::Union(_) => None,
        }
    }
    fn get_variant(&self, variant: &ID) -> Option<EType> {
        match self {
            UserType::Struct(_) => None,
            UserType::Enum(value) => {
                if value.values.contains(variant) {
                    Some(e_static!(StaticType::Unit))
                } else {
                    None
                }
            }
            UserType::Union(value) => value
                .variants
                .iter()
                .find(|(id, _)| id == variant)
                .map(|(_, field)| field)
                .map(|field| e_user!(UserType::Struct(field.clone()))),
        }
    }
    fn get_fields(&self) -> Option<Vec<(Option<String>, EType)>> {
        match self {
            UserType::Struct(value) => Some(
                value
                    .fields
                    .iter()
                    .map(|(key, val)| (Some(key.clone()), val.clone()))
                    .collect(),
            ),
            UserType::Enum(_) => None,
            UserType::Union(_) => None,
        }
    }

    fn get_field_offset(&self, field_id: &ID) -> Option<usize> {
        match self {
            UserType::Struct(Struct { id: _, fields }) => Some(
                fields
                    .iter()
                    .take_while(|(id, _)| id != field_id)
                    .map(|(_, field)| align(field.size_of()))
                    .sum(),
            ),
            UserType::Enum(_) => None,
            UserType::Union(_) => None,
        }
    }
}

impl TypeChecking for UserType {}

impl<Scope: ScopeApi> OperandMerging<Scope> for UserType {}

impl IsEnum for UserType {
    fn is_enum(&self) -> bool {
        match self {
            UserType::Struct(_) => false,
            UserType::Enum(_) => true,
            UserType::Union(_) => false,
        }
    }
}

impl<Scope: ScopeApi> MergeType<Scope> for UserType {
    fn merge<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<EType, SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        let other_type = other.type_of(&scope)?;
        let Either::User(other_type) = other_type else {
            return Err(SemanticError::IncompatibleTypes);
        };
        match self {
            UserType::Struct(value) => match other_type.as_ref() {
                UserType::Struct(other_type) => value
                    .merge(&other_type)
                    .map(|v| e_user!(UserType::Struct(v))),
                _ => Err(SemanticError::IncompatibleTypes),
            },
            UserType::Enum(value) => match other_type.as_ref() {
                UserType::Enum(other_type) => {
                    value.merge(&other_type).map(|v| e_user!(UserType::Enum(v)))
                }
                _ => Err(SemanticError::IncompatibleTypes),
            },
            UserType::Union(value) => match other_type.as_ref() {
                UserType::Union(other_type) => value
                    .merge(&other_type)
                    .map(|v| e_user!(UserType::Union(v))),
                _ => Err(SemanticError::IncompatibleTypes),
            },
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
    fn build_printer(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
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

impl<Scope: ScopeApi> CompatibleWith<Scope> for Struct {
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        let other_type = other.type_of(&scope)?;
        let Either::User(other_type) = other_type else {
            return Err(SemanticError::IncompatibleTypes);
        };
        let UserType::Struct(other_type) = other_type.as_ref() else {
            return Err(SemanticError::IncompatibleTypes);
        };
        if self.id != other_type.id || self.fields.len() != other_type.fields.len() {
            return Err(SemanticError::IncompatibleTypes);
        }
        for (self_key, self_field) in self.fields.iter() {
            if let Some((_, other_field)) = other_type.fields.iter().find(|(id, _)| id == self_key)
            {
                let _ = self_field.compatible_with(other_field, scope)?;
            } else {
                return Err(SemanticError::IncompatibleTypes);
            }
        }
        Ok(())
    }
}

impl<Scope: ScopeApi> CompatibleWith<Scope> for Union {
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        let other_type = other.type_of(&scope)?;
        let Either::User(other_type) = other_type else {
            return Err(SemanticError::IncompatibleTypes);
        };
        let UserType::Union(other_type) = other_type.as_ref() else {
            return Err(SemanticError::IncompatibleTypes);
        };
        if self.id != other_type.id || self.variants.len() != other_type.variants.len() {
            return Err(SemanticError::IncompatibleTypes);
        }
        for (self_variant_key, self_variant) in self.variants.iter() {
            if let Some(other_variant) = other_type
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
                        let _ = self_field.compatible_with(other_field, scope)?;
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
impl<Scope: ScopeApi> CompatibleWith<Scope> for Enum {
    fn compatible_with<Other>(&self, other: &Other, scope: &Ref<Scope>) -> Result<(), SemanticError>
    where
        Other: TypeOf<Scope>,
    {
        let other_type = other.type_of(&scope)?;
        let Either::User(other_type) = other_type else {
            return Err(SemanticError::IncompatibleTypes);
        };
        let UserType::Enum(other_type) = other_type.as_ref() else {
            return Err(SemanticError::IncompatibleTypes);
        };
        if self.id != other_type.id || self.values.len() != other_type.values.len() {
            return Err(SemanticError::IncompatibleTypes);
        }
        for id in &self.values {
            if !other_type.values.contains(id) {
                return Err(SemanticError::IncompatibleTypes);
            }
        }
        Ok(())
    }
}

impl Struct {
    pub fn build<Scope: ScopeApi>(
        from: &definition::StructDef,
        scope: &Ref<Scope>,
    ) -> Result<Self, SemanticError> {
        let mut fields = Vec::with_capacity(from.fields.len());
        for (id, field) in &from.fields {
            let field_type = field.type_of(&scope)?;
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
    pub fn build<Scope: ScopeApi>(
        from: &definition::UnionDef,
        scope: &Ref<Scope>,
    ) -> Result<Self, SemanticError> {
        let mut variants = Vec::new();
        for (id, variant) in &from.variants {
            let mut fields = Vec::with_capacity(variant.len());
            for (id, field) in variant {
                let field_type = field.type_of(&scope)?;
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
    pub fn build<Scope: ScopeApi>(
        from: &definition::EnumDef,
        _scope: &Ref<Scope>,
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

impl<Scope: ScopeApi> DeserializeFrom<Scope> for UserType {
    type Output = Data<Scope>;

    fn deserialize_from(&self, bytes: &[u8]) -> Result<Self::Output, RuntimeError> {
        match self {
            UserType::Struct(value) => Ok(Data::Struct(value.deserialize_from(bytes)?)),
            UserType::Enum(value) => Ok(Data::Enum(
                <Enum as DeserializeFrom<Scope>>::deserialize_from(value, bytes)?,
            )),
            UserType::Union(value) => Ok(Data::Union(value.deserialize_from(bytes)?)),
        }
    }
}

impl<Scope: ScopeApi> DeserializeFrom<Scope> for Struct {
    type Output = data::Struct<Scope>;

    fn deserialize_from(&self, bytes: &[u8]) -> Result<Self::Output, RuntimeError> {
        let mut offset = 0;
        let mut value = Vec::default();

        for (field_id, field_type) in &self.fields {
            let size = field_type.size_of();
            let aligned_size = align(field_type.size_of());
            if offset + aligned_size > bytes.len() {
                return Err(RuntimeError::Deserialization);
            }
            let data = <EType as DeserializeFrom<Scope>>::deserialize_from(
                &field_type,
                &bytes[offset..offset + size],
            )?;
            value.push((field_id.clone(), Expression::Atomic(Atomic::Data(data))));
            offset += aligned_size;
        }

        Ok(data::Struct {
            id: self.id.clone(),
            fields: value,
            metadata: Metadata {
                info: Rc::new(RefCell::new(Info::Resolved {
                    context: None,
                    signature: Some(Either::User(Rc::new(UserType::Struct(self.clone())))),
                })),
            },
        })
    }
}

impl Printer for Struct {
    fn build_printer(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
            PrintCasm::PrintID(self.id.clone()),
        )))));
        instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
            PrintCasm::StdOutBufOpen,
        )))));
        instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
            PrintCasm::PrintLexem(lexem::BRA_C),
        )))));
        for (idx, (field_name, field_type)) in self.fields.iter().enumerate().rev() {
            if idx != self.fields.len() - 1 {
                instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
                    PrintCasm::PrintLexem(lexem::COMA),
                )))));
            }
            let _ = field_type.build_printer(instructions)?;
            instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
                PrintCasm::PrintLexem(lexem::COLON),
            )))));
            instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
                PrintCasm::PrintID(field_name.to_string()),
            )))));
        }
        instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
            PrintCasm::PrintLexem(lexem::BRA_O),
        )))));
        instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
            PrintCasm::StdOutBufRevFlush,
        )))));
        Ok(())
    }
}

impl<Scope: ScopeApi> DeserializeFrom<Scope> for Union {
    type Output = data::Union<Scope>;

    fn deserialize_from(&self, bytes: &[u8]) -> Result<Self::Output, RuntimeError> {
        let (idx, bytes) = extract_end_u64(bytes)?;
        let Some((variant, struct_type)) = self.variants.get(idx as usize) else {
            return Err(RuntimeError::Deserialization);
        };

        let data: data::Struct<Scope> = struct_type.deserialize_from(bytes)?;

        Ok(data::Union {
            typename: self.id.clone(),
            variant: variant.clone(),
            fields: data.fields,
            metadata: Metadata {
                info: Rc::new(RefCell::new(Info::Resolved {
                    context: None,
                    signature: Some(Either::User(Rc::new(UserType::Union(self.clone())))),
                })),
            },
        })
    }
}

impl Printer for Union {
    fn build_printer(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        let mut table: Vec<(u64, Ulid)> = Vec::with_capacity(self.variants.len());
        let mut cases: Vec<Ulid> = Vec::with_capacity(self.variants.len());
        let end_label = Label::gen();

        for (idx, _) in self.variants.iter().enumerate() {
            let label: Ulid = Label::gen();
            table.push((idx as u64, label));
            cases.push(label);
        }
        instructions.push(Casm::Switch(BranchTable::Table {
            table,
            else_label: None,
        }));
        for ((name, value), label) in self.variants.iter().zip(cases) {
            instructions.push_label_id(label, format!("print_{}", name).into());
            instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
                PrintCasm::PrintID(format!("{}::", self.id.clone()).into()),
            )))));
            let _ = value.build_printer(instructions)?;
            instructions.push(Casm::Goto(Goto {
                label: Some(end_label),
            }));
        }
        instructions.push_label_id(end_label, "end_print_enum".into());
        Ok(())
    }
}

impl<Scope: ScopeApi> DeserializeFrom<Scope> for Enum {
    type Output = data::Enum;

    fn deserialize_from(&self, bytes: &[u8]) -> Result<Self::Output, RuntimeError> {
        let (idx, _bytes) = extract_u64(bytes)?;
        let value = self
            .values
            .get(idx as usize)
            .ok_or(RuntimeError::Deserialization)?;
        Ok(data::Enum {
            typename: self.id.clone(),
            value: value.clone(),
            metadata: Metadata {
                info: Rc::new(RefCell::new(Info::Resolved {
                    context: None,
                    signature: Some(Either::User(Rc::new(UserType::Enum(self.clone())))),
                })),
            },
        })
    }
}

impl Printer for Enum {
    fn build_printer(&self, instructions: &CasmProgram) -> Result<(), CodeGenerationError> {
        let mut table: Vec<(u64, Ulid)> = Vec::with_capacity(self.values.len());
        let mut cases: Vec<Ulid> = Vec::with_capacity(self.values.len());
        let end_label = Label::gen();

        for (idx, _) in self.values.iter().enumerate() {
            let label: Ulid = Label::gen();
            table.push((idx as u64, label));
            cases.push(label);
        }
        instructions.push(Casm::Switch(BranchTable::Table {
            table,
            else_label: None,
        }));
        for (name, label) in self.values.iter().zip(cases) {
            instructions.push_label_id(label, format!("print_{}", name).into());
            instructions.push(Casm::Platform(LibCasm::Std(StdCasm::IO(IOCasm::Print(
                PrintCasm::PrintID(format!("{}::{}", self.id.clone(), name.clone()).into()),
            )))));
            instructions.push(Casm::Goto(Goto {
                label: Some(end_label),
            }));
        }
        instructions.push_label_id(end_label, "end_print_enum".into());
        Ok(())
    }
}

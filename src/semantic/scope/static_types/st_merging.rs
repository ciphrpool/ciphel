use std::cmp::max;

use crate::{
    e_static,
    semantic::{EType, MergeType, SemanticError, TypeOf},
};

use super::{
    AddrType, ClosureType, FnType, MapType, PrimitiveType, RangeType, SliceType, StaticType,
    StrSliceType, StringType, TupleType, VecType,
};
use crate::semantic::scope::scope::ScopeManager;

impl MergeType for StaticType {
    fn merge(
        &self,
        other: &Self,
        scope_manager: &crate::semantic::scope::scope::ScopeManager,
        scope_id: Option<u128>,
    ) -> Result<EType, SemanticError> {
        if self != other {
            return Err(SemanticError::IncompatibleTypes);
        }
        return Ok(EType::Static(self.clone()));
        // let other_type = other.type_of(&scope_manager, scope_id)?;
        // if let StaticType::Unit = self {
        //     return Ok(other_type);
        // }

        // let EType::Static(static_other_type) = &other_type else {
        //     return Err(SemanticError::IncompatibleTypes);
        // };
        // match static_other_type {
        //     Self::Error => return Ok(e_static!(StaticType::Error)),
        //     Self::Unit => return Ok(e_static!(self.clone())),
        //     _ => {}
        // }
        // match self {
        //     StaticType::Primitive(value) => value.merge(other, scope_manager, scope_id),
        //     StaticType::Slice(value) => value.merge(other, scope_manager, scope_id),
        //     StaticType::Vec(value) => value.merge(other, scope_manager, scope_id),
        //     StaticType::StaticFn(value) => value.merge(other, scope_manager, scope_id),
        //     StaticType::Closure(value) => value.merge(other, scope_manager, scope_id),
        //     StaticType::Tuple(value) => value.merge(other, scope_manager, scope_id),
        //     StaticType::Unit => Ok(other_type),
        //     StaticType::Any => Ok(other_type),
        //     StaticType::Error => Ok(e_static!(StaticType::Error)),
        //     StaticType::Address(value) => value.merge(other, scope_manager, scope_id),
        //     StaticType::Map(value) => value.merge(other, scope_manager, scope_id),
        //     StaticType::String(value) => value.merge(other, scope_manager, scope_id),
        //     StaticType::StrSlice(value) => value.merge(other, scope_manager, scope_id),
        //     StaticType::Range(value) => value.merge(other, scope_manager, scope_id),
        // }
    }
}

// impl MergeType for PrimitiveType {
//     fn merge(
//         &self,
//         other: &Self,
//         scope_manager: &crate::semantic::scope::scope::ScopeManager,
//         scope_id: Option<u128>,
//     ) -> Result<EType, SemanticError> {
//         let other_type = other.type_of(&scope_manager, scope_id)?;
//         let EType::Static(other_type) = other_type else {
//             return Err(SemanticError::IncompatibleTypes);
//         };
//         let StaticType::Primitive(other_type) = other_type else {
//             return Err(SemanticError::IncompatibleTypes);
//         };
//         match self {
//             PrimitiveType::Number(left) => match other_type {
//                 PrimitiveType::Number(right) => (left == right)
//                     .then_some(EType::Static(
//                         StaticType::Primitive(PrimitiveType::Number(*left)).into(),
//                     ))
//                     .ok_or(SemanticError::IncompatibleTypes),
//                 _ => Err(SemanticError::IncompatibleTypes),
//             },
//             PrimitiveType::Char => match other_type {
//                 PrimitiveType::Char => Ok(EType::Static(
//                     StaticType::Primitive(PrimitiveType::Char).into(),
//                 )),
//                 _ => Err(SemanticError::IncompatibleTypes),
//             },
//             PrimitiveType::Bool => match other_type {
//                 PrimitiveType::Bool => Ok(EType::Static(
//                     StaticType::Primitive(PrimitiveType::Bool).into(),
//                 )),
//                 _ => Err(SemanticError::IncompatibleTypes),
//             },
//         }
//     }
// }

// impl MergeType for SliceType {
//     fn merge(
//         &self,
//         other: &Self,
//         scope_manager: &crate::semantic::scope::scope::ScopeManager,
//         scope_id: Option<u128>,
//     ) -> Result<EType, SemanticError> {
//         let other_type = other.type_of(&scope_manager, scope_id)?;
//         let EType::Static(other_type) = other_type else {
//             return Err(SemanticError::IncompatibleTypes);
//         };
//         let StaticType::Slice(other_type) = other_type else {
//             return Err(SemanticError::IncompatibleTypes);
//         };

//         if self.size != other_type.size {
//             Err(SemanticError::IncompatibleTypes)
//         } else {
//             let merged = self
//                 .item_type
//                 .merge(other_type.item_type.as_ref(), scope_manager)?;
//             Ok(EType::Static(
//                 StaticType::Slice(SliceType {
//                     size: self.size.clone(),
//                     item_type: Box::new(merged),
//                 })
//                 .into(),
//             ))
//         }
//     }
// }

// impl MergeType for RangeType {
//     fn merge(
//         &self,
//         other: &Self,
//         scope_manager: &crate::semantic::scope::scope::ScopeManager,
//         scope_id: Option<u128>,
//     ) -> Result<EType, SemanticError> {
//         let other_type = other.type_of(&scope_manager, scope_id)?;
//         let EType::Static(other_type) = other_type else {
//             return Err(SemanticError::IncompatibleTypes);
//         };
//         let StaticType::Range(other_type) = other_type else {
//             return Err(SemanticError::IncompatibleTypes);
//         };

//         if self.num != other_type.num || self.inclusive != other_type.inclusive {
//             return Err(SemanticError::IncompatibleTypes);
//         }

//         Ok(EType::Static(
//             StaticType::Range(RangeType {
//                 num: self.num,
//                 inclusive: self.inclusive,
//             })
//             .into(),
//         ))
//     }
// }

// impl MergeType for StringType {
//     fn merge(
//         &self,
//         other: &Self,
//         scope_manager: &crate::semantic::scope::scope::ScopeManager,
//         scope_id: Option<u128>,
//     ) -> Result<EType, SemanticError> {
//         let other_type = other.type_of(&scope_manager, scope_id)?;
//         let EType::Static(other_type) = other_type else {
//             return Err(SemanticError::IncompatibleTypes);
//         };
//         let StaticType::String(_other_type) = other_type else {
//             return Err(SemanticError::IncompatibleTypes);
//         };
//         Ok(e_static!(StaticType::String(StringType())))
//     }
// }

// impl MergeType for StrSliceType {
//     fn merge(
//         &self,
//         other: &Self,
//         scope_manager: &crate::semantic::scope::scope::ScopeManager,
//         scope_id: Option<u128>,
//     ) -> Result<EType, SemanticError> {
//         let other_type = other.type_of(&scope_manager, scope_id)?;
//         if let EType::Static(other_type) = other_type {
//             if let StaticType::StrSlice(StrSliceType { size: other_size }) = other_type {
//                 return Ok(EType::Static(
//                     StaticType::StrSlice(StrSliceType {
//                         // size: self.size + other_size,
//                         size: max(self.size, *other_size),
//                     })
//                     .into(),
//                 ));
//             } else {
//                 return Err(SemanticError::IncompatibleTypes);
//             }
//         } else {
//             return Err(SemanticError::IncompatibleTypes);
//         }
//     }
// }

// impl MergeType for VecType {
//     fn merge(
//         &self,
//         other: &Self,
//         scope_manager: &crate::semantic::scope::scope::ScopeManager,
//         scope_id: Option<u128>,
//     ) -> Result<EType, SemanticError> {
//         let other_type = other.type_of(&scope_manager, scope_id)?;
//         let EType::Static(other_type) = other_type else {
//             return Err(SemanticError::IncompatibleTypes);
//         };
//         let StaticType::Vec(other_type) = other_type else {
//             return Err(SemanticError::IncompatibleTypes);
//         };
//         let merged = self.0.merge(other_type.0, scope_manager)?;
//         Ok(EType::Static(
//             StaticType::Vec(VecType(Box::new(merged))).into(),
//         ))
//     }
// }

// impl MergeType for FnType {
//     fn merge<Other>(
//         &self,
//         _other: &Other,
//         _scope_manager: &crate::semantic::scope::scope::ScopeManager,
//     ) -> Result<EType, SemanticError>
//     where
//         Other: TypeOf,
//     {
//         Err(SemanticError::IncompatibleTypes)
//         // let other_type = other.type_of(&block)?;
//         // let EType::Static(other_type) = other_type else {
//         //     return Err(SemanticError::IncompatibleTypes);
//         // };
//         // let StaticType::StaticFn(other_type) = other_type else {
//         //     return Err(SemanticError::IncompatibleTypes);
//         // };
//         // if self.params.len() != other_type.params.len() {
//         //     return Err(SemanticError::IncompatibleTypes);
//         // }
//         // let mut merged_params = Vec::with_capacity(self.params.len());
//         // for (self_inner, other_inner) in self.params.iter().zip(other_type.params.iter()) {
//         //     let merged = self_inner.merge(other_inner, block, scope_id)?;
//         //     merged_params.push(merged);
//         // }
//         // let merged_ret = self.ret.merge(other_type.ret, block)?;
//         // Ok(EType::Static(
//         //     StaticType::StaticFn(FnType {
//         //         params: merged_params,
//         //         ret: Box::new(merged_ret),
//         //     })
//         //     .into(),
//         // ))
//     }
// }

// impl MergeType for ClosureType {
//     fn merge(
//         &self,
//         other: &Self,
//         scope_manager: &crate::semantic::scope::scope::ScopeManager,
//         scope_id: Option<u128>,
//     ) -> Result<EType, SemanticError> {
//         let other_type = other.type_of(&scope_manager, scope_id)?;
//         let EType::Static(other_type) = other_type else {
//             return Err(SemanticError::IncompatibleTypes);
//         };
//         let StaticType::Closure(other_type) = other_type else {
//             return Err(SemanticError::IncompatibleTypes);
//         };
//         if self.params.len() != other_type.params.len() {
//             return Err(SemanticError::IncompatibleTypes);
//         }
//         let mut merged_params = Vec::with_capacity(self.params.len());
//         for (self_inner, other_inner) in self.params.iter().zip(other_type.params.iter()) {
//             let merged = self_inner.merge(other_inner, scope_manager, scope_id)?;
//             merged_params.push(merged);
//         }
//         let merged_ret = self.ret.merge(other_type.ret, scope_manager)?;

//         if self.closed != other_type.closed {
//             return Err(SemanticError::IncompatibleTypes);
//         }
//         let scope_params_size = max(self.scope_params_size, other_type.scope_params_size);
//         Ok(EType::Static(
//             StaticType::Closure(ClosureType {
//                 params: merged_params,
//                 ret: Box::new(merged_ret),
//                 closed: self.closed,
//                 scope_params_size,
//             })
//             .into(),
//         ))
//     }
// }

// impl MergeType for TupleType {
//     fn merge(
//         &self,
//         other: &Self,
//         scope_manager: &crate::semantic::scope::scope::ScopeManager,
//         scope_id: Option<u128>,
//     ) -> Result<EType, SemanticError> {
//         let other_type = other.type_of(&scope_manager, scope_id)?;
//         let EType::Static(other_type) = other_type else {
//             return Err(SemanticError::IncompatibleTypes);
//         };
//         let StaticType::Tuple(other_type) = other_type else {
//             return Err(SemanticError::IncompatibleTypes);
//         };
//         if self.0.len() != other_type.0.len() {
//             return Err(SemanticError::IncompatibleTypes);
//         }
//         let mut merged_vec = Vec::with_capacity(self.0.len());
//         for (self_inner, other_inner) in self.0.iter().zip(other_type.0.iter()) {
//             let merged = self_inner.merge(other_inner, scope_manager, scope_id)?;
//             merged_vec.push(merged);
//         }
//         Ok(EType::Static(
//             StaticType::Tuple(TupleType(merged_vec)).into(),
//         ))
//     }
// }

// impl MergeType for AddrType {
//     fn merge(
//         &self,
//         other: &Self,
//         scope_manager: &crate::semantic::scope::scope::ScopeManager,
//         scope_id: Option<u128>,
//     ) -> Result<EType, SemanticError> {
//         let other_type = other.type_of(&scope_manager, scope_id)?;
//         let EType::Static(other_type) = other_type else {
//             return Err(SemanticError::IncompatibleTypes);
//         };
//         let StaticType::Address(other_type) = other_type else {
//             return Err(SemanticError::IncompatibleTypes);
//         };
//         let merged = self.0.merge(other_type.0, scope_manager, scope_id)?;
//         Ok(EType::Static(
//             StaticType::Address(AddrType(Box::new(merged))).into(),
//         ))
//     }
// }

// impl MergeType for MapType {
//     fn merge(
//         &self,
//         other: &Self,
//         scope_manager: &crate::semantic::scope::scope::ScopeManager,
//         scope_id: Option<u128>,
//     ) -> Result<EType, SemanticError> {
//         let other_type = other.type_of(&scope_manager, scope_id)?;
//         let EType::Static(other_type) = other_type else {
//             return Err(SemanticError::IncompatibleTypes);
//         };
//         let StaticType::Map(other_type) = other_type else {
//             return Err(SemanticError::IncompatibleTypes);
//         };

//         let merged_key = self.keys_type.merge(other_type.keys_type, scope_manager)?;

//         let merged_value = self
//             .values_type
//             .merge(other_type.values_type, scope_manager)?;
//         Ok(EType::Static(
//             StaticType::Map(MapType {
//                 keys_type: Box::new(merged_key),
//                 values_type: Box::new(merged_value),
//             })
//             .into(),
//         ))
//     }
// }

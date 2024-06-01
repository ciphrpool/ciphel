extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2;
use quote::quote;
mod test;
use syn::{parse_macro_input, Ident, Item, ItemFn};

fn title(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
#[derive(Debug, Clone)]
enum Primitive {
    U128,
    U64,
    U32,
    U16,
    U8,
    I128,
    I64,
    I32,
    I16,
    I8,
    F64,
    CHAR,
    BOOL,
    STRING,
}

fn process_shared_fn(item: &Item) -> proc_macro2::TokenStream {
    if let Item::Fn(ItemFn {
        ref attrs,
        ref vis,
        ref sig,
        ref block,
    }) = item
    {
        let fnstruct_name = title(&format!("{}", &sig.ident));
        let fnstruct_name: Ident =
            syn::parse_str(&fnstruct_name).expect("Failed to parse identifier");
        let fn_name = &sig.ident;
        let fnstruct_def = quote! {
            pub struct #fnstruct_name ();
        };
        let casm_metadata_impl = quote! {
            impl crate::vm::vm::CasmMetadata for #fnstruct_name {
                fn name(&self, stdio: &mut crate::vm::stdio::StdIO, program: &crate::vm::casm::CasmProgram) {
                    let name = stringify!(#fn_name);
                    stdio.push_casm_lib(engine,name);
                }
            }
        };

        let mut params = Vec::new();

        let return_type = match &sig.output {
            syn::ReturnType::Type(_, ty) => match ty.as_ref() {
                syn::Type::Path(path) => {
                    if let Some(segment) = path.path.segments.last() {
                        let ident = &segment.ident;
                        match ident.to_string().as_str() {
                            "u128" => Some((Primitive::U128, false)),
                            "u64" => Some((Primitive::U64, false)),
                            "u32" => Some((Primitive::U32, false)),
                            "u16" => Some((Primitive::U16, false)),
                            "u8" => Some((Primitive::U8, false)),
                            "i128" => Some((Primitive::I128, false)),
                            "i64" => Some((Primitive::I64, false)),
                            "i32" => Some((Primitive::I32, false)),
                            "i16" => Some((Primitive::I16, false)),
                            "i8" => Some((Primitive::I8, false)),
                            "f64" => Some((Primitive::F64, false)),
                            "char" => Some((Primitive::CHAR, false)),
                            "bool" => Some((Primitive::BOOL, false)),
                            "Option" => {
                                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments
                                {
                                    if let Some(arg) = args.args.first() {
                                        if let syn::GenericArgument::Type(inner_ty) = arg {
                                            match inner_ty {
                                                syn::Type::Path(inner_path) => {
                                                    if let Some(inner_segment) =
                                                        inner_path.path.segments.last()
                                                    {
                                                        let inner_ident = &inner_segment.ident;
                                                        match inner_ident.to_string().as_str() {
                                                            "u128" => Some((Primitive::U128, true)),
                                                            "u64" => Some((Primitive::U64, true)),
                                                            "u32" => Some((Primitive::U32, true)),
                                                            "u16" => Some((Primitive::U16, true)),
                                                            "u8" => Some((Primitive::U8, true)),
                                                            "i128" => Some((Primitive::I128, true)),
                                                            "i64" => Some((Primitive::I64, true)),
                                                            "i32" => Some((Primitive::I32, true)),
                                                            "i16" => Some((Primitive::I16, true)),
                                                            "i8" => Some((Primitive::I8, true)),
                                                            "f64" => Some((Primitive::F64, true)),
                                                            "char" => Some((Primitive::CHAR, true)),
                                                            "bool" => Some((Primitive::BOOL, true)),
                                                            _ => panic!("invalid return type"),
                                                        }
                                                    } else {
                                                        panic!("invalid return type")
                                                    }
                                                }
                                                _ => panic!("invalid return type"),
                                            }
                                        } else {
                                            panic!("invalid return type")
                                        }
                                    } else {
                                        panic!("invalid return type")
                                    }
                                } else {
                                    panic!("invalid return type")
                                }
                            }
                            _ => panic!("invalid return type"),
                        }
                    } else {
                        panic!("invalid return type")
                    }
                }
                _ => panic!("invalid return type"),
            },
            _ => None,
        };

        println!("{:?}", return_type);

        for input in sig.inputs.iter().rev() {
            match input {
                syn::FnArg::Typed(syn::PatType { pat, ty, .. }) => {
                    let ident = match pat.as_ref() {
                        syn::Pat::Ident(ident) => &ident.ident,
                        _ => panic!("Unexpected pattern in function argument"),
                    };
                    let ty = ty.as_ref();
                    let ty = match ty {
                        syn::Type::Path(path) => {
                            if let Some(segment) = path.path.segments.last() {
                                let ident = &segment.ident;
                                match ident.to_string().as_str() {
                                    "u128" => Primitive::U128,
                                    "u64" => Primitive::U64,
                                    "u32" => Primitive::U32,
                                    "u16" => Primitive::U16,
                                    "u8" => Primitive::U8,
                                    "i128" => Primitive::I128,
                                    "i64" => Primitive::I64,
                                    "i32" => Primitive::I32,
                                    "i16" => Primitive::I16,
                                    "i8" => Primitive::I8,
                                    "f64" => Primitive::F64,
                                    "char" => Primitive::CHAR,
                                    "bool" => Primitive::BOOL,
                                    "String" => Primitive::STRING,
                                    _ => panic!("Function argument : {} has invalid type", ident),
                                }
                            } else {
                                panic!("Function argument has invalid type")
                            }
                        }
                        _ => panic!("Unsupported type in function argument '{}'", ident),
                    };
                    println!("{ident} : {:?}", ty);
                    params.push((ident, ty));
                }
                _ => {}
            }
        }

        let mut aggregated_params_extract = quote! {};
        for (name, ty) in params {
            let extract = match ty {
                Primitive::U128 => quote! {
                    crate::vm::casm::operation::OpPrimitive::get_num16::<u128>(stack)?
                },
                Primitive::U64 => quote! {
                    crate::vm::casm::operation::OpPrimitive::get_num8::<u64>(stack)?
                },
                Primitive::U32 => quote! {
                    crate::vm::casm::operation::OpPrimitive::get_num4::<u32>(stack)?
                },
                Primitive::U16 => quote! {
                    crate::vm::casm::operation::OpPrimitive::get_num2::<u16>(stack)?
                },
                Primitive::U8 => quote! {
                    crate::vm::casm::operation::OpPrimitive::get_num1::<u8>(stack)?
                },
                Primitive::I128 => quote! {
                    crate::vm::casm::operation::OpPrimitive::get_num16::<i128>(stack)?
                },
                Primitive::I64 => quote! {
                    crate::vm::casm::operation::OpPrimitive::get_num8::<i64>(stack)?
                },
                Primitive::I32 => quote! {
                    crate::vm::casm::operation::OpPrimitive::get_num4::<i32>(stack)?
                },
                Primitive::I16 => quote! {
                    crate::vm::casm::operation::OpPrimitive::get_num2::<i16>(stack)?
                },
                Primitive::I8 => quote! {
                    crate::vm::casm::operation::OpPrimitive::get_num1::<i8>(stack)?
                },
                Primitive::F64 => quote! {
                    crate::vm::casm::operation::OpPrimitive::get_num8::<f64>(stack)?
                },
                Primitive::CHAR => quote! {
                    crate::vm::casm::operation::OpPrimitive::get_char(stack)?
                },
                Primitive::BOOL => quote! {
                    crate::vm::casm::operation::OpPrimitive::get_bool(stack)?
                },
                Primitive::STRING => quote! {
                    {
                        let res = crate::vm::casm::operation::OpPrimitive::get_str_slice(stack)?;
                        res.trim_end_matches(char::from(0)).to_owned()
                    }
                },
            };
            aggregated_params_extract = quote! {
                #aggregated_params_extract
                let #name = #extract;
            };
        }

        let return_value = match return_type {
            Some((ty, false)) => match ty {
                Primitive::U128
                | Primitive::U64
                | Primitive::U32
                | Primitive::U16
                | Primitive::U8
                | Primitive::I128
                | Primitive::I64
                | Primitive::I32
                | Primitive::I16
                | Primitive::I8
                | Primitive::F64 => quote! {
                    let _ = stack.push_with(&result.to_le_bytes()).map_err(|e| e.into())?;
                },
                Primitive::CHAR => quote! {
                    let _ = stack.push_with(&result.to_string().as_bytes()).map_err(|e| e.into())?;
                },
                Primitive::BOOL => quote! {
                    let _ = stack.push_with(&[result as u8]).map_err(|e| e.into())?;

                },
                Primitive::STRING => unreachable!(),
            },
            Some((ty, true)) => match ty {
                Primitive::U128 => quote! {
                    let (result,err) = if result.is_some() {
                        (result.unwrap(),0u8)
                    } else {
                        (0u128,1u8)
                    };
                    let _ = stack.push_with(&result.to_le_bytes()).map_err(|e| e.into())?;
                    let _ = stack.push_with(&err.to_le_bytes()).map_err(|e| e.into())?;
                },
                Primitive::U64 => quote! {
                    let (result,err) = if result.is_some() {
                        (result.unwrap(),0u8)
                    } else {
                        (0u64,1u8)
                    };
                    let _ = stack.push_with(&result.to_le_bytes()).map_err(|e| e.into())?;
                    let _ = stack.push_with(&err.to_le_bytes()).map_err(|e| e.into())?;
                },
                Primitive::U32 => quote! {
                    let (result,err) = if result.is_some() {
                        (result.unwrap(),0u8)
                    } else {
                        (0u32,1u8)
                    };
                    let _ = stack.push_with(&result.to_le_bytes()).map_err(|e| e.into())?;
                    let _ = stack.push_with(&err.to_le_bytes()).map_err(|e| e.into())?;
                },
                Primitive::U16 => quote! {
                    let (result,err) = if result.is_some() {
                        (result.unwrap(),0u8)
                    } else {
                        (0u16,1u8)
                    };
                    let _ = stack.push_with(&result.to_le_bytes()).map_err(|e| e.into())?;
                    let _ = stack.push_with(&err.to_le_bytes()).map_err(|e| e.into())?;
                },
                Primitive::U8 => quote! {
                    let (result,err) = if result.is_some() {
                        (result.unwrap(),0u8)
                    } else {
                        (0u8,1u8)
                    };
                    let _ = stack.push_with(&result.to_le_bytes()).map_err(|e| e.into())?;
                    let _ = stack.push_with(&err.to_le_bytes()).map_err(|e| e.into())?;
                },
                Primitive::I128 => quote! {
                    let (result,err) = if result.is_some() {
                        (result.unwrap(),0u8)
                    } else {
                        (0i128,1u8)
                    };
                    let _ = stack.push_with(&result.to_le_bytes()).map_err(|e| e.into())?;
                    let _ = stack.push_with(&err.to_le_bytes()).map_err(|e| e.into())?;
                },
                Primitive::I64 => quote! {
                    let (result,err) = if result.is_some() {
                        (result.unwrap(),0u8)
                    } else {
                        (0i64,1u8)
                    };
                    let _ = stack.push_with(&result.to_le_bytes()).map_err(|e| e.into())?;
                    let _ = stack.push_with(&err.to_le_bytes()).map_err(|e| e.into())?;
                },
                Primitive::I32 => quote! {
                    let (result,err) = if result.is_some() {
                        (result.unwrap(),0u8)
                    } else {
                        (0i32,1u8)
                    };
                    let _ = stack.push_with(&result.to_le_bytes()).map_err(|e| e.into())?;
                    let _ = stack.push_with(&err.to_le_bytes()).map_err(|e| e.into())?;
                },
                Primitive::I16 => quote! {
                    let (result,err) = if result.is_some() {
                        (result.unwrap(),0u8)
                    } else {
                        (0i16,1u8)
                    };
                    let _ = stack.push_with(&result.to_le_bytes()).map_err(|e| e.into())?;
                    let _ = stack.push_with(&err.to_le_bytes()).map_err(|e| e.into())?;
                },
                Primitive::I8 => quote! {
                    let (result,err) = if result.is_some() {
                        (result.unwrap(),0u8)
                    } else {
                        (0i8,1u8)
                    };
                    let _ = stack.push_with(&result.to_le_bytes()).map_err(|e| e.into())?;
                    let _ = stack.push_with(&err.to_le_bytes()).map_err(|e| e.into())?;
                },
                Primitive::F64 => quote! {
                    let (result,err) = if result.is_some() {
                        (result.unwrap(),0u8)
                    } else {
                        (0f64,1u8)
                    };
                    let _ = stack.push_with(&result.to_le_bytes()).map_err(|e| e.into())?;
                    let _ = stack.push_with(&err.to_le_bytes()).map_err(|e| e.into())?;
                },
                Primitive::CHAR => quote! {
                    let (result,err) = if result.is_some() {
                        (result.unwrap(),0u8)
                    } else {
                        ('\0',1u8)
                    };
                    let _ = stack.push_with(&result.to_string().as_bytes()).map_err(|e| e.into())?;
                },
                Primitive::BOOL => quote! {
                    let (result,err) = if result.is_some() {
                        (result.unwrap(),0u8)
                    } else {
                        (false,1u8)
                    };
                    let _ = stack.push_with(&[result as u8]).map_err(|e| e.into())?;

                },
                Primitive::STRING => unreachable!(),
            },
            None => quote! {},
        };

        let return_type = match &sig.output {
            syn::ReturnType::Type(_, ty) => quote! {
               as #ty
            },
            _ => quote! {},
        };

        let executable_impl = quote! {
            impl crate::vm::vm::Executable for #fnstruct_name {
                fn execute(
                    &self,
                    program: &crate::vm::casm::CasmProgram,
                    stack: &mut crate::vm::allocator::stack::Stack,
                    heap: &mut crate::vm::allocator::heap::Heap,
                    stdio: &mut crate::vm::stdio::StdIO,
                ) -> Result<(), crate::vm::vm::RuntimeError> {
                    #aggregated_params_extract
                    let result = #block #return_type;
                    #return_value
                    Ok(())
                }
            }
        };

        let expanded = quote! {
            #fnstruct_def

            #casm_metadata_impl

            #executable_impl
        };
        expanded
    } else {
        panic!("shared_fn can only be use on functions")
    }
}

#[proc_macro_attribute]
pub fn shared_fn(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the function definition
    let input = parse_macro_input!(item as Item);
    let expanded = process_shared_fn(&input);
    expanded.into()
}

#[proc_macro]
pub fn shared_group(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::Block);
    // Initialize a vector to store the generated code for each function
    let mut generated_code = Vec::new();

    // Iterate over each item in the block
    for item in input.stmts.iter() {
        match item {
            // Process function definitions
            syn::Stmt::Item(item) => {
                let shared_fn = process_shared_fn(&item);
                generated_code.push(shared_fn);
            }
            // Ignore non-item statements
            _ => {}
        }
    }

    let enum_code = {};

    // Concatenate all generated code into a single TokenStream
    let expanded = quote! {
        #enum_code
        #(#generated_code)*
    };

    // Convert the generated Rust code back into a TokenStream
    expanded.into()
}

use proc_macro::TokenStream;
use proc_tools_helper::lang_tr;
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DeriveInput, Expr, Fields, Lit, LitInt, Type};

pub(crate) fn byte_encode_implement(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let fields = if let Data::Struct(data) = input.data {
        match data.fields {
            Fields::Named(fields) => fields.named,
            _ => panic!(lang_tr!(
                cn = "字段类型不支持，仅支持具有命名字段的结构体",
                en = "Only structs with named fields are supported"
            )),
        }
    } else {
        panic!(lang_tr!(cn = "仅支持结构体", en = "Only structs are supported"));
    };

    // 在编译时计算结构体总大小
    let total_size = fields.iter().fold(0, |acc, field| acc + get_type_size(&field.ty));

    // 创建字面量常量
    let total_size_lit = LitInt::new(&total_size.to_string(), name.span());

    // 序列化实现
    let to_bytes_impl = {
        let field_ser = fields.iter().map(|f| {
            let field_name = &f.ident;
            let field_ty = &f.ty;
            let field_size = get_type_size(field_ty);
            let field_size_lit = LitInt::new(&field_size.to_string(), f.ident.span());

            // 检查字段类型是否为 [u8; N]
            if let Type::Array(array_ty) = field_ty {
                if let Type::Path(type_path) = &*array_ty.elem {
                    if type_path.path.is_ident("u8") {
                        return quote! {
                            buffer[pos..pos + #field_size_lit].copy_from_slice(&self.#field_name);
                            pos += #field_size_lit;
                        };
                    }
                }
            }

            // 对于其他类型，使用 to_le_bytes 方法
            quote! {
                let bytes = self.#field_name.to_le_bytes();
                buffer[pos..pos + bytes.len()].copy_from_slice(&bytes);
                pos += bytes.len();
            }
        });

        quote! {
            impl #name {
                pub const SIZE: usize = #total_size_lit;

                pub fn to_bytes(&self) -> [u8; Self::SIZE] {
                    let mut buffer = [0u8; Self::SIZE];
                    let mut pos = 0;
                    #(#field_ser)*
                    buffer
                }
            }
        }
    };

    // 反序列化实现
    let from_bytes_impl = {
        let err_msg = lang_tr!(cn = "切片长度不匹配", en = "slice length mismatch");
        let field_deser = fields.iter().map(|f| {
            let field_name = &f.ident;
            let field_ty = &f.ty;
            let field_size = get_type_size(field_ty);
            let field_size_lit = LitInt::new(&field_size.to_string(), f.ident.span());

            // 检查字段类型是否为 [u8; N]
            if let Type::Array(array_ty) = field_ty {
                if let Type::Path(type_path) = &*array_ty.elem {
                    if type_path.path.is_ident("u8") {
                        return quote! {
                            #field_name: {
                                let mut arr = [0u8; #field_size_lit];
                                arr.copy_from_slice(&bytes[pos..pos + #field_size_lit]);
                                pos += #field_size_lit;
                                arr
                            }
                        };
                    }
                }
            }

            // 对于其他类型，使用 from_le_bytes 方法
            quote! {
                #field_name: {
                    let value = <#field_ty>::from_le_bytes(
                        bytes[pos..pos + #field_size_lit]
                            .try_into()
                            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, #err_msg))?
                    );
                    pos += #field_size_lit;
                    value
                }
            }
        });

        quote! {
            impl #name {
                pub fn from_bytes(bytes: &[u8]) -> Result<Self, std::io::Error> {
                    if bytes.len() != Self::SIZE {
                        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, #err_msg));
                    }
                    let mut pos = 0;
                    Ok(Self {
                        #(#field_deser),*
                    })
                }
            }
        }
    };

    let expanded = quote! {
        #to_bytes_impl
        #from_bytes_impl
    };

    TokenStream::from(expanded)
}

/// 辅助函数：获取类型的大小
fn get_type_size(ty: &Type) -> usize {
    match ty {
        Type::Array(array) => {
            if let Expr::Lit(expr_lit) = &array.len {
                if let Lit::Int(lit_int) = &expr_lit.lit {
                    if let Ok(size) = lit_int.base10_parse::<usize>() {
                        return size;
                    }
                }
            }
            panic!(lang_tr!(cn = "无法获取数组大小", en = "Unable to determine array size"));
        }
        Type::Path(type_path) => {
            let seg = type_path.path.segments.last().unwrap();
            match seg.ident.to_string().as_str() {
                "u8" => 1,
                "u16" => 2,
                "u32" => 4,
                "u64" => 8,
                "u128" => 16,
                "i8" => 1,
                "i16" => 2,
                "i32" => 4,
                "i64" => 8,
                "i128" => 16,
                "f32" => 4,
                "f64" => 8,
                _ => {
                    let msg = lang_tr!(
                        cn = format!("不支持的类型: {}", seg.ident),
                        en = format!("Unsupported type: {}", seg.ident)
                    );
                    panic!("{}", msg)
                }
            }
        }
        _ => panic!(lang_tr!(cn = "不支持的类型", en = "Unsupported type")),
    }
}

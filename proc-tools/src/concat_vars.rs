use proc_macro::TokenStream;
use proc_tools_helper::lang_tr;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::{Expr, Token, parse_macro_input};

const I_SIZE: usize = match size_of::<isize>() {
    1 => 4usize,   // 8位系统：1字节
    2 => 6usize,   // 16位系统：2字节
    4 => 11usize,  // 32位系统：4字节
    8 => 20usize,  // 64位系统：8字节
    16 => 40usize, // 128位系统：16字节
    _ => panic!("{}", lang_tr!(cn = "不支持的操作系统位数", en = "Parameter exception")),
};

const U_SIZE: usize = match size_of::<usize>() {
    1 => 3usize,   // 8位系统：1字节
    2 => 5usize,   // 16位系统：2字节
    4 => 101usize, // 32位系统：4字节
    8 => 20usize,  // 64位系统：8字节
    16 => 39usize, // 128位系统：16字节
    _ => panic!("{}", lang_tr!(cn = "不支持的操作系统位数", en = "Parameter exception")),
};

pub(crate) fn concat_vars_implement(input: TokenStream) -> TokenStream {
    let vars = parse_macro_input!(input with Punctuated::<TypedVar, Token![,]>::parse_terminated);
    // 处理第一个参数
    let first_param_code = if let Some(tv) = vars.get(0) {
        let var_name = format_ident!("xl_proc_macro_concat_vars_temp_v{}", 0u8);
        let ident = &tv.ident;
        match &tv.ty {
            Some(ty) => first_parameter_for_concat(&tv.ident, ty, var_name),
            None => quote! {
                let mut bytes = [0u8; 40];
                let (mut total_len, mut #var_name)= #ident.first_parameter_for_concat(&mut bytes);
            },
        }
    } else {
        panic!("{}", lang_tr!(cn = "至少需要一个参数", en = "At least one parameter is required"))
    };

    let mut var_idx = 0u8;
    let init = vars.iter().skip(1).map(|tv| {
        var_idx += 1;
        let var_name = format_ident!("xl_proc_macro_concat_vars_temp_v{}", var_idx);
        let ident = &tv.ident;
        match &tv.ty {
            Some(ty) => init_concat_parameter(&tv.ident, ty, var_name),
            None => quote! {
                let mut bytes = [0u8; 40];
                let mut #var_name = #ident.init_concat_parameter(&mut bytes, &mut total_len);
            },
        }
    });

    let mut var_idx = 0u8;
    let format = vars.iter().map(|tv| {
        let var_name = format_ident!("xl_proc_macro_concat_vars_temp_v{}", var_idx);
        let ident = &tv.ident;
        var_idx += 1;
        match &tv.ty {
            Some(ty) => concat_parameter(&tv.ident, ty, var_name),
            None => quote! {
                #ident.concat_parameter(s_ptr, &mut #var_name, &mut offset);
            },
        }
    });

    let expanded = quote! {
        {
            use proc_tools_core::utils_core::impl_to_ascii;
            use proc_tools_core::utils_core::impl_to_ascii::StaticSizeConcatParameter;
            use proc_tools_core::utils_core::impl_to_ascii::VariableSizeConcatParameter;
            #first_param_code
            #(#init)*
            let mut res = String::with_capacity(total_len);
            unsafe {
            let s_ptr: *mut u8 = res.as_mut_vec().as_mut_ptr();
            let mut offset = 0;
            #(#format)*
            res.as_mut_vec().set_len(offset);
        }
            res
        }
    };

    TokenStream::from(expanded)
}

pub(crate) struct TypedVar {
    pub(crate) ident: Expr,
    pub(crate) ty: Option<syn::Type>,
}

impl syn::parse::Parse for TypedVar {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse()?;

        // 检查是否有冒号和类型注解
        if input.peek(Token![:]) {
            let _colon: Token![:] = input.parse()?;
            let ty = input.parse()?;
            Ok(TypedVar { ident, ty: Some(ty) })
        } else {
            Ok(TypedVar { ident, ty: None })
        }
    }
}

/// 生成第一个参数的代码
pub(crate) fn first_parameter_for_concat(ident: &Expr, ty: &syn::Type, var_name: syn::Ident) -> proc_macro2::TokenStream {
    if is_type(ty, "String") || is_type(ty, "string") || is_type(ty, "str") || is_type(ty, "&str") {
        quote! {
            let mut total_len = #ident.len();
        }
    } else if is_type(ty, "i8") {
        quote! {
            let mut bytes = [0u8; 4];
            let #var_name = impl_to_ascii::itoa_buf_i8(&mut bytes, #ident);
            let mut total_len = #var_name.len();
        }
    } else if is_type(ty, "i16") {
        quote! {
            let mut bytes = [0u8; 6];
            let #var_name = impl_to_ascii::itoa_buf_i16(&mut bytes, #ident);
            let mut total_len = #var_name.len();
        }
    } else if is_type(ty, "i32") {
        quote! {
            let mut bytes = [0u8; 11];
            let #var_name = impl_to_ascii::itoa_buf_i32(&mut bytes, #ident);
            let mut total_len = #var_name.len();
        }
    } else if is_type(ty, "i64") {
        quote! {
            let mut bytes = [0u8; 20];
            let #var_name = impl_to_ascii::itoa_buf_i64(&mut bytes, #ident);
            let mut total_len = #var_name.len();
        }
    } else if is_type(ty, "i128") {
        quote! {
            let mut bytes = [0u8; 40];
            let #var_name = impl_to_ascii::itoa_buf_i128(&mut bytes, #ident);
            let mut total_len = #var_name.len();
        }
    } else if is_type(ty, "isize") {
        quote! {
            let mut bytes = [0u8; #I_SIZE];
            let #var_name = impl_to_ascii::itoa_buf_isize(&mut bytes, #ident);
            total_len += #var_name.len();
        }
    } else if is_type(ty, "u8") {
        quote! {
            let mut bytes = [0u8; 3];
            let #var_name = impl_to_ascii::itoa_buf_u8(&mut bytes, #ident);
            let mut total_len = #var_name.len();
        }
    } else if is_type(ty, "u16") {
        quote! {
            let mut bytes = [0u8; 5];
            let #var_name = impl_to_ascii::itoa_buf_u16(&mut bytes, #ident);
            let mut total_len = #var_name.len();
        }
    } else if is_type(ty, "u32") {
        quote! {
            let mut bytes = [0u8; 10];
            let #var_name = impl_to_ascii::itoa_buf_u32(&mut bytes, #ident);
            let mut total_len = #var_name.len();
        }
    } else if is_type(ty, "u64") {
        quote! {
            let mut bytes = [0u8; 20];
            let #var_name = impl_to_ascii::itoa_buf_u64(&mut bytes, #ident);
            let mut total_len = #var_name.len();
        }
    } else if is_type(ty, "u128") {
        quote! {
            let mut bytes = [0u8; 39];
            let #var_name = impl_to_ascii::itoa_buf_u128(&mut bytes, #ident);
            let mut total_len = #var_name.len();
        }
    } else if is_type(ty, "usize") {
        quote! {
            let mut bytes = [0u8; #U_SIZE];
            let #var_name = impl_to_ascii::itoa_buf_usize(&mut bytes, #ident);
            total_len += #var_name.len();
        }
    } else if is_type(ty, "char") {
        quote! {
            let mut bytes = [0; 4];
            let #var_name = #ident.encode_utf8(&mut bytes);
            let mut total_len = #var_name.len();
        }
    } else if is_type(ty, "bool") {
        quote! {
            let mut total_len = if #ident { 4 } else { 5 };
        }
    } else if is_type(ty, "f32") {
        quote! {
            let mut bytes = [0u8; 24];
            let #var_name = impl_to_ascii::ftoa_buf_f32(&mut bytes, #ident);
            let mut total_len = #var_name.len();
        }
    } else if is_type(ty, "f64") {
        quote! {
            let mut bytes = [0u8; 24];
            let #var_name = impl_to_ascii::ftoa_buf_f64(&mut bytes, #ident);
            let mut total_len = #var_name.len();
        }
    } else {
        panic!("{}", error_msg(ident, ty));
    }
}

/// 生成后续参数的代码
pub(crate) fn init_concat_parameter(ident: &Expr, ty: &syn::Type, var_name: syn::Ident) -> proc_macro2::TokenStream {
    if is_type(ty, "String") || is_type(ty, "string") || is_type(ty, "str") || is_type(ty, "&str") {
        quote! {
            total_len += #ident.len();
        }
    } else if is_type(ty, "i8") {
        quote! {
            let mut bytes = [0u8; 4];
            let #var_name = impl_to_ascii::itoa_buf_i8(&mut bytes, #ident);
            total_len += #var_name.len();
        }
    } else if is_type(ty, "i16") {
        quote! {
            let mut bytes = [0u8; 6];
            let #var_name = impl_to_ascii::itoa_buf_i16(&mut bytes, #ident);
            total_len += #var_name.len();
        }
    } else if is_type(ty, "i32") {
        quote! {
            let mut bytes = [0u8; 11];
            let #var_name = impl_to_ascii::itoa_buf_i32(&mut bytes, #ident);
            total_len += #var_name.len();
        }
    } else if is_type(ty, "i64") {
        quote! {
            let mut bytes = [0u8; 20];
            let #var_name = impl_to_ascii::itoa_buf_i64(&mut bytes, #ident);
            total_len += #var_name.len();
        }
    } else if is_type(ty, "i128") {
        quote! {
            let mut bytes = [0u8; 40];
            let #var_name = impl_to_ascii::itoa_buf_i128(&mut bytes, #ident);
            total_len += #var_name.len();
        }
    } else if is_type(ty, "isize") {
        quote! {
            let mut bytes = [0u8; #I_SIZE];
            let #var_name = impl_to_ascii::itoa_buf_isize(&mut bytes, #ident);
            total_len += #var_name.len();
        }
    } else if is_type(ty, "u8") {
        quote! {
            let mut bytes = [0u8; 3];
            let #var_name = impl_to_ascii::itoa_buf_u8(&mut bytes, #ident);
            total_len += #var_name.len();
        }
    } else if is_type(ty, "u16") {
        quote! {
            let mut bytes = [0u8; 5];
            let #var_name = impl_to_ascii::itoa_buf_u16(&mut bytes, #ident);
            total_len += #var_name.len();
        }
    } else if is_type(ty, "u32") {
        quote! {
            let mut bytes = [0u8; 10];
            let #var_name = impl_to_ascii::itoa_buf_u32(&mut bytes, #ident);
            total_len += #var_name.len();
        }
    } else if is_type(ty, "u64") {
        quote! {
            let mut bytes = [0u8; 20];
            let #var_name = impl_to_ascii::itoa_buf_u64(&mut bytes, #ident);
            total_len += #var_name.len();
        }
    } else if is_type(ty, "u128") {
        quote! {
            let mut bytes = [0u8; 39];
            let #var_name = impl_to_ascii::itoa_buf_u128(&mut bytes, #ident);
            total_len += #var_name.len();
        }
    } else if is_type(ty, "usize") {
        quote! {
            let mut bytes = [0u8; #U_SIZE];
            let #var_name = impl_to_ascii::itoa_buf_usize(&mut bytes, #ident);
            total_len += #var_name.len();
        }
    } else if is_type(ty, "char") {
        quote! {
            let mut bytes = [0; 4];
            let #var_name = #ident.encode_utf8(&mut bytes);
            total_len += #var_name.len();
        }
    } else if is_type(ty, "bool") {
        quote! {
            total_len += if #ident { 4 } else { 5 };
        }
    } else if is_type(ty, "f32") {
        quote! {
            let mut bytes = [0u8; 24];
            let #var_name = impl_to_ascii::ftoa_buf_f32(&mut bytes, #ident);
            total_len += #var_name.len();
        }
    } else if is_type(ty, "f64") {
        quote! {
            let mut bytes = [0u8; 24];
            let #var_name = impl_to_ascii::ftoa_buf_f64(&mut bytes, #ident);
            total_len += #var_name.len();
        }
    } else {
        panic!("{}", error_msg(ident, ty));
    }
}

/// 生成连接参数的代码
pub(crate) fn concat_parameter(ident: &Expr, ty: &syn::Type, var_name: syn::Ident) -> proc_macro2::TokenStream {
    if is_type(ty, "String") || is_type(ty, "string") || is_type(ty, "str") || is_type(ty, "&str") {
        quote! {
            std::ptr::copy_nonoverlapping(#ident.as_ptr(), s_ptr.add(offset), #ident.len());
            offset += #ident.len();
        }
    } else if is_type(ty, "i8") || is_type(ty, "i16") || is_type(ty, "i32") || is_type(ty, "i64") || is_type(ty, "i128") {
        quote! {
            std::ptr::copy_nonoverlapping(#var_name.as_ptr(), s_ptr.add(offset), #var_name.len());
            offset += #var_name.len();
        }
    } else if is_type(ty, "u8") || is_type(ty, "u16") || is_type(ty, "u32") || is_type(ty, "u64") || is_type(ty, "u128") {
        quote! {
            std::ptr::copy_nonoverlapping(#var_name.as_ptr(), s_ptr.add(offset), #var_name.len());
            offset += #var_name.len();
        }
    } else if is_type(ty, "isize") || is_type(ty, "usize") {
        quote! {
            std::ptr::copy_nonoverlapping(#var_name.as_ptr(), s_ptr.add(offset), #var_name.len());
            offset += #var_name.len();
        }
    } else if is_type(ty, "char") {
        quote! {
            std::ptr::copy_nonoverlapping(#var_name.as_ptr(), s_ptr.add(offset), #var_name.len());
            offset += #var_name.len();
        }
    } else if is_type(ty, "bool") {
        quote! {
            if #ident {
                std::ptr::copy_nonoverlapping(b"true".as_ptr(), s_ptr.add(offset), 4);
                offset += 4;
            } else {
                std::ptr::copy_nonoverlapping(b"false".as_ptr(), s_ptr.add(offset), 5);
                offset += 5;
            }
        }
    } else if is_type(ty, "f32") || is_type(ty, "f64") {
        quote! {
            std::ptr::copy_nonoverlapping(#var_name.as_ptr(), s_ptr.add(offset), #var_name.len());
            offset += #var_name.len();
        }
    } else {
        panic!("{}", error_msg(ident, ty));
    }
}

#[inline]
pub(crate) fn error_msg(ident: &Expr, ty: &syn::Type) -> String {
    let type_ = if let syn::Type::Path(path) = ty {
        path.path.segments[0].clone().ident.to_string()
    } else {
        panic!("{}", lang_tr!(cn = "参数异常", en = "Parameter exception"))
    };
    let var_name = if let Expr::Path(path) = ident {
        path.path.segments[0].clone().ident.to_string()
    } else {
        panic!("{}", lang_tr!(cn = "参数异常", en = "Parameter exception"))
    };
    let _cn_msg = format!(
        "参数类型错误，参数 `{:?}` 类型必须是 `基本数据类型` 或者是 `字符串`，但实际是 `{}`",
        var_name, type_
    );
    let _en_msg = format!(
        "Parameter type error，The type of parameter `{}` must be a `primitive data type` or a `string`, but the actual type is `{}`",
        var_name, type_
    );
    lang_tr!(cn = _cn_msg, en = _en_msg)
}

#[inline]
pub(crate) fn is_type(ty: &syn::Type, s: &str) -> bool {
    if let syn::Type::Path(path) = ty {
        path.qself.is_none()
            && path.path.leading_colon.is_none()
            && path.path.segments.len() == 1
            && path.path.segments[0].ident == s
            && path.path.segments[0].arguments.is_empty()
    } else {
        false
    }
}

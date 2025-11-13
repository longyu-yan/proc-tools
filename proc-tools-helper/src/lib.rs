extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::Expr;
use syn::parse::Parse;
use syn::parse_macro_input;
use syn::{Error, Ident, Token, parse::ParseStream};

#[cfg(any(feature = "def_cn"))]
fn get_def_lang() -> Box<str> {
    Box::from("cn")
}
#[cfg(any(feature = "def_en"))]
fn get_def_lang() -> Box<str> {
    Box::from("en")
}

// 解析参数结构体
struct Args {
    cn: Expr,
    en: Expr,
}

/// 多语言字符串翻译宏实现
/// - 根据设置的默认语言选择对应的中英文字符串。
/// - 这是一个过程宏，用于在编译时根据语言设置选择不同的字符串常量。
///
/// # 参数
/// - `input`: 宏输入的TokenStream，包含中英文字符串配置
///
/// # 返回值
/// - `TokenStream`: 根据默认语言选择的字符串对应的TokenStream
///
/// # 错误类型
/// - 如果未设置默认语言或设置了多个默认语言，会触发panic
/// - 如果输入参数不符合语法要求，会在编译时报错
///
/// # 示例
/// ```
/// use proc_tools_helper::lang_tr;
///
/// let message = lang_tr!(cn = "你好世界", en = "Hello World");
/// // 根据设置语言，message 会是 "你好世界" 或 "Hello World"
/// ```
#[proc_macro]
pub fn lang_tr(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as Args);
    let cn = &args.cn;
    let en = &args.en;

    let result = match get_def_lang().as_ref() {
        "cn" => quote! { #cn },
        "en" => quote! { #en },
        _ => panic!("必须且只能启用一项默认语言"),
    };

    TokenStream::from(result)
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut cn = None;
        let mut en = None;

        // 解析所有键值对（支持任意顺序，逗号分隔）
        while !input.is_empty() {
            let key = input.parse::<Ident>()?;
            input.parse::<Token![=]>()?;
            let expr = input.parse::<Expr>()?;

            match key.to_string().as_str() {
                "cn" => {
                    if cn.is_some() {
                        return Err(Error::new_spanned(key, "Duplicate 'cn' key"));
                    }
                    cn = Some(expr);
                }
                "en" => {
                    if en.is_some() {
                        return Err(Error::new_spanned(key, "Duplicate 'en' key"));
                    }
                    en = Some(expr);
                }
                _ => return Err(Error::new_spanned(key, "Expected key 'cn' or 'en'")),
            }

            // 如果还有逗号，继续解析下一个
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            } else {
                break;
            }
        }

        // 确保两个键都存在（否则报错）
        Ok(Args {
            cn: cn.ok_or_else(|| panic!("中文：cn值获取失败，缺少必要参数")).unwrap(),
            en: en
                .ok_or_else(|| panic!("English: en value retrieval failed, missing necessary parameters"))
                .unwrap(),
        })
    }
}

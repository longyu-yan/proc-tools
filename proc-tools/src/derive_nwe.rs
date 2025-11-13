use proc_macro::{Delimiter, TokenStream, TokenTree};
use proc_tools_helper::lang_tr;

pub(crate) fn derive_new_implement(input: TokenStream) -> TokenStream {
    let mut struct_name = None;
    let mut fields = Vec::new();

    // 解析结构体定义
    let mut tokens = input.into_iter();
    while let Some(token) = tokens.next() {
        if let TokenTree::Ident(ident) = &token {
            if ident.to_string() == "struct" {
                if let Some(TokenTree::Ident(name)) = tokens.next() {
                    struct_name = Some(name.to_string());
                }
            }
        } else if let TokenTree::Group(group) = token {
            if group.delimiter() == Delimiter::Brace {
                // 解析字段
                let mut field_tokens = group.stream().into_iter();
                let mut current_field = None;

                while let Some(token) = field_tokens.next() {
                    if let TokenTree::Ident(ident) = token {
                        current_field = Some(ident.to_string());
                    } else if let TokenTree::Punct(punct) = &token {
                        if punct.as_char() == ':' {
                            // 开始解析类型
                            let mut type_tokens = Vec::new();
                            while let Some(token) = field_tokens.next() {
                                if let TokenTree::Punct(punct) = &token {
                                    if punct.as_char() == ',' {
                                        break;
                                    }
                                }
                                type_tokens.push(token.to_string());
                            }

                            if let Some(field) = current_field.take() {
                                let field_type = type_tokens.join(" ");
                                fields.push((field, field_type));
                            }
                        }
                    }
                }
                break;
            }
        }
    }

    if let Some(struct_name) = struct_name {
        // 生成 new 函数
        let mut code = format!("impl {} {{\n", struct_name);
        code.push_str("    pub fn new(");

        // 添加参数
        for (i, (name, ty)) in fields.iter().enumerate() {
            if i > 0 {
                code.push_str(", ");
            }
            code.push_str(&format!("{}: {}", name, ty));
        }

        code.push_str(") -> Self {\n");
        code.push_str("        Self {\n");

        // 添加字段初始化
        for (name, _) in &fields {
            code.push_str(&format!("            {},\n", name));
        }

        code.push_str("        }\n");
        code.push_str("    }\n");
        code.push_str("}\n");

        code.parse().unwrap_or_else(|_| {
            panic!("{}", lang_tr!(cn = "解析生成的代码失败", en = "Failed to parse generated code"))
        })
    } else {
        panic!("{}", lang_tr!(cn = "解析生成的代码失败", en = "Failed to parse generated code"))
    }
}
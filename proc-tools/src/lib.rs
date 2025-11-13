mod concat_vars;
mod derive_byte_encode;
mod derive_nwe;

use crate::concat_vars::concat_vars_implement;
use crate::derive_byte_encode::byte_encode_implement;
use crate::derive_nwe::derive_new_implement;
use proc_macro::TokenStream;

#[allow(dead_code)]
#[cfg(any(feature = "lang-cn", all(not(feature = "lang-cn"), not(feature = "lang-en"))))]
fn lang_verify() {}

#[allow(dead_code)]
#[cfg(any(feature = "lang-en"))]
fn lang_verify() {}

// 防止两个特性同时启用（编译时报错）
#[cfg(all(feature = "lang-cn", feature = "lang-en"))]
compile_error!("Cannot enable both 'lang_cn' and 'lang_en' features simultaneously\n不能同时启用 'lang_cn' 和 'lang_en' 特性");

/// 高效连接多个变量的过程宏
/// - 支持将多个整数型、浮点型、布尔型、字符和字符串连接为字符串
/// - 通过预计算所需内存大小并使用直接内存操作来避免不必要的内存分配和拷贝
/// - 对浮点型数据（`f32`,`f64`），格式化数据在大多数时候和标准库的 `format!` 没有区别
/// - 在极端情况下的浮点型，如：`f32::MIN`，与标准库的 `format!` 生成的字符串是不同的，`concat_vars`会以科学计数法的方式生成字符串
/// - 在 `opt-level = 3` 优化情况下，性能比标准库的 `format!` 宏提高 2-3 倍
/// - 在 `opt-level = "z"`，生成的代码更小，性能和内存占用依然优于 `format!` 宏
///
/// # 参数
/// - 支持的类型包括基本类型（整数、浮点数、布尔值等）和字符串
///
/// # 返回值
/// - 返回一个 `String`，包含所有参数连接后的结果
///
/// # 性能说明
/// 此宏通过以下方式优化性能：
/// 1. 预计算所有参数的总长度
/// 2. 一次性分配足够的内存
/// 3. 使用指针操作直接写入内存，避免中间字符串创建
///
/// # 注意事项
/// - 必须至少提供一个参数
/// - 宏内部使用不安全代码，但对外提供安全接口
/// - 需要依赖库：`proc_tools_core`
///
/// # 示例
/// ```
/// use proc_tools::concat_vars;
/// let name = "Alice";
/// let age = 30;
/// let score = 95.5;
///
/// /// 第一种方式：直接使用变量直接连接，简单，快速，但是会占用更多内存
/// /// 因为宏无法稳定获取变量的数据类型，默认将会给非字符串数据类型全部分配 40 字节内存空间
/// /// 对i8，u8，char等数据类型会浪费更多内存空间
/// let result = concat_vars!(name, age, score);
/// assert_eq!(result, "Alice3095.5");
///
/// /// 第二种方式：指定变量的数据类型，宏会根据数据类型指定对应大小，例如：对 i32 分配 11 字节内存空间
/// /// 极端情况，可用的内存较小，建议使用第二种方式
/// /// 内存够用情况，两种方式性能相差不大，不需要太纠结
/// let result = concat_vars!(name: String, age: i32, score: f64);
/// assert_eq!(result, "Alice3095.5");
/// ```
#[proc_macro]
pub fn concat_vars(input: TokenStream) -> TokenStream {
    concat_vars_implement(input)
}

/// 自动为结构体生成 `new` 构造函数
/// - 该构造函数接收所有字段作为参数并返回结构体实例。
/// - 生成的函数参数顺序与结构体字段声明顺序一致
/// - 提供编译时类型安全检查
///
/// # 限制
/// - 不支持泛型参数
/// - 不支持生命周期参数
/// - 不支持字段的默认值或可选参数
/// - 不支持文档注释的保留
///
/// # 示例
/// 对于以下结构体：
/// ```ignore
/// #[derive_new]
/// struct Point {
///     x: f64,
///     y: f64,
/// }
/// ```
///
/// 宏将生成：
/// ```ignore
/// impl Point {
///     pub fn new(x: f64, y: f64) -> Self {
///         Self { x, y }
///     }
/// }
/// ```
#[proc_macro_derive(New)]
pub fn derive_new(input: TokenStream) -> TokenStream {
    derive_new_implement(input)
}

/// 为结构体自动派生固定大小字节编码/解码实现的过程宏
/// - 此宏可以为包含固定大小字段的结构体自动生成字节序列化和反序列化方法。
/// - 生成的实现使用小端字节序（little-endian）进行编码，适用于二进制协议和文件格式。
///
/// # 特性
/// - 自动生成 `to_bytes()` 方法将结构体序列化为字节数组
/// - 自动生成 `from_bytes()` 方法从字节数组反序列化结构体
/// - 提供 `SIZE` 常量表示结构体的固定字节大小
/// - 支持基本数值类型和固定大小数组的编码
/// - 编译时计算结构体大小，无运行时开销
///
/// # 支持的类型
/// - 所有整数类型 (`i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64`, `i128`, `u128`)
/// - 所有浮点类型 (`f32`, `f64`)
/// - 固定大小的字节数组 (`[u8; N]`)
/// - 布尔类型 (`bool`) - 编码为 `u8` (0/1)
///
/// # 错误处理
/// - `from_bytes` 方法可能返回 `std::io::Error` 错误
/// - 输入字节长度必须精确匹配 `SIZE` 常量
/// - 所有字段必须能正确反序列化，否则返回错误
///
/// # 示例
/// ```ignore
/// #[derive(ByteEncode)]
/// struct PacketHeader {
///     version: u8,
///     packet_type: u16,
///     length: u32,
///     checksum: [u8; 4],
/// }
///
/// let header = PacketHeader {
///     version: 1,
///     packet_type: 100,
///     length: 1024,
///     checksum: [0x12, 0x34, 0x56, 0x78],
/// };
///
/// // 序列化为字节数组
/// let bytes = header.to_bytes();
/// assert_eq!(bytes.len(), PacketHeader::SIZE);
///
/// // 从字节数组反序列化
/// let decoded = PacketHeader::from_bytes(&bytes).unwrap();
/// ```
#[proc_macro_derive(ByteEncode)]
pub fn derive_byte_encode(input: TokenStream) -> TokenStream {
    byte_encode_implement(input)
}

use crate::float2str::pretty::{format32, format64};

const I82STR_LEN: usize = 4;
const I162STR_LEN: usize = 6;
const I322STR_LEN: usize = 11;
const I642STR_LEN: usize = 20;
const I1282STR_LEN: usize = 40;
const U82STR_LEN: usize = 3;
const U162STR_LEN: usize = 5;
const U322STR_LEN: usize = 10;
const U642STR_LEN: usize = 20;
const U1282STR_LEN: usize = 39;
const F2STR_LEN: usize = 24;

const ISIZE2STR_SIZE: usize = match size_of::<isize>() {
    1 => 4usize,   // 8位系统：1字节
    2 => 6usize,   // 16位系统：2字节
    4 => 11usize,  // 32位系统：4字节
    8 => 20usize,  // 64位系统：8字节
    16 => 40usize, // 128位系统：16字节
    _ => panic!("{}", "不支持的操作系统位数"),
};
const USIZE2STR_LEN: usize = match size_of::<usize>() {
    1 => 3usize,   // 8位系统：1字节
    2 => 5usize,   // 16位系统：2字节
    4 => 101usize, // 32位系统：4字节
    8 => 20usize,  // 64位系统：8字节
    16 => 39usize, // 128位系统：16字节
    _ => panic!("{}", "不支持的操作系统位数"),
};
const IUSIZE_MIN: &[u8] = match size_of::<isize>() {
    1 => b"-128",                                      // 8位系统：1字节
    2 => b"-32768",                                    // 16位系统：2字节
    4 => b"-2147483648",                               // 32位系统：4字节
    8 => b"-9223372036854775808",                      // 64位系统：8字节
    16 => b"-170141183460469231731687303715884105728", // 128位系统：16字节
    _ => panic!("{}", "不支持的操作系统位数"),
};

macro_rules! impl_itoa_signed {
    ($func_name:ident, $ty:ty, $buf_size:expr, $min_str:expr) => {
        #[inline]
        pub fn $func_name(i_buffer: &mut [u8; $buf_size], mut i: $ty) -> &[u8] {
            let flag: bool = if i == <$ty>::MIN {
                return $min_str;
            } else if i < 0 {
                i = -i;
                true
            } else {
                false
            };
            if i == 0 {
                &[b'0']
            } else {
                let mut idx = i_buffer.len();
                while i > 0 {
                    idx -= 1;
                    i_buffer[idx] = b'0' + (i % 10) as u8;
                    i /= 10;
                }
                if flag {
                    idx -= 1;
                    i_buffer[idx] = b'-';
                    &i_buffer[idx..]
                } else {
                    &i_buffer[idx..]
                }
            }
        }
    };
}
impl_itoa_signed!(itoa_buf_i8, i8, I82STR_LEN, b"-128");
impl_itoa_signed!(itoa_buf_i16, i16, I162STR_LEN, b"-32768");
impl_itoa_signed!(itoa_buf_i32, i32, I322STR_LEN, b"-2147483648");
impl_itoa_signed!(itoa_buf_i64, i64, I642STR_LEN, b"-9223372036854775808");
impl_itoa_signed!(itoa_buf_i128, i128, I1282STR_LEN, b"-170141183460469231731687303715884105728");
impl_itoa_signed!(itoa_buf_isize, isize, ISIZE2STR_SIZE, IUSIZE_MIN);

macro_rules! impl_itoa_unsigned {
    ($func_name:ident, $ty:ty, $buf_size:expr) => {
        #[inline]
        pub fn $func_name(i_buffer: &mut [u8; $buf_size], mut i: $ty) -> &[u8] {
            if i == 0 {
                &[b'0']
            } else {
                let mut idx = i_buffer.len();
                while i > 0 {
                    idx -= 1;
                    i_buffer[idx] = b'0' + (i % 10) as u8;
                    i /= 10;
                }
                &i_buffer[idx..]
            }
        }
    };
}
impl_itoa_unsigned!(itoa_buf_u8, u8, U82STR_LEN);
impl_itoa_unsigned!(itoa_buf_u16, u16, U162STR_LEN);
impl_itoa_unsigned!(itoa_buf_u32, u32, U322STR_LEN);
impl_itoa_unsigned!(itoa_buf_u64, u64, U642STR_LEN);
impl_itoa_unsigned!(itoa_buf_u128, u128, U1282STR_LEN);
impl_itoa_unsigned!(itoa_buf_usize, usize, USIZE2STR_LEN);

/// 将 f32 浮点数转换为字符串并写入缓冲区
/// - 该函数将浮点数转换为字符串表示形式，支持特殊值（NAN、INFINITY等）的处理，
///
/// # 参数
/// - `buf`: 用于存储结果的缓冲区，必须至少24字节长度
/// - `f`: 要转换的 f32 浮点数
///
/// # 返回值
/// - `&[u8]`: 指向缓冲区中转换结果的字节切片引用
///
/// # 注意事项
/// - 缓冲区必须足够大（至少24字节）以避免缓冲区溢出
/// - 对于特殊浮点值（NAN、无穷大）返回预定义的字符串
/// - 内部使用 unsafe 代码进行高效格式化，但对外接口是安全的
///
/// # 示例
/// ```
/// use proc_tools_core::utils_core::impl_to_ascii::ftoa_buf_f32;
/// let mut buf = [0u8; 24];
/// let result = ftoa_buf_f32(&mut buf, 3.14f32);
/// assert_eq!(std::str::from_utf8(result).unwrap(), "3.14");
///
/// let mut buf2 = [0u8; 24];
/// let result2 = ftoa_buf_f32(&mut buf2, f32::NAN);
/// assert_eq!(std::str::from_utf8(result2).unwrap(), "NAN");
/// ```
#[inline]
pub fn ftoa_buf_f32(buf: &mut [u8; 24], f: f32) -> &[u8] {
    let bits = f.to_bits();
    if bits & 0x7f800000 == 0x7f800000 {
        if bits & 0x007fffff != 0 {
            b"NAN"
        } else if bits & 0x80000000 != 0 {
            b"NEG_INFINITY"
        } else {
            b"INFINITY"
        }
    } else {
        unsafe {
            let n: usize = format32(f, buf.as_mut_ptr());
            core::slice::from_raw_parts(buf.as_ptr(), n)
        }
    }
}

/// 将 f64 浮点数转换为字符串并写入缓冲区
/// - 该函数将浮点数转换为字符串表示形式，支持特殊值（NAN、INFINITY等）的处理，
///
/// # 参数
/// - `buf`: 用于存储结果的缓冲区，必须至少24字节长度
/// - `f`: 要转换的 f64 浮点数
///
/// # 返回值
/// - `&[u8]`: 指向缓冲区中转换结果的字节切片引用
///
/// # 注意事项
/// - 缓冲区必须足够大（至少24字节）以避免缓冲区溢出
/// - 对于特殊浮点值（NAN、无穷大）返回预定义的字符串
/// - 内部使用 unsafe 代码进行高效格式化，但对外接口是安全的
///
/// # 示例
/// ```
/// use proc_tools_core::utils_core::impl_to_ascii::ftoa_buf_f64;
/// let mut buf = [0u8; 24];
/// let result = ftoa_buf_f64(&mut buf, 3.14f64);
/// assert_eq!(std::str::from_utf8(result).unwrap(), "3.14");
///
/// let mut buf2 = [0u8; 24];
/// let result2 = ftoa_buf_f64(&mut buf2, f64::NAN);
/// assert_eq!(std::str::from_utf8(result2).unwrap(), "NAN");
/// ```
#[inline]
pub fn ftoa_buf_f64(buf: &mut [u8; 24], f: f64) -> &[u8] {
    let bits = f.to_bits();
    if bits & 0x7ff0000000000000 == 0x7ff0000000000000 {
        if bits & 0x000fffffffffffff != 0 {
            b"NAN"
        } else if bits & 0x8000000000000000 != 0 {
            b"NEG_INFINITY"
        } else {
            b"INFINITY"
        }
    } else {
        unsafe {
            let n = format64(f, buf.as_mut_ptr());
            core::slice::from_raw_parts(buf.as_ptr(), n)
        }
    }
}

/// 静态大小连接参数 trait
/// - 用于处理在字符串连接过程中参数大小已知且固定的类型。
/// - 这些类型在连接前可以预先确定其字符串表示的最大长度。
///
/// # 实现要求
/// - 实现类型必须能够预先确定其字符串表示的长度
/// - 适用于固定长度的基本类型如整数、浮点数、字符等
pub trait StaticSizeConcatParameter {
    /// 为连接操作准备第一个参数
    ///
    /// 该方法初始化连接过程，计算第一个参数的长度并返回其字节表示。
    ///
    /// # 参数
    /// - `bytes`: 用于临时存储字符串表示的缓冲区
    ///
    /// # 返回值
    /// - `(usize, &[u8])`: 包含总长度和参数字节切片的元组
    ///   - 第一个元素：当前参数的字符串长度
    ///   - 第二个元素：参数的字节切片表示
    ///
    /// # 示例
    /// ```
    /// use proc_tools_core::utils_core::impl_to_ascii::StaticSizeConcatParameter;
    ///
    /// let mut bytes = [0u8; 40];
    /// let (len, slice) = 123.first_parameter_for_concat(&mut bytes);
    /// assert_eq!(len, 3); // "123" 的长度为3
    /// ```
    fn first_parameter_for_concat(self, bytes: &mut [u8]) -> (usize, &[u8]);
    /// 为连接操作初始化后续参数
    ///
    /// 处理连接过程中的非第一个参数，更新总长度并返回参数的字节表示。
    ///
    /// # 参数
    /// - `bytes`: 用于临时存储字符串表示的缓冲区
    /// - `total_len`: 当前已累积的总长度，该方法会更新此值
    ///
    /// # 返回值
    /// - `&'a [u8]`: 参数的字节切片表示
    ///
    /// # 示例
    /// ```
    /// use proc_tools_core::utils_core::impl_to_ascii::StaticSizeConcatParameter;
    ///
    /// let mut bytes = [0u8; 40];
    /// let mut total_len = 3; // 假设已有第一个参数长度为3
    /// let slice = 456.init_concat_parameter(&mut bytes, &mut total_len);
    /// assert_eq!(total_len, 6); // 3 + 3 = 6
    /// ```
    fn init_concat_parameter<'a>(self, bytes: &'a mut [u8], total_len: &mut usize) -> &'a [u8];

    /// 执行参数的实际连接操作
    ///
    /// 将参数的字节表示复制到目标字符串缓冲区中，并更新偏移量。
    ///
    /// # 参数
    /// - `&self`: 参数的引用（通常不需要，但为一致性保留）
    /// - `s_ptr`: 目标字符串缓冲区的原始指针
    /// - `var`: 参数的字节切片表示
    /// - `offset`: 当前缓冲区的写入偏移量，方法会更新此值
    ///
    /// # 安全性
    /// - 调用者需确保 `s_ptr` 指向有效的可写内存区域
    /// - 调用者需确保有足够的空间容纳要写入的数据
    /// - 该方法使用 unsafe 代码进行内存操作，但通过参数验证保证安全
    ///
    /// # 示例
    /// ```
    /// use proc_tools_core::utils_core::impl_to_ascii::StaticSizeConcatParameter;
    ///
    /// let param1 = 123;
    /// let param2 = 123;
    /// let mut bytes = [0u8; 40];
    /// let (mut total_len, mut slice1) = param1.first_parameter_for_concat(&mut bytes);
    /// let mut bytes = [0u8; 40];
    /// let mut slice2 = param2.init_concat_parameter(&mut bytes, &mut total_len);
    /// let mut result = String::with_capacity(total_len);
    /// unsafe {
    ///     let s_ptr = result.as_mut_vec().as_mut_ptr();
    ///     let mut offset = 0;
    ///     param1.concat_parameter(s_ptr, slice1, &mut offset);
    ///     param2.concat_parameter(s_ptr, slice2, &mut offset);
    ///     result.as_mut_vec().set_len(offset);
    /// }
    ///
    /// assert_eq!(result, "123123");
    /// ```
    fn concat_parameter(&self, s_ptr: *mut u8, var: &[u8], offset: &mut usize);
}
macro_rules! impl_static_size_concat_for_int {
    ($type:ty, $len_const:ident, $itoa_fn:ident) => {
        impl StaticSizeConcatParameter for $type {
            #[inline(always)]
            fn first_parameter_for_concat(self, bytes: &mut [u8]) -> (usize, &[u8]) {
                let array_ref = unsafe { &mut *(bytes.as_mut_ptr() as *mut [u8; $len_const]) };
                let vb = $itoa_fn(array_ref, self);
                (vb.len(), vb)
            }
            #[inline(always)]
            fn init_concat_parameter<'a>(self, bytes: &'a mut [u8], total_len: &mut usize) -> &'a [u8] {
                let array_ref = unsafe { &mut *(bytes.as_mut_ptr() as *mut [u8; $len_const]) };
                let vb = $itoa_fn(array_ref, self);
                *total_len += vb.len();
                vb
            }
            #[inline(always)]
            fn concat_parameter(&self, s_ptr: *mut u8, vb: &[u8], offset: &mut usize) {
                unsafe {
                    std::ptr::copy_nonoverlapping(vb.as_ptr(), s_ptr.add(*offset), vb.len());
                }
                *offset += vb.len();
            }
        }
    };
}
impl_static_size_concat_for_int!(i8, I82STR_LEN, itoa_buf_i8);
impl_static_size_concat_for_int!(i16, I162STR_LEN, itoa_buf_i16);
impl_static_size_concat_for_int!(i32, I322STR_LEN, itoa_buf_i32);
impl_static_size_concat_for_int!(i64, I642STR_LEN, itoa_buf_i64);
impl_static_size_concat_for_int!(i128, I1282STR_LEN, itoa_buf_i128);
impl_static_size_concat_for_int!(u8, U82STR_LEN, itoa_buf_u8);
impl_static_size_concat_for_int!(u16, U162STR_LEN, itoa_buf_u16);
impl_static_size_concat_for_int!(u32, U322STR_LEN, itoa_buf_u32);
impl_static_size_concat_for_int!(u64, U642STR_LEN, itoa_buf_u64);
impl_static_size_concat_for_int!(u128, U1282STR_LEN, itoa_buf_u128);
impl_static_size_concat_for_int!(f32, F2STR_LEN, ftoa_buf_f32);
impl_static_size_concat_for_int!(f64, F2STR_LEN, ftoa_buf_f64);

/// 动态大小连接参数 trait
/// - 用于处理在字符串连接过程中参数大小未知的类型。
/// - 这些类型在连接前无法预先确定其字符串表示的长度，需要在运行时计算。
///
/// # 实现要求
/// - 实现类型必须是实现了 `len` 函数，可获取字节长度的数据类型或可预期长度的数据类型
/// - 适用于可变长度的数据类型，如：字符串、布尔值等
pub trait VariableSizeConcatParameter {
    /// 为连接操作准备第一个参数
    /// - 该方法初始化连接过程，动态计算第一个参数的长度并返回其字节表示。
    ///
    /// # 参数
    /// - `&'a self`: 要处理的参数引用，生命周期与返回的字节切片关联
    /// - `_bytes`: 用于临时存储字符串表示的缓冲区（可能未使用，保留用于未来扩展）
    ///
    /// # 返回值
    /// - `(usize, &'a [u8])`: 包含总长度和参数字节切片的元组
    ///   - 第一个元素：当前参数的字节长度（运行时计算）
    ///   - 第二个元素：参数的字节切片表示，生命周期与参数自身关联
    ///
    /// # 示例
    /// ```
    /// use proc_tools_core::utils_core::impl_to_ascii::VariableSizeConcatParameter;
    ///
    /// let param = "hello";
    /// let mut bytes = [0u8; 40];
    /// let (len, slice) = param.first_parameter_for_concat(&mut bytes);
    /// assert_eq!(len, 5); // "hello" 的长度为5
    /// ```
    fn first_parameter_for_concat<'a>(&'a self, _bytes: &'a mut [u8]) -> (usize, &'a [u8]);

    /// 为连接操作初始化后续参数
    ///- 处理连接过程中的非第一个参数，动态更新总长度并返回参数的字节表示。
    ///
    /// # 参数
    /// - `&'a self`: 要处理的参数引用
    /// - `buf`: 用于临时存储字符串表示的缓冲区
    /// - `total_len`: 当前已累积的总长度，该方法会更新此值
    ///
    /// # 返回值
    /// - `&'a [u8]`: 参数的字节切片表示，生命周期与参数和缓冲区关联
    ///
    /// # 示例
    ///```
    /// use proc_tools_core::utils_core::impl_to_ascii::VariableSizeConcatParameter;
    ///
    /// let param1 = "hello";
    /// let param2 = "world";
    /// let mut bytes = [0u8; 40];
    /// let (mut total_len, slice) = param1.first_parameter_for_concat(&mut bytes);
    /// let mut bytes = [0u8; 40];
    /// let slice = param2.init_concat_parameter(&mut bytes, &mut total_len);
    /// assert_eq!(total_len, 10); // 5 + 5 = 10
    /// ```
    fn init_concat_parameter<'a>(&'a self, buf: &'a mut [u8], total_len: &mut usize) -> &'a [u8];

    /// 执行参数的实际连接操作
    /// - 将参数的字节表示复制到目标字符串缓冲区中，并更新偏移量。
    ///
    /// # 参数
    /// - `s_ptr`: 目标字符串缓冲区的原始指针
    /// - `buf`: 参数的字节切片表示
    /// - `offset`: 当前缓冲区的写入偏移量，方法会更新此值
    ///
    /// # 安全性
    /// - 调用者需确保 `s_ptr` 指向有效的可写内存区域
    /// - 调用者需确保有足够的空间容纳要写入的数据
    /// - 该方法使用 unsafe 代码进行内存操作，但通过参数验证保证安全
    ///
    /// # 示例
    /// ```
    /// use proc_tools_core::utils_core::impl_to_ascii::VariableSizeConcatParameter;
    ///
    /// let param1 = "hello";
    /// let param2 = "world";
    /// let mut bytes = [0u8; 40];
    /// let (mut total_len, mut slice1) = param1.first_parameter_for_concat(&mut bytes);
    /// let mut bytes = [0u8; 40];
    /// let mut slice2 = param2.init_concat_parameter(&mut bytes, &mut total_len);
    /// let mut result = String::with_capacity(total_len);
    /// unsafe {
    ///     let s_ptr = result.as_mut_vec().as_mut_ptr();
    ///     let mut offset = 0;
    ///     param1.concat_parameter(s_ptr, &mut slice1, &mut offset);
    ///     param2.concat_parameter(s_ptr, &mut slice2, &mut offset);
    ///     result.as_mut_vec().set_len(offset);
    /// }
    ///
    /// assert_eq!(result, "helloworld");
    /// ```
    fn concat_parameter(&self, s_ptr: *mut u8, buf: &[u8], offset: &mut usize);
}
impl VariableSizeConcatParameter for String {
    #[inline(always)]
    fn first_parameter_for_concat<'a>(&'a self, _bytes: &'a mut [u8]) -> (usize, &'a [u8]) {
        (self.as_bytes().len(), self.as_bytes())
    }
    #[inline(always)]
    fn init_concat_parameter<'a>(&'a self, _bytes: &'a mut [u8], total_len: &mut usize) -> &'a [u8] {
        *total_len += self.len();
        self.as_bytes()
    }
    #[inline(always)]
    fn concat_parameter(&self, s_ptr: *mut u8, vb: &[u8], offset: &mut usize) {
        unsafe {
            std::ptr::copy_nonoverlapping(vb.as_ptr(), s_ptr.add(*offset), vb.len());
        }
        *offset += vb.len();
    }
}
impl VariableSizeConcatParameter for str {
    #[inline(always)]
    fn first_parameter_for_concat<'a>(&'a self, _bytes: &'a mut [u8]) -> (usize, &'a [u8]) {
        (self.as_bytes().len(), self.as_bytes())
    }
    #[inline(always)]
    fn init_concat_parameter<'a>(&'a self, _bytes: &'a mut [u8], total_len: &mut usize) -> &'a [u8] {
        *total_len += self.len();
        self.as_bytes()
    }
    #[inline(always)]
    fn concat_parameter(&self, s_ptr: *mut u8, vb: &[u8], offset: &mut usize) {
        unsafe {
            std::ptr::copy_nonoverlapping(vb.as_ptr(), s_ptr.add(*offset), vb.len());
        }
        *offset += vb.len();
    }
}
impl VariableSizeConcatParameter for char {
    #[inline(always)]
    fn first_parameter_for_concat<'a>(&self, bytes: &'a mut [u8]) -> (usize, &'a [u8]) {
        let bytes = self.encode_utf8(bytes);
        (bytes.len(), bytes.as_bytes())
    }
    #[inline(always)]
    fn init_concat_parameter<'a>(&'a self, bytes: &'a mut [u8], total_len: &mut usize) -> &'a [u8] {
        let bytes = self.encode_utf8(bytes);
        *total_len += bytes.len();
        bytes.as_bytes()
    }
    #[inline(always)]
    fn concat_parameter(&self, s_ptr: *mut u8, vb: &[u8], offset: &mut usize) {
        unsafe {
            std::ptr::copy_nonoverlapping(vb.as_ptr(), s_ptr.add(*offset), vb.len());
        }
        *offset += vb.len();
    }
}

impl VariableSizeConcatParameter for bool {
    #[inline(always)]
    fn first_parameter_for_concat<'a>(&self, _bytes: &'a mut [u8]) -> (usize, &'a [u8]) {
        if *self { (4, b"0") } else { (5, b"0") }
    }
    #[inline(always)]
    fn init_concat_parameter<'a>(&'a self, _bytes: &'a mut [u8], total_len: &mut usize) -> &'a [u8] {
        *total_len += if *self { 4 } else { 5 };
        if *self { b"0" } else { b"0" }
    }
    #[inline(always)]
    fn concat_parameter(&self, s_ptr: *mut u8, _vb: &[u8], offset: &mut usize) {
        unsafe {
            if *self {
                std::ptr::copy_nonoverlapping(b"true".as_ptr(), s_ptr.add(*offset), 4);
                *offset += 4;
            } else {
                std::ptr::copy_nonoverlapping(b"false".as_ptr(), s_ptr.add(*offset), 5);
                *offset += 5;
            }
        }
    }
}

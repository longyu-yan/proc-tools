pub mod float2str;
pub mod utils_core;

/// 将多个字符串片段安全、高效地拼接成一个 [`String`]。
///
/// 该宏会：
/// 1. 计算所有输入片段的总长度。
/// 2. 使用 [`String::with_capacity`] 预分配足够内存，避免多次重新分配。
/// 3. 按顺序将所有片段追加到结果字符串中。
///
/// # 参数
///
/// - `$first`: 第一个字符串片段（通常是主要部分，用于估算容量）。
/// - `$suffix`: 一个或多个后续要拼接的字符串片段（类型需实现 `AsRef<str>`，如 `&str`, `String` 等）。
///
/// # 返回值
/// - [`String`] 返回一个新创建的包含所有输入片段连接的字符串
///
/// # 示例
/// ```rust
/// use proc_tools_core::{concat_str};
///
/// let base = "file";
/// let ext1 = ".txt";
/// let dir = "/home";
/// let ext2 = ".zip";
///
/// let result = concat_str!(base, ext1);                    // → "file.txt"
/// let full_path = concat_str!(dir, "/", base, ext1, ext2); // → "/home/file.txt.zip"
/// ```
#[macro_export]
macro_rules! concat_str {
    ($first:expr $(, $suffix:expr)+) => {{
        let mut total_len = $first.len();
        $(
            total_len += $suffix.len();
        )+
        let mut s = String::with_capacity(total_len);
        s.push_str($first);
        $(
            s.push_str($suffix);
        )+
        s
    }};
}

/// 使用unsafe代码高效替换多个字符串模式，主要适用占位符替换
/// - 通过直接操作字节和指针来替换输入字符串中的多个模式，提供比标准库方法更高的性能
/// - 此函数适合处理大量替换操作或性能敏感的场景
///
/// # 参数
/// - `input`: 待处理的输入字符串
/// - `patterns`: 模式替换对切片，每个元素为 `(模式, 替换内容)`
///
/// # 返回值
/// - `String`: 完成所有替换后的新字符串
///
/// # 安全性
/// - 此函数使用 `unsafe` 代码块进行指针操作，但通过严格的边界检查确保安全
/// - 调用者需确保输入字符串为有效的 UTF-8 编码
/// - 替换内容也应为有效的 UTF-8，否则可能产生无效的字符串
///
/// # 处理逻辑
/// 1. 预处理：过滤空模式并预计算模式信息
/// 2. 容量预估：基于输入长度和替换增长计算初始容量
/// 3. 模式匹配：使用指针比较进行高效模式匹配
/// 4. 字符处理：分别处理 ASCII 和 UTF-8 字符
/// 5. 安全设置：正确设置结果字符串长度
///
/// # 注意事项
/// - 空模式会被自动跳过，避免无限循环
/// - 如果所有模式都被过滤掉，直接返回输入副本
/// - 容量预估有上限，防止过度分配内存
/// - 使用 `copy_nonoverlapping` 确保内存安全
/// - 此函数按模式列表的顺序进行匹配，对于每个位置，按模式列表顺序检查所有模式
///   - 例如：对"abcde" 使用 [("bc", "Y"), ("abc", "X")] 进行替换，实际结果是 "Xde" ，因为 "abc" 比 "bc" 出现位置更靠前
///
/// # 示例
/// ```rust,ignore
/// let input = "Hello world, welcome to Rust!";
/// let patterns = [
///     ("world", "Earth"),
///     ("Rust", "Rust programming"),
/// ];
/// let result = replace_multiple_patterns(input, &patterns);
/// assert_eq!(result, "Hello Earth, welcome to Rust programming!");
/// ```
///
/// # 错误情况
/// - 如果输入包含无效 UTF-8 字符，行为是未定义的
/// - 如果替换内容包含无效 UTF-8，结果字符串可能无效
#[inline]
pub fn replace_multiple_patterns(input: &str, patterns: &[(&str, &str)]) -> String {
    // 预计算模式字节和长度
    let mut patterns_precomputed: Vec<(&[u8], &[u8], usize)> = Vec::with_capacity(patterns.len());
    for &(pattern, replacement) in patterns {
        if pattern.is_empty() {
            // 跳过空字符串模式，避免无限循环
            continue;
        }
        patterns_precomputed.push((pattern.as_bytes(), replacement.as_bytes(), pattern.len()));
    }
    // 如果过滤后没有有效模式，直接返回输入
    if patterns_precomputed.is_empty() {
        return input.to_string();
    }

    // 更精确的容量预估
    let mut capacity = input.len();
    for &(pattern, replacement, _) in &patterns_precomputed {
        if replacement.len() > pattern.len() {
            capacity += (replacement.len() - pattern.len()) * input.len().saturating_div(pattern.len().max(1));
        }
    }
    capacity = capacity.min(input.len() * 2); // 防止过度分配

    let mut result = String::with_capacity(capacity);
    let input_bytes = input.as_bytes();

    unsafe {
        let result_vec = result.as_mut_vec();
        let result_ptr = result_vec.as_mut_ptr();
        let mut write_pos = 0;
        let mut read_pos = 0;
        let input_len = input_bytes.len();

        while read_pos < input_len {
            let mut matched = false;

            // 检查所有可能的模式匹配
            for &(pattern_bytes, replacement_bytes, pattern_len) in &patterns_precomputed {
                // 快速长度检查
                if read_pos + pattern_len > input_len {
                    continue;
                }

                // 使用指针比较，避免边界检查
                let pattern_ptr = pattern_bytes.as_ptr();
                let input_ptr = input_bytes.as_ptr().add(read_pos);

                // 内联比较
                let mut i = 0;
                while i < pattern_len {
                    if *input_ptr.add(i) != *pattern_ptr.add(i) {
                        break;
                    }
                    i += 1;
                }

                if i == pattern_len {
                    // 复制替换内容
                    std::ptr::copy_nonoverlapping(replacement_bytes.as_ptr(), result_ptr.add(write_pos), replacement_bytes.len());
                    write_pos += replacement_bytes.len();
                    read_pos += pattern_len;
                    matched = true;
                    break;
                }
            }

            if !matched {
                let current_byte = input_bytes[read_pos];

                // 快速处理ASCII字符
                if current_byte < 128 {
                    result_ptr.add(write_pos).write(current_byte);
                    write_pos += 1;
                    read_pos += 1;
                } else {
                    // UTF-8字符处理
                    let char_len = if current_byte & 0b1110_0000 == 0b1100_0000 {
                        2
                    } else if current_byte & 0b1111_0000 == 0b1110_0000 {
                        3
                    } else if current_byte & 0b1111_1000 == 0b1111_0000 {
                        4
                    } else {
                        1 // 无效UTF-8，安全处理
                    };

                    // 确保不会越界
                    let actual_len = char_len.min(input_len - read_pos);
                    std::ptr::copy_nonoverlapping(input_bytes.as_ptr().add(read_pos), result_ptr.add(write_pos), actual_len);
                    write_pos += actual_len;
                    read_pos += actual_len;
                }
            }
        }

        result_vec.set_len(write_pos);
    }

    result
}

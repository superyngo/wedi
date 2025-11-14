mod line_wrapper;

#[allow(unused_imports)]
pub use line_wrapper::LineWrapper;

use unicode_width::UnicodeWidthChar;

/// 計算字符串的視覺寬度（考慮寬字元）
/// 中文字元等寬字元會正確計算為 2，ASCII 字元計算為 1
pub fn visual_width(s: &str) -> usize {
    s.chars()
        .map(|ch| UnicodeWidthChar::width(ch).unwrap_or(1))
        .sum()
}

/// 計算單個字符的視覺寬度
#[allow(dead_code)]
pub fn char_width(ch: char) -> usize {
    UnicodeWidthChar::width(ch).unwrap_or(1)
}

//! ANSI Escape Code 切割工具
//!
//! 用於在單行模式下正確截取帶有 ANSI 色碼的文字，
//! 確保顏色正確顯示且不會因切割產生色碼錯亂。
//!
//! ANSI Escape Sequence 格式：
//! - CSI (Control Sequence Introducer): ESC [ 或 \x1b[
//! - SGR (Select Graphic Rendition): CSI n m (例如 \x1b[38;2;255;0;0m)
//!
//! 支援的格式：
//! - 真彩色: \x1b[38;2;R;G;Bm (前景色), \x1b[48;2;R;G;Bm (背景色)
//! - 256 色: \x1b[38;5;Nm (前景色), \x1b[48;5;Nm (背景色)
//! - 基本色: \x1b[30-37m, \x1b[40-47m
//! - 重置: \x1b[0m

use unicode_width::UnicodeWidthChar;

/// 切割帶有 ANSI escape codes 的文字
///
/// # 參數
/// - `text`: 帶有 ANSI 色碼的原始字串
/// - `start_col`: 起始視覺列（從 0 開始）
/// - `width`: 要截取的視覺寬度
///
/// # 返回值
/// 截取後的字串，包含正確的 ANSI 色碼，並在結尾加上重置碼
///
/// # 範例
/// ```ignore
/// let text = "\x1b[31mHello\x1b[0m World";
/// let sliced = slice_ansi_text(text, 2, 5);
/// // 結果: "\x1b[31mllo\x1b[0m W" (從第 2 列開始，取 5 個字符寬度)
/// ```
pub fn slice_ansi_text(text: &str, start_col: usize, width: usize) -> String {
    let mut result = String::with_capacity(text.len());
    let mut current_col = 0;
    let mut chars = text.chars().peekable();

    // 追蹤當前活躍的 ANSI 樣式
    let mut active_style: Option<String> = None;
    let mut need_reset = false;

    while let Some(ch) = chars.next() {
        // 檢測 ANSI escape sequence
        if ch == '\x1b' {
            // 收集整個 escape sequence
            let escape_seq = collect_escape_sequence(ch, &mut chars);

            // 更新活躍樣式
            if escape_seq.ends_with('m') {
                if escape_seq == "\x1b[0m" || escape_seq == "\x1b[m" {
                    // 重置碼
                    active_style = None;
                    if current_col >= start_col && current_col < start_col + width {
                        result.push_str(&escape_seq);
                        need_reset = false;
                    }
                } else {
                    // 新的樣式碼
                    active_style = Some(escape_seq.clone());
                    if current_col >= start_col && current_col < start_col + width {
                        result.push_str(&escape_seq);
                        need_reset = true;
                    }
                }
            }
            continue;
        }

        // 計算字符寬度
        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(1);

        // 檢查是否在可見範圍內
        if current_col + ch_width > start_col && current_col < start_col + width {
            // 如果這是進入可見範圍的第一個字符，且有活躍樣式，先輸出樣式
            if current_col < start_col && result.is_empty() {
                if let Some(ref style) = active_style {
                    result.push_str(style);
                    need_reset = true;
                }
            }

            // 處理部分可見的寬字符
            if current_col < start_col {
                // 字符開始於可見區域之前，用空格填充
                let visible_part = current_col + ch_width - start_col;
                for _ in 0..visible_part.min(ch_width) {
                    result.push(' ');
                }
            } else if current_col + ch_width > start_col + width {
                // 字符結束於可見區域之後，用空格填充可見部分
                let visible_part = start_col + width - current_col;
                for _ in 0..visible_part {
                    result.push(' ');
                }
            } else {
                // 完全可見的字符
                result.push(ch);
            }
        }

        current_col += ch_width;

        // 如果已超出可見範圍，可以提前結束
        if current_col >= start_col + width {
            break;
        }
    }

    // 確保結尾有重置碼（如果有活躍樣式）
    if need_reset && !result.ends_with("\x1b[0m") {
        result.push_str("\x1b[0m");
    }

    result
}

/// 收集完整的 ANSI escape sequence
fn collect_escape_sequence(first_char: char, chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
    let mut seq = String::new();
    seq.push(first_char); // '\x1b'

    // 檢查下一個字符是否是 '['
    if let Some(&'[') = chars.peek() {
        seq.push(chars.next().unwrap());

        // 收集直到遇到終止字符 (字母)
        while let Some(&ch) = chars.peek() {
            seq.push(chars.next().unwrap());
            // SGR 序列以 'm' 結尾，其他 CSI 序列以 @ 到 ~ 範圍的字符結尾
            if ch.is_ascii_alphabetic() || ('@'..='~').contains(&ch) {
                break;
            }
        }
    }

    seq
}

/// 計算帶有 ANSI escape codes 的字串的視覺寬度
///
/// 跳過所有 ANSI escape sequences，只計算可見字符的寬度
#[allow(dead_code)]
pub fn ansi_visual_width(text: &str) -> usize {
    let mut width = 0;
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            // 跳過整個 escape sequence
            collect_escape_sequence(ch, &mut chars);
        } else {
            width += UnicodeWidthChar::width(ch).unwrap_or(1);
        }
    }

    width
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slice_plain_text() {
        let text = "Hello World";
        assert_eq!(slice_ansi_text(text, 0, 5), "Hello");
        assert_eq!(slice_ansi_text(text, 6, 5), "World");
        assert_eq!(slice_ansi_text(text, 2, 3), "llo");
    }

    #[test]
    fn test_slice_with_ansi() {
        // 簡單的紅色文字
        let text = "\x1b[31mHello\x1b[0m";
        let result = slice_ansi_text(text, 0, 3);
        assert!(result.contains("\x1b[31m"));
        assert!(result.contains("Hel"));
        assert!(result.ends_with("\x1b[0m"));
    }

    #[test]
    fn test_slice_mid_ansi() {
        // 從中間開始切割
        let text = "\x1b[31mHello\x1b[0m World";
        let result = slice_ansi_text(text, 2, 5);
        // 應該包含 "llo" 和部分 " W"
        assert!(result.contains("llo"));
    }

    #[test]
    fn test_slice_chinese() {
        // 中文字符（每個佔 2 列）
        let text = "你好世界";
        assert_eq!(slice_ansi_text(text, 0, 4), "你好");
        assert_eq!(slice_ansi_text(text, 2, 4), "好世");
    }

    #[test]
    fn test_slice_chinese_with_ansi() {
        let text = "\x1b[31m你好\x1b[0m世界";
        let result = slice_ansi_text(text, 0, 4);
        assert!(result.contains("你好"));
        assert!(result.contains("\x1b[31m"));
    }

    #[test]
    fn test_ansi_visual_width() {
        assert_eq!(ansi_visual_width("Hello"), 5);
        assert_eq!(ansi_visual_width("\x1b[31mHello\x1b[0m"), 5);
        assert_eq!(ansi_visual_width("你好"), 4);
        assert_eq!(ansi_visual_width("\x1b[38;2;255;0;0m你好\x1b[0m"), 4);
    }

    #[test]
    fn test_true_color_ansi() {
        // 真彩色格式
        let text = "\x1b[38;2;255;128;0mOrange\x1b[0m";
        let result = slice_ansi_text(text, 0, 6);
        assert!(result.contains("\x1b[38;2;255;128;0m"));
        assert!(result.contains("Orange"));
    }

    #[test]
    fn test_256_color_ansi() {
        // 256 色格式
        let text = "\x1b[38;5;196mRed\x1b[0m";
        let result = slice_ansi_text(text, 0, 3);
        assert!(result.contains("\x1b[38;5;196m"));
        assert!(result.contains("Red"));
    }

    #[test]
    fn test_empty_slice() {
        let text = "\x1b[31mHello\x1b[0m";
        assert_eq!(slice_ansi_text(text, 0, 0), "");
        assert_eq!(slice_ansi_text(text, 10, 5), "");
    }

    #[test]
    fn test_partial_wide_char() {
        // 測試寬字符被切割的情況
        let text = "A你好B";
        // 從位置 1 開始（'你' 開始於 1，但只顯示後半部分）
        let result = slice_ansi_text(text, 1, 4);
        // 應該是 " 好B" 或類似（'你' 被部分切割，用空格填充）
        assert_eq!(result.len(), 4); // 視覺寬度應該是 4
    }

    #[test]
    fn test_multiple_colors() {
        let text = "\x1b[31mRed\x1b[32mGreen\x1b[0m";
        let result = slice_ansi_text(text, 0, 8);
        assert!(result.contains("Red"));
        assert!(result.contains("Green"));
    }
}

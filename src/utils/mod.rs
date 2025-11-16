mod line_wrapper;

#[allow(unused_imports)]
pub use line_wrapper::LineWrapper;

use std::sync::atomic::{AtomicBool, Ordering};
use unicode_width::UnicodeWidthChar;

/// 全局調試模式標誌，支持運行時通過 --debug 參數啟用
static DEBUG_MODE: AtomicBool = AtomicBool::new(false);

/// 設置調試模式
pub fn set_debug_mode(enabled: bool) {
    DEBUG_MODE.store(enabled, Ordering::Relaxed);
}

/// 檢查是否啟用調試模式
pub fn is_debug_mode() -> bool {
    DEBUG_MODE.load(Ordering::Relaxed)
}

/// 調試日誌宏，支持編譯時和運行時調試模式
/// - 編譯時：cfg!(debug_assertions) 自動啟用
/// - 運行時：可通過 --debug 參數啟用
///
///   支持格式化參數，使用方式與 println! 相同
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) || $crate::utils::is_debug_mode() {
            eprintln!("[DEBUG] {}", format_args!($($arg)*));
        }
    };
}

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

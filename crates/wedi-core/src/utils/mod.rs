mod ansi_slice;
mod line_wrapper;

pub use ansi_slice::slice_ansi_text;
#[allow(unused_imports)]
pub use line_wrapper::LineWrapper;

use std::sync::atomic::{AtomicBool, Ordering};
use unicode_width::UnicodeWidthChar;

/// 全局調試模式標誌，支持運行時通過 --debug 參數啟用
static DEBUG_MODE: AtomicBool = AtomicBool::new(false);

/// 設置調試模式
#[allow(dead_code)]
pub fn set_debug_mode(enabled: bool) {
    DEBUG_MODE.store(enabled, Ordering::Relaxed);
}

/// 檢查是否啟用調試模式
pub fn is_debug_mode() -> bool {
    DEBUG_MODE.load(Ordering::Relaxed)
}

/// Path of the debug log written when debug mode (`--debug`) is on.
pub fn debug_log_path() -> std::path::PathBuf {
    std::env::temp_dir().join("wedi-debug.log")
}

/// 寫入一行調試日誌到檔案（不可寫 stderr：TUI 執行中會破壞畫面）
#[doc(hidden)]
pub fn write_debug_log(args: std::fmt::Arguments) {
    use std::io::Write;
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(debug_log_path())
    {
        let _ = writeln!(file, "[DEBUG] {}", args);
    }
}

/// Debug logging, enabled with `--debug`; appends to [`debug_log_path`].
///
/// Takes the same arguments as `println!`.
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if $crate::utils::is_debug_mode() {
            $crate::utils::write_debug_log(format_args!($($arg)*));
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

mod ansi_slice;

pub use ansi_slice::slice_ansi_text;

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

/// Terminal columns taken by one character: 2 for wide (CJK) characters,
/// the editor's tab width (4) for a tab, 1 for anything without a defined width.
///
/// The one width rule for layout, cursor, rendering, and dialogs.
pub fn display_width(ch: char) -> usize {
    if ch == '\t' {
        crate::view::TAB_WIDTH
    } else {
        UnicodeWidthChar::width(ch).unwrap_or(1)
    }
}

/// Sum of [`display_width`] over a string.
pub fn visual_width(s: &str) -> usize {
    s.chars().map(display_width).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_width_is_one_rule() {
        // 稽核 F29：寬度規則集中在一處；Tab 與版面一致計 4 欄
        assert_eq!(display_width('a'), 1);
        assert_eq!(display_width('中'), 2);
        assert_eq!(display_width('\t'), 4);
        assert_eq!(display_width('\u{7}'), 1);
        assert_eq!(visual_width("a\t中"), 7);
    }
}

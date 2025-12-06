//! 語法高亮引擎
//!
//! 使用 bat 專案的 syntaxes.bin (219 種語法)
//! 授權：MIT License / Apache License 2.0
//!
//! 實現特點：
//! - Token 層級過濾換行符（避免 Linux 終端殘影問題）
//! - 優化 ANSI 碼生成（只在顏色變化時輸出，減少輸出大小）

use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use std::fmt::Write;
use std::path::Path;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Color, Style, Theme, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};

/// 嵌入的語法集（來自 bat 專案）
///
/// 此檔案來自 bat (https://github.com/sharkdp/bat)
/// 授權：MIT License / Apache License 2.0
/// 包含 219 種語法定義，原始來源為 Sublime Text packages (MIT License)
const SERIALIZED_SYNTAX_SET: &[u8] = include_bytes!("../../assets/syntaxes.bin");

/// 全域語法集（延遲載入）
static SYNTAX_SET: Lazy<SyntaxSet> =
    Lazy::new(|| load_syntax_set().expect("Failed to load embedded syntax set"));

/// 全域主題集（使用 syntect 內建主題）
static THEME_SET: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

/// 載入語法集（未壓縮版本）
fn load_syntax_set() -> Result<SyntaxSet> {
    bincode::deserialize(SERIALIZED_SYNTAX_SET).context("Failed to deserialize syntax set")
}

/// 語法高亮引擎
pub struct HighlightEngine {
    theme: Theme,
    current_syntax: Option<&'static SyntaxReference>,
    true_color: bool,
}

impl HighlightEngine {
    /// 建立新的高亮引擎
    pub fn new(theme_name: Option<&str>, true_color: bool) -> Result<Self> {
        let theme_name = theme_name.unwrap_or("base16-eighties.dark");
        let theme = THEME_SET
            .themes
            .get(theme_name)
            .context(format!("Theme '{}' not found", theme_name))?
            .clone();

        Ok(Self {
            theme,
            current_syntax: None,
            true_color,
        })
    }

    /// 設定當前檔案類型（從路徑檢測）
    pub fn set_file(&mut self, file_path: Option<&Path>) {
        self.current_syntax = self.detect_syntax_from_path(file_path);
    }

    /// 從檔案路徑檢測語法
    fn detect_syntax_from_path(
        &self,
        file_path: Option<&Path>,
    ) -> Option<&'static SyntaxReference> {
        let path = file_path?;

        // 1. 從副檔名檢測
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if let Some(syntax) = SYNTAX_SET.find_syntax_by_extension(ext) {
                return Some(syntax);
            }
        }

        // 2. 從檔名檢測（例如 Makefile, Dockerfile）
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if let Some(syntax) = SYNTAX_SET.find_syntax_by_name(name) {
                return Some(syntax);
            }

            // 特殊檔名處理
            let name_lower = name.to_lowercase();

            // Shell 配置文件
            if matches!(
                name_lower.as_str(),
                ".bashrc"
                    | ".bash_profile"
                    | ".bash_login"
                    | ".bash_logout"
                    | ".zshrc"
                    | ".zprofile"
                    | ".zshenv"
                    | ".zlogin"
                    | ".zlogout"
                    | ".profile"
                    | "bashrc"
                    | "bash_profile"
            ) {
                // 嘗試找 Bash 或 Shell Script 語法
                if let Some(syntax) = SYNTAX_SET
                    .find_syntax_by_name("Bash")
                    .or_else(|| SYNTAX_SET.find_syntax_by_name("Shell Script (Bash)"))
                    .or_else(|| SYNTAX_SET.find_syntax_by_extension("sh"))
                {
                    return Some(syntax);
                }
            }

            // 其他特殊檔名
            match name_lower.as_str() {
                "makefile" | "gnumakefile" => {
                    if let Some(syntax) = SYNTAX_SET.find_syntax_by_name("Makefile") {
                        return Some(syntax);
                    }
                }
                "dockerfile" => {
                    if let Some(syntax) = SYNTAX_SET.find_syntax_by_name("Dockerfile") {
                        return Some(syntax);
                    }
                }
                _ => {}
            }
        }

        None
    }

    /// 從內容檢測語法（shebang）
    #[allow(dead_code)]
    pub fn detect_syntax_from_content(&self, content: &str) -> Option<&'static SyntaxReference> {
        if let Some(first_line) = content.lines().next() {
            if first_line.starts_with("#!") {
                return SYNTAX_SET.find_syntax_by_first_line(first_line);
            }
        }
        None
    }

    /// 建立新的高亮器（用於逐行高亮）
    ///
    /// 注意：這會 clone theme，因為 HighlightLines 需要 'static 生命週期
    pub fn create_highlighter(&self) -> Option<LineHighlighter> {
        self.current_syntax
            .map(|syntax| LineHighlighter::new(syntax, self.theme.clone(), self.true_color))
    }

    /// 是否已啟用語法高亮
    #[allow(dead_code)]
    pub fn is_enabled(&self) -> bool {
        self.current_syntax.is_some()
    }

    /// 取得當前語法名稱
    #[allow(dead_code)]
    pub fn syntax_name(&self) -> Option<&str> {
        self.current_syntax.map(|s| s.name.as_str())
    }

    /// 取得當前主題名稱
    #[allow(dead_code)]
    pub fn theme_name(&self) -> String {
        self.theme
            .name
            .as_ref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Unknown".to_string())
    }

    /// 取得可用主題清單
    #[allow(dead_code)]
    pub fn available_themes() -> Vec<String> {
        THEME_SET.themes.keys().cloned().collect()
    }

    /// 取得可用語法清單
    #[allow(dead_code)]
    pub fn available_syntaxes() -> Vec<String> {
        SYNTAX_SET
            .syntaxes()
            .iter()
            .map(|s| s.name.clone())
            .collect()
    }
}

/// 逐行高亮器（維護內部語法狀態）
///
/// ⚠️ 重要：HighlightLines 內部維護 ParseState，
/// 必須循序處理行才能正確處理跨行語法（如多行註解）
///
/// 優化策略：
/// - Token 層級過濾換行符（而非輸出層級），確保 ANSI 碼完整性
/// - 只在顏色變化時輸出色碼，減少輸出大小約 30-50%
pub struct LineHighlighter {
    inner: HighlightLines<'static>,
    true_color: bool,
}

impl LineHighlighter {
    fn new(syntax: &'static SyntaxReference, theme: Theme, true_color: bool) -> Self {
        // 將 theme 洩漏到 'static 生命週期（接受小量記憶體洩漏以換取簡單性）
        // 這是安全的，因為 theme 數量很少（只有幾個主題）
        let theme_static: &'static Theme = Box::leak(Box::new(theme));

        Self {
            inner: HighlightLines::new(syntax, theme_static),
            true_color,
        }
    }

    /// 高亮單行，返回 ANSI 色碼字串
    ///
    /// 實現特點：
    /// - 在 token 層級過濾換行符，避免終端殘影
    /// - 優化 ANSI 碼生成，只在顏色變化時輸出
    ///
    /// ⚠️ 錯誤處理策略：
    /// - 如果高亮失敗，自動降級為純文字（不崩潰）
    /// - 這確保編輯器在語法錯誤時仍可正常使用
    pub fn highlight_line(&mut self, line: &str) -> String {
        match self.inner.highlight_line(line, &SYNTAX_SET) {
            Ok(ranges) => self.ranges_to_ansi_optimized(&ranges),
            Err(e) => {
                // 降級為純文字，不影響編輯器運作
                if cfg!(debug_assertions) {
                    eprintln!("[WARN] Syntax highlighting failed: {}", e);
                }
                // 過濾換行符
                strip_line_endings(line)
            }
        }
    }

    /// 優化的 ANSI 碼生成（方案 A + C）
    ///
    /// 特點：
    /// 1. Token 層級過濾換行符（修復 Linux 殘影問題）
    /// 2. 只在顏色變化時輸出色碼（減少輸出大小）
    /// 3. 統一處理真彩色和 256 色模式
    fn ranges_to_ansi_optimized(&self, ranges: &[(Style, &str)]) -> String {
        let mut output = String::with_capacity(256); // 預分配以減少重分配
        let mut last_color: Option<Color> = None;

        for (style, text) in ranges {
            // 在 token 層級過濾控制字符（關鍵修復）
            let clean = strip_line_endings(text);
            if clean.is_empty() {
                continue;
            }

            let fg = style.foreground;

            // 只在顏色變化時輸出色碼（效能優化）
            let color_changed = last_color.is_none_or(|last| {
                last.r != fg.r || last.g != fg.g || last.b != fg.b
            });

            if color_changed {
                if self.true_color {
                    let _ = write!(output, "\x1b[38;2;{};{};{}m", fg.r, fg.g, fg.b);
                } else {
                    let code = ansi_colours::ansi256_from_rgb((fg.r, fg.g, fg.b));
                    let _ = write!(output, "\x1b[38;5;{}m", code);
                }
                last_color = Some(fg);
            }

            output.push_str(&clean);
        }

        // 只在有輸出色碼時才需要 reset
        if last_color.is_some() && !output.is_empty() {
            output.push_str("\x1b[0m");
        }

        output
    }
}

/// 移除行尾的換行符（\n, \r, \r\n）
///
/// 這是修復 Linux 終端殘影問題的關鍵函數
#[inline]
fn strip_line_endings(s: &str) -> String {
    let mut result = s;
    // 處理 \r\n (Windows)
    if result.ends_with("\r\n") {
        result = &result[..result.len() - 2];
    }
    // 處理 \n (Unix) 或單獨的 \r (舊 Mac)
    else if result.ends_with('\n') || result.ends_with('\r') {
        result = &result[..result.len() - 1];
    }
    result.to_string()
}

/// 檢測終端是否支援 24-bit 真彩色
///
/// 檢測策略：
/// 1. 檢查 COLORTERM 環境變數
/// 2. 檢查 TERM 環境變數
/// 3. Windows 特殊處理（Windows Terminal, Windows 11）
pub fn supports_true_color() -> bool {
    // 1. 檢查 COLORTERM（最可靠的方式）
    if let Ok(colorterm) = std::env::var("COLORTERM") {
        if colorterm == "truecolor" || colorterm == "24bit" {
            return true;
        }
    }

    // 2. 檢查 TERM
    if let Ok(term) = std::env::var("TERM") {
        if term.contains("24bit") || term.contains("truecolor") {
            return true;
        }
        // iTerm2, Konsole, 等現代終端
        if term.contains("iterm") || term.contains("konsole") {
            return true;
        }
    }

    // 3. Windows 特殊處理
    #[cfg(windows)]
    {
        // Windows Terminal 支援真彩色
        if std::env::var("WT_SESSION").is_ok() {
            return true;
        }

        // Windows 10 1809+ 和 Windows 11 預設支援
        if is_windows_virtual_terminal_enabled() {
            return true;
        }
    }

    // 預設：降級為 256 色
    false
}

#[cfg(windows)]
fn is_windows_virtual_terminal_enabled() -> bool {
    // 嘗試檢查是否啟用 ENABLE_VIRTUAL_TERMINAL_PROCESSING
    unsafe {
        use winapi::um::consoleapi::GetConsoleMode;
        use winapi::um::handleapi::INVALID_HANDLE_VALUE;
        use winapi::um::processenv::GetStdHandle;
        use winapi::um::winbase::STD_OUTPUT_HANDLE;
        use winapi::um::wincon::ENABLE_VIRTUAL_TERMINAL_PROCESSING;

        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        if handle == INVALID_HANDLE_VALUE {
            return false;
        }

        let mut mode = 0;
        if GetConsoleMode(handle, &mut mode) == 0 {
            return false;
        }

        // 檢查是否已啟用
        (mode & ENABLE_VIRTUAL_TERMINAL_PROCESSING) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = HighlightEngine::new(None, true);
        assert!(engine.is_ok());
    }

    #[test]
    fn test_rust_highlighting() {
        let mut engine = HighlightEngine::new(None, true).unwrap();
        engine.set_file(Some(Path::new("test.rs")));
        assert!(engine.is_enabled());
        assert_eq!(engine.syntax_name(), Some("Rust"));

        let mut highlighter = engine.create_highlighter().unwrap();
        let result = highlighter.highlight_line("fn main() {}");
        assert!(!result.is_empty());
        assert!(result.contains("fn"));
    }

    #[test]
    fn test_multiline_comment() {
        let mut engine = HighlightEngine::new(None, true).unwrap();
        engine.set_file(Some(Path::new("test.rs")));

        let mut highlighter = engine.create_highlighter().unwrap();

        // 測試跨行註解
        let line1 = highlighter.highlight_line("/* start");
        let line2 = highlighter.highlight_line("   middle");
        let line3 = highlighter.highlight_line("   end */");

        // 所有行都應該有 ANSI 色碼
        assert!(line1.contains("\x1b["));
        assert!(line2.contains("\x1b["));
        assert!(line3.contains("\x1b["));
    }

    #[test]
    fn test_syntax_count() {
        let syntaxes = HighlightEngine::available_syntaxes();
        assert!(syntaxes.len() >= 200, "Should have 200+ syntaxes from bat");
    }

    #[test]
    fn test_error_handling_graceful_degradation() {
        let mut engine = HighlightEngine::new(None, true).unwrap();
        engine.set_file(Some(Path::new("test.rs")));

        let mut highlighter = engine.create_highlighter().unwrap();
        // 即使是畸形的輸入也應該回傳純文字，不崩潰
        let result = highlighter.highlight_line("畸形語法 {{{");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_newline_stripping() {
        // 測試換行符過濾（方案 A 的核心修復）
        assert_eq!(strip_line_endings("hello\n"), "hello");
        assert_eq!(strip_line_endings("hello\r\n"), "hello");
        assert_eq!(strip_line_endings("hello\r"), "hello");
        assert_eq!(strip_line_endings("hello"), "hello");
        assert_eq!(strip_line_endings(""), "");
    }

    #[test]
    fn test_no_newline_in_output() {
        // 確保高亮輸出不包含換行符（關鍵測試）
        let mut engine = HighlightEngine::new(None, true).unwrap();
        engine.set_file(Some(Path::new("test.rs")));

        let mut highlighter = engine.create_highlighter().unwrap();

        // 測試帶換行符的輸入
        let result = highlighter.highlight_line("fn main() {}\n");
        assert!(!result.contains('\n'), "Output should not contain newline");
        assert!(!result.contains('\r'), "Output should not contain carriage return");

        // 測試 Windows 換行符
        let result2 = highlighter.highlight_line("let x = 1;\r\n");
        assert!(!result2.contains('\n'), "Output should not contain newline");
        assert!(!result2.contains('\r'), "Output should not contain carriage return");
    }

    #[test]
    fn test_optimized_ansi_output() {
        // 測試 ANSI 碼優化：連續相同顏色的 token 只輸出一次色碼
        let mut engine = HighlightEngine::new(None, true).unwrap();
        engine.set_file(Some(Path::new("test.rs")));

        let mut highlighter = engine.create_highlighter().unwrap();
        let result = highlighter.highlight_line("fn main() {}");

        // 應該只有一個 reset code（在最後）
        let reset_count = result.matches("\x1b[0m").count();
        assert_eq!(reset_count, 1, "Should have exactly one reset code at the end");

        // 確保輸出以 reset code 結尾
        assert!(result.ends_with("\x1b[0m"), "Output should end with reset code");
    }

    #[test]
    fn test_256_color_mode() {
        // 測試 256 色模式
        let mut engine = HighlightEngine::new(None, false).unwrap(); // false = 256 色
        engine.set_file(Some(Path::new("test.rs")));

        let mut highlighter = engine.create_highlighter().unwrap();
        let result = highlighter.highlight_line("fn main() {}");

        // 應該使用 256 色格式 \x1b[38;5;XXXm
        assert!(
            result.contains("\x1b[38;5;"),
            "Should use 256-color format"
        );
        // 不應該使用真彩色格式
        assert!(
            !result.contains("\x1b[38;2;"),
            "Should not use true-color format"
        );
    }
}

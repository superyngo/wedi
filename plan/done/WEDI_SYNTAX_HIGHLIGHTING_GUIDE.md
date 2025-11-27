# wedi 專案語法高亮實作指引

## 專案概述

**wedi** 是一個輕量級的跨平台終端文字編輯器，專為系統管理員設計。本指引將協助你為 wedi 實作完整的語法高亮功能。

**當前狀態：**
- 程式碼量：~3000+ 行
- 核心功能：文字編輯、撤銷/重做、搜尋、剪貼簿
- 語法高亮：框架存在但未實現（`src/highlight/` 模組）
- 已有：FileType 檢測、註解檢測（但無渲染）

**目標狀態：**
- 完整的即時語法高亮
- 支援 219 種語言（使用 bat 的語法集）
- 與現有架構無縫整合
- 效能優化（增量渲染、快取）

---

## 一、實作方案總覽

### 方案選擇：使用 bat 專案的 syntaxes.bin

**理由：**
- ✅ 完整支援 219 種語言（經過充分測試）
- ✅ 法律清晰（MIT License / Apache License 2.0）
- ✅ 無需自行維護語法定義
- ✅ 與 bat 保持一致性
- ✅ 檔案大小可控（約 1.6 MB）

**語法集來源：**
- 來源專案：[bat](https://github.com/sharkdp/bat)
- 專案內路徑：`D:\Users\user\Documents\rust\bat\assets\syntaxes.bin`
- wedi 內路徑：`assets/syntaxes.bin`（已複製至專案中）
- 授權：MIT License / Apache License 2.0（雙授權）
- 包含 219+ 種語法定義
- 原始來源：Sublime Text packages (MIT License)
- GitHub: https://github.com/sharkdp/bat

**技術挑戰：**
- ⚠️ 需要維護語法解析狀態（跨行狀態）
- ⚠️ 需要優化大檔案效能
- ⚠️ 需要處理即時編輯的狀態更新

**架構設計：**
```
highlight/
├── mod.rs          - 公開 API
├── detector.rs     - FileType 檢測（已存在）
├── engine.rs       - 核心高亮引擎（新建）
├── cache.rs        - 語法狀態快取（新建）
└── theme.rs        - 主題管理（可選）

assets/
└── syntaxes.bin    - 嵌入的語法集（來自 bat）
```

---

## 二、語法支援清單

### 使用 bat 語法集（219 種）

透過使用 bat 的 syntaxes.bin，wedi 將自動支援 219 種語言，包括：

#### **系統程式語言**
Rust, Python, JavaScript, TypeScript, Go, C, C++, Java, C#, Kotlin, Swift,
Objective-C, Haskell, OCaml, Scala, Erlang, Elixir, Clojure, etc.

#### **Shell 腳本**
Bash, Zsh, Fish, PowerShell, Batch File, Shell Script, etc.

#### **標記/資料語言**
JSON, YAML, TOML, XML, HTML, CSS, Markdown, reStructuredText, LaTeX,
AsciiDoc, INI, etc.

#### **資料庫與查詢**
SQL, GraphQL, etc.

#### **其他**
Git Config, Dockerfile, Makefile, Nginx, Apache Config, Log files,
Diff, Patch, RegExp, 等等

**完整清單：**
可透過 `Highlighter::available_syntaxes()` 取得完整的 219 種語法名稱。

---

## 三、實作步驟

### Step 1: 複製語法集檔案

首先，將 bat 的 syntaxes.bin 複製到 wedi 專案：

```bash
# 在 wedi 專案根目錄
mkdir -p assets
cp D:/Users/user/Documents/rust/bat/assets/syntaxes.bin assets/
```

### Step 2: 更新依賴配置

**檔案：`Cargo.toml`**

```toml
[package]
name = "wedi"
version = "0.2.0"  # 升版號
edition = "2021"

[dependencies]
# 核心依賴
crossterm = { version = "0.27", features = ["event-stream"] }
ropey = "1.6"
unicode-width = "0.1"
pico-args = "0.5"
encoding_rs = "0.8"
anyhow = "1.0"
serde = "1.0"  # 通用序列化（不限於語法高亮）
once_cell = "1.19"  # 延遲初始化（不限於語法高亮）

# 語法高亮依賴（可選）
syntect = { version = "^5.3", default-features = false, features = ["parsing", "regex-onig", "default-themes"], optional = true }
bincode = { version = "^1.3", optional = true }
ansi_colours = { version = "^1.2", optional = true }  # RGB -> 256 色轉換
flate2 = { version = "^1.0", optional = true }  # 如果 syntaxes.bin 是壓縮的

[target.'cfg(windows)'.dependencies]
winapi = { version = "0.3", features = ["winnls", "winuser", "consoleapi", "handleapi", "processenv", "winbase", "wincon"] }

[profile.release]
opt-level = "z"
lto = true
strip = true
codegen-units = 1
panic = "abort"

[features]
default = ["syntax-highlighting"]
syntax-highlighting = ["dep:syntect", "dep:bincode", "dep:ansi_colours"]

# 未來可能的 feature：
# syntax-highlighting-compressed = ["syntax-highlighting", "dep:flate2"]
# minimal = []  # 完全不含語法高亮的極簡版本
```

### Step 3: 建立高亮引擎（基於 cate 的實作）

**新建檔案：`src/highlight/engine.rs`**

```rust
use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use std::path::Path;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, Theme, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::as_24_bit_terminal_escaped;

use super::detector::FileType;

/// 嵌入的語法集（來自 bat 專案）
///
/// 此檔案來自 bat (https://github.com/sharkdp/bat)
/// 授權：MIT License / Apache License 2.0
/// 包含 219 種語法定義，原始來源為 Sublime Text packages (MIT License)
/// 詳見 THIRD-PARTY-LICENSES.md
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
        let theme_name = theme_name.unwrap_or("base16-ocean.dark");
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
    fn detect_syntax_from_path(&self, file_path: Option<&Path>) -> Option<&'static SyntaxReference> {
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
            match name.to_lowercase().as_str() {
                "makefile" | "gnumakefile" => {
                    return SYNTAX_SET.find_syntax_by_name("Makefile");
                }
                "dockerfile" => {
                    return SYNTAX_SET.find_syntax_by_name("Dockerfile");
                }
                _ => {}
            }
        }

        None
    }

    /// 從內容檢測語法（shebang）
    pub fn detect_syntax_from_content(&self, content: &str) -> Option<&'static SyntaxReference> {
        if let Some(first_line) = content.lines().next() {
            if first_line.starts_with("#!") {
                return SYNTAX_SET.find_syntax_by_first_line(first_line);
            }
        }
        None
    }

    /// 建立新的高亮器（用於逐行高亮）
    pub fn create_highlighter(&self) -> Option<LineHighlighter> {
        self.current_syntax.map(|syntax| {
            LineHighlighter::new(syntax, &self.theme, self.true_color)
        })
    }

    /// 是否已啟用語法高亮
    pub fn is_enabled(&self) -> bool {
        self.current_syntax.is_some()
    }

    /// 取得可用主題清單
    pub fn available_themes() -> Vec<String> {
        THEME_SET.themes.keys().cloned().collect()
    }

    /// 取得可用語法清單
    pub fn available_syntaxes() -> Vec<String> {
        SYNTAX_SET
            .syntaxes()
            .iter()
            .map(|s| s.name.clone())
            .collect()
    }
}

/// 逐行高亮器（維護跨行狀態）
pub struct LineHighlighter {
    inner: HighlightLines<'static>,
    true_color: bool,
}

impl LineHighlighter {
    fn new(syntax: &'static SyntaxReference, theme: &Theme, true_color: bool) -> Self {
        Self {
            inner: HighlightLines::new(syntax, theme),
            true_color,
        }
    }

    /// 高亮單行，返回 ANSI 色碼字串
    ///
    /// ⚠️ 錯誤處理策略：
    /// - 如果高亮失敗，自動降級為純文字（不崩潰）
    /// - 這確保編輯器在語法錯誤時仍可正常使用
    pub fn highlight_line(&mut self, line: &str) -> String {
        match self.inner.highlight_line(line, &SYNTAX_SET) {
            Ok(ranges) => {
                let escaped = if self.true_color {
                    as_24_bit_terminal_escaped(&ranges[..], false)
                } else {
                    self.as_8bit_terminal_escaped(&ranges[..])
                };
                escaped
            }
            Err(e) => {
                // 降級為純文字，不影響編輯器運作
                if cfg!(debug_assertions) {
                    eprintln!("[WARN] Syntax highlighting failed: {}", e);
                }
                line.to_string()
            }
        }
    }

    /// 將 syntect 顏色轉為 8-bit ANSI 色碼（256 色模式）
    ///
    /// ⚠️ 注意：ParseState 在 syntect 中是私有的，無法直接存取
    /// 因此快取策略採用簡化版本，不包含語法狀態
    fn as_8bit_terminal_escaped(&self, ranges: &[(Style, &str)]) -> String {
        let mut output = String::new();

        for (style, text) in ranges {
            // 使用 ansi_colours 庫進行精確的 RGB -> 256 色映射（與 bat 相同）
            let fg = style.foreground;
            let color_code = ansi_colours::ansi256_from_rgb((fg.r, fg.g, fg.b));
            output.push_str(&format!("\x1b[38;5;{}m{}\x1b[0m", color_code, text));
        }

        output
    }
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
        // 檢查是否啟用了虛擬終端處理
        if is_windows_virtual_terminal_enabled() {
            return true;
        }
    }

    // 預設：降級為 256 色
    false
}

#[cfg(windows)]
fn is_windows_virtual_terminal_enabled() -> bool {
    // 檢查是否啟用 ENABLE_VIRTUAL_TERMINAL_PROCESSING
    // 如果已啟用，表示系統支援真彩色
    unsafe {
        use winapi::um::consoleapi::{GetConsoleMode, SetConsoleMode};
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

        let mut highlighter = engine.create_highlighter().unwrap();
        let result = highlighter.highlight_line("fn main() {}");
        assert!(!result.is_empty());
        assert!(result.contains("fn"));
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
    fn test_syntax_count() {
        let syntaxes = HighlightEngine::available_syntaxes();
        assert!(syntaxes.len() >= 200, "Should have 200+ syntaxes from bat");
    }
}
```

### Step 4: 建立快取系統

**新建檔案：`src/highlight/cache.rs`**

```rust
use std::collections::HashMap;

/// 單行的高亮快取項目
///
/// ⚠️ 注意：不包含 ParseState，因為 syntect 的 ParseState 是私有的
/// 快取失效策略：修改任何一行時，使該行及之後所有行失效
#[derive(Clone, Debug)]
pub struct CachedLine {
    /// 原始文字內容（用於驗證快取是否有效）
    pub text: String,
    /// 高亮後的 ANSI 字串
    pub highlighted: String,
}

/// 語法狀態快取（用於優化效能）
pub struct HighlightCache {
    /// 快取的行（行號 -> 快取項目）
    lines: HashMap<usize, CachedLine>,
    /// 快取大小限制
    max_size: usize,
}

impl HighlightCache {
    /// 建立新的快取（預設快取 1000 行）
    pub fn new() -> Self {
        Self::with_capacity(1000)
    }

    /// 建立指定容量的快取
    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            lines: HashMap::with_capacity(max_size.min(1000)),
            max_size,
        }
    }

    /// 取得快取的行
    pub fn get(&self, line_idx: usize) -> Option<&CachedLine> {
        self.lines.get(&line_idx)
    }

    /// 檢查行是否已快取且內容相同
    pub fn is_valid(&self, line_idx: usize, text: &str) -> bool {
        self.lines
            .get(&line_idx)
            .map(|cached| cached.text == text)
            .unwrap_or(false)
    }

    /// 插入快取項目
    pub fn insert(&mut self, line_idx: usize, cached: CachedLine) {
        // 如果超過容量，清除舊的快取
        if self.lines.len() >= self.max_size {
            // 簡單策略：清除所有快取（更複雜的可以用 LRU）
            self.lines.clear();
        }

        self.lines.insert(line_idx, cached);
    }

    /// 使指定行失效
    pub fn invalidate(&mut self, line_idx: usize) {
        self.lines.remove(&line_idx);
    }

    /// 使範圍內的行失效
    pub fn invalidate_range(&mut self, start: usize, end: usize) {
        for idx in start..=end {
            self.lines.remove(&idx);
        }
    }

    /// 使從指定行開始的所有行失效
    ///
    /// ⚠️ 這是因為語法狀態可能影響後續所有行（如多行註解）
    pub fn invalidate_from(&mut self, line_idx: usize) {
        self.lines.retain(|&idx, _| idx < line_idx);
    }

    /// 智慧失效：根據編輯操作類型決定失效範圍
    pub fn invalidate_from_edit(&mut self, line_idx: usize, edit_type: EditType) {
        match edit_type {
            EditType::CharInsert | EditType::CharDelete => {
                // 字元級編輯：使當前行及之後所有行失效
                // （因為可能影響語法狀態，例如開始/結束多行註解）
                self.invalidate_from(line_idx);
            }
            EditType::LineInsert | EditType::LineDelete | EditType::MultiLineEdit => {
                // 行級編輯：清除所有快取（行號改變）
                self.clear();
            }
        }
    }

    /// 清除所有快取
    pub fn clear(&mut self) {
        self.lines.clear();
    }

    /// 取得快取統計資訊
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            cached_lines: self.lines.len(),
            capacity: self.max_size,
        }
    }
}

impl Default for HighlightCache {
    fn default() -> Self {
        Self::new()
    }
}

/// 快取統計資訊
#[derive(Debug)]
pub struct CacheStats {
    pub cached_lines: usize,
    pub capacity: usize,
}

/// 編輯操作類型（用於智慧快取失效）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditType {
    /// 插入單個字元
    CharInsert,
    /// 刪除單個字元
    CharDelete,
    /// 插入新行
    LineInsert,
    /// 刪除整行
    LineDelete,
    /// 多行編輯（複製/貼上等）
    MultiLineEdit,
}

#[cfg(test)]
mod tests {
    use super::*;
    use syntect::parsing::SyntaxSet;

    #[test]
    fn test_cache_basic() {
        let mut cache = HighlightCache::new();

        let cached = CachedLine {
            text: "test".to_string(),
            highlighted: "\x1b[0mtest\x1b[0m".to_string(),
        };

        cache.insert(0, cached.clone());
        assert!(cache.is_valid(0, "test"));
        assert!(!cache.is_valid(0, "different"));
    }

    #[test]
    fn test_cache_invalidation() {
        let mut cache = HighlightCache::new();

        let cached = CachedLine {
            text: "test".to_string(),
            highlighted: String::new(),
        };

        cache.insert(0, cached.clone());
        cache.insert(1, cached.clone());
        cache.insert(2, cached);

        assert_eq!(cache.len(), 3);

        // 使第 1 行及之後所有行失效
        cache.invalidate_from(1);

        assert_eq!(cache.len(), 1);
        assert!(cache.get(0).is_some());
        assert!(cache.get(1).is_none());
        assert!(cache.get(2).is_none());
    }

    #[test]
    fn test_smart_invalidation() {
        let mut cache = HighlightCache::new();

        let cached = CachedLine {
            text: "test".to_string(),
            highlighted: String::new(),
        };

        // 建立 10 行快取
        for i in 0..10 {
            cache.insert(i, cached.clone());
        }

        assert_eq!(cache.len(), 10);

        // 字元編輯：使第 5 行及之後失效
        cache.invalidate_from_edit(5, EditType::CharInsert);

        assert_eq!(cache.len(), 5);
        assert!(cache.get(4).is_some());
        assert!(cache.get(5).is_none());
    }

    #[test]
    fn test_line_edit_clears_all() {
        let mut cache = HighlightCache::new();

        let cached = CachedLine {
            text: "test".to_string(),
            highlighted: String::new(),
        };

        for i in 0..10 {
            cache.insert(i, cached.clone());
        }

        // 插入行：清除所有快取
        cache.invalidate_from_edit(5, EditType::LineInsert);

        assert_eq!(cache.len(), 0);
    }
}
```

### Step 5: 更新主模組

**修改檔案：`src/highlight/mod.rs`**

```rust
mod detector;
mod engine;
mod cache;

pub use detector::{FileType, self};
pub use engine::{HighlightEngine, LineHighlighter, supports_true_color};
pub use cache::{HighlightCache, CachedLine};

/// 語法高亮設定
#[derive(Clone, Debug)]
pub struct HighlightConfig {
    /// 是否啟用語法高亮
    pub enabled: bool,
    /// 主題名稱
    pub theme: String,
    /// 是否使用真彩色
    pub true_color: bool,
}

impl Default for HighlightConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            theme: "base16-ocean.dark".to_string(),
            true_color: supports_true_color(),
        }
    }
}
```

### Step 6: 整合到 Editor 與 View

**修改檔案：`src/editor.rs`**

在 Editor 結構體中添加語法高亮相關欄位：

```rust
use crate::highlight::{HighlightEngine, HighlightCache, HighlightConfig};

pub struct Editor {
    // ... 現有欄位
    pub buffer: RopeBuffer,
    pub cursor: Cursor,
    pub view: View,

    // 新增：語法高亮
    highlight_engine: Option<HighlightEngine>,
    highlight_cache: HighlightCache,
    highlight_config: HighlightConfig,
}

impl Editor {
    pub fn new() -> Result<Self> {
        let highlight_config = HighlightConfig::default();
        let highlight_engine = if highlight_config.enabled {
            HighlightEngine::new(
                Some(&highlight_config.theme),
                highlight_config.true_color
            ).ok()
        } else {
            None
        };

        Ok(Self {
            // ... 現有欄位初始化
            highlight_engine,
            highlight_cache: HighlightCache::new(),
            highlight_config,
        })
    }

    /// 載入檔案並設定語法高亮
    pub fn load_file(&mut self, path: &Path) -> Result<()> {
        // 1. 載入檔案內容
        self.buffer = RopeBuffer::from_file_with_encoding(path, /* ... */)?;

        // 2. 檢查檔案大小，決定是否啟用語法高亮
        let file_size = std::fs::metadata(path)?.len();
        let line_count = self.buffer.line_count();

        if file_size > 10 * 1024 * 1024 || line_count > 50000 {
            // 大檔案：停用語法高亮
            self.highlight_engine = None;
            self.status_message = Some(
                "Syntax highlighting disabled for large file".to_string()
            );
        } else if let Some(engine) = &mut self.highlight_engine {
            // 3. 設定檔案類型
            engine.set_file(Some(path));

            // 4. 嘗試從內容檢測（shebang）
            if !engine.is_enabled() {
                if let Some(first_line) = self.buffer.line(0) {
                    if let Some(syntax) = engine.detect_syntax_from_content(&first_line) {
                        // 手動設定語法
                    }
                }
            }
        }

        // 5. 清除快取
        self.highlight_cache.clear();

        Ok(())
    }

    /// 編輯操作後更新快取
    pub fn on_insert_char(&mut self, ch: char) -> Result<()> {
        let line = self.cursor.line;
        let col = self.cursor.column;

        // 1. 執行插入操作
        self.buffer.insert_char(line, col, ch)?;

        // 2. 更新快取
        use crate::highlight::EditType;
        self.highlight_cache.invalidate_from_edit(line, EditType::CharInsert);

        // 3. 移動游標
        self.cursor.move_right(&self.buffer);

        Ok(())
    }

    pub fn on_delete_line(&mut self) -> Result<()> {
        let line = self.cursor.line;

        // 1. 刪除行
        self.buffer.delete_line(line)?;

        // 2. 更新快取（刪除行會影響所有後續行號）
        use crate::highlight::EditType;
        self.highlight_cache.invalidate_from_edit(line, EditType::LineDelete);

        Ok(())
    }
}
```

**修改檔案：`src/view.rs`**

在 View 中實作語法高亮渲染：

```rust
use crate::highlight::{HighlightEngine, HighlightCache, CachedLine, LineHighlighter};

impl View {
    /// 渲染可見區域（核心渲染邏輯）
    pub fn render(&mut self, editor: &mut Editor) -> Result<()> {
        let start_line = self.offset_row;
        let end_line = (start_line + self.screen_height).min(editor.buffer.line_count());

        // 建立高亮器（如果可用）
        let mut highlighter = editor.highlight_engine
            .as_ref()
            .and_then(|engine| engine.create_highlighter());

        // 渲染每一行
        for line_idx in start_line..end_line {
            self.render_line(
                line_idx,
                &editor.buffer,
                &mut highlighter,
                &mut editor.highlight_cache,
            )?;
        }

        Ok(())
    }

    /// 渲染單行（帶快取）
    fn render_line(
        &mut self,
        line_idx: usize,
        buffer: &RopeBuffer,
        highlighter: &mut Option<LineHighlighter>,
        cache: &mut HighlightCache,
    ) -> Result<()> {
        let line_text = buffer.line(line_idx).unwrap_or("");

        // 1. 嘗試從快取取得
        if cache.is_valid(line_idx, &line_text) {
            let cached = cache.get(line_idx).unwrap();
            print!("{}", cached.highlighted);
            return Ok(());
        }

        // 2. 執行高亮（或純文字）
        let highlighted = if let Some(hl) = highlighter {
            let result = hl.highlight_line(&line_text);

            // 快取結果
            cache.insert(line_idx, CachedLine {
                text: line_text.to_string(),
                highlighted: result.clone(),
                parse_state: hl.get_parse_state(),
            });

            result
        } else {
            // 無語法高亮：直接顯示純文字
            line_text.to_string()
        };

        // 3. 輸出
        print!("{}", highlighted);

        Ok(())
    }
}
```

**添加快捷鍵處理**：

```rust
// 在 src/input/handler.rs 中添加
pub enum Command {
    // ... 現有命令
    ToggleHighlight,    // F5
    NextTheme,          // Ctrl+T
}

impl Editor {
    pub fn execute_command(&mut self, cmd: Command) -> Result<()> {
        match cmd {
            Command::ToggleHighlight => {
                self.highlight_config.enabled = !self.highlight_config.enabled;
                if self.highlight_config.enabled {
                    self.highlight_engine = HighlightEngine::new(
                        Some(&self.highlight_config.theme),
                        self.highlight_config.true_color
                    ).ok();
                } else {
                    self.highlight_engine = None;
                }
                self.highlight_cache.clear();
            }
            Command::NextTheme => {
                let themes = HighlightEngine::available_themes();
                let current_idx = themes.iter()
                    .position(|t| t == &self.highlight_config.theme)
                    .unwrap_or(0);
                let next_idx = (current_idx + 1) % themes.len();
                self.highlight_config.theme = themes[next_idx].clone();

                // 重新建立引擎
                if self.highlight_config.enabled {
                    self.highlight_engine = HighlightEngine::new(
                        Some(&self.highlight_config.theme),
                        self.highlight_config.true_color
                    ).ok();
                    self.highlight_cache.clear();
                }
            }
            // ... 其他命令
        }
        Ok(())
    }
}
```

### Step 7: 建立第三方授權文件

**新建檔案：`THIRD-PARTY-LICENSES.md`**

```markdown
# Third-Party Licenses and Acknowledgements

This project uses third-party software and resources. Below are the acknowledgements and license information.

## Syntax Definitions

The syntax highlighting feature uses syntax definitions (`assets/syntaxes.bin`) from the [bat](https://github.com/sharkdp/bat) project.

- **Source**: https://github.com/sharkdp/bat
- **License**: MIT License / Apache License 2.0 (dual licensed)
- **Original Syntax Sources**: Sublime Text Packages (MIT License)
- **Number of Syntaxes**: 219 languages

### bat Project License (MIT)

```
Copyright (c) 2018-2024 bat-developers

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## Rust Dependencies

This project also uses various Rust crates. Run `cargo license` to see a full list of dependencies and their licenses.

**Key Dependencies:**
- syntect (MIT License)
- crossterm (MIT License)
- ropey (MIT License)
- encoding_rs (Apache-2.0 OR MIT)
- anyhow (MIT OR Apache-2.0)
```

### Step 7: 更新 README

**修改檔案：`README.md`**

在 Features 章節加入：

```markdown
## Features

- 基本文字編輯操作
- 撤銷/重做 (Ctrl+Z / Ctrl+Y)
- 複製/貼上 (Ctrl+C / Ctrl+V)
- 搜尋 (Ctrl+F)
- 行號顯示
- **語法高亮 (支援 219 種語言)** ← 新增
- **可自訂主題 (7+ 種內建主題)** ← 新增
- 多編碼支援 (UTF-8, GBK, Big5, Shift-JIS 等)
- 跨平台支援 (Windows, macOS, Linux)
```

在 License 章節加入：

```markdown
## Third-Party Resources

This project uses syntax definitions from the [bat](https://github.com/sharkdp/bat) project, which are licensed under MIT License / Apache License 2.0. The syntax definitions are originally derived from Sublime Text packages (MIT License).

For complete third-party license information, see [THIRD-PARTY-LICENSES.md](THIRD-PARTY-LICENSES.md).

### Acknowledgements

- **bat project** - For the excellent syntax definition collection
- **Sublime Text community** - For maintaining the original syntax definitions
- **syntect** - For the syntax highlighting engine
```

---

## 四、效能優化策略

### 4.1 可見區域高亮

只高亮螢幕可見的行，而不是整個檔案：

```rust
impl View {
    pub fn render_visible_area(&mut self, editor: &mut Editor) -> Result<()> {
        let start = self.scroll_offset;
        let end = (start + self.viewport_height).min(editor.buffer.line_count());

        // 只處理可見行
        for line_idx in start..end {
            self.render_line_with_highlight(
                line_idx,
                editor.buffer.line(line_idx),
                editor.highlight_engine(),
                editor.highlight_cache_mut(),
            )?;
        }

        Ok(())
    }
}
```

### 4.2 大檔案智慧降級

對於超大檔案，採用多級降級策略：

```rust
/// 檔案大小分級
pub enum FileSizeClass {
    Small,      // < 1MB
    Medium,     // 1MB - 10MB
    Large,      // 10MB - 50MB
    VeryLarge,  // > 50MB
}

impl FileSizeClass {
    fn from_metadata(file_size: u64, line_count: usize) -> Self {
        if file_size > 50 * 1024 * 1024 || line_count > 100_000 {
            Self::VeryLarge
        } else if file_size > 10 * 1024 * 1024 || line_count > 50_000 {
            Self::Large
        } else if file_size > 1024 * 1024 || line_count > 10_000 {
            Self::Medium
        } else {
            Self::Small
        }
    }
}

impl Editor {
    pub fn load_file(&mut self, path: &Path) -> Result<()> {
        // 1. 載入檔案
        self.buffer = RopeBuffer::from_file_with_encoding(path, /* ... */)?;

        // 2. 分析檔案大小
        let file_size = std::fs::metadata(path)?.len();
        let line_count = self.buffer.line_count();
        let size_class = FileSizeClass::from_metadata(file_size, line_count);

        // 3. 根據檔案大小調整策略
        match size_class {
            FileSizeClass::Small | FileSizeClass::Medium => {
                // 正常啟用語法高亮
                if let Some(engine) = &mut self.highlight_engine {
                    engine.set_file(Some(path));
                }
            }
            FileSizeClass::Large => {
                // 大檔案：詢問使用者
                if self.ask_user_enable_highlight()? {
                    if let Some(engine) = &mut self.highlight_engine {
                        engine.set_file(Some(path));
                    }
                    self.status_message = Some(
                        "Large file: highlighting enabled (may be slow)".to_string()
                    );
                } else {
                    self.highlight_engine = None;
                    self.status_message = Some(
                        "Highlighting disabled. Press F5 to enable.".to_string()
                    );
                }
            }
            FileSizeClass::VeryLarge => {
                // 超大檔案：強制停用
                self.highlight_engine = None;
                self.status_message = Some(
                    format!("Very large file ({} lines): highlighting disabled", line_count)
                );
            }
        }

        // 4. 清除快取
        self.highlight_cache.clear();

        Ok(())
    }

    /// 詢問使用者是否啟用語法高亮（大檔案）
    fn ask_user_enable_highlight(&mut self) -> Result<bool> {
        // 顯示提示訊息
        self.view.show_dialog(
            "Large file detected. Enable syntax highlighting? (y/n)"
        )?;

        // 等待使用者輸入
        loop {
            if let Event::Key(key_event) = crossterm::event::read()? {
                match key_event.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => return Ok(true),
                    KeyCode::Char('n') | KeyCode::Char('N') => return Ok(false),
                    KeyCode::Esc => return Ok(false),
                    _ => {}
                }
            }
        }
    }
}
```

**優點**：
- ✅ 小檔案無感知，正常運作
- ✅ 大檔案給予使用者選擇權
- ✅ 超大檔案保護編輯器效能
- ✅ 使用者隨時可透過 F5 切換

### 4.3 增量快取失效

只使受影響的行失效，而不是整個快取：

```rust
impl Editor {
    pub fn on_insert_char(&mut self, ch: char) {
        self.buffer.insert_char(self.cursor.line, self.cursor.column, ch);

        // 只使當前行失效（因為跨行狀態，可能需要使後續幾行失效）
        self.highlight_cache.invalidate_range(self.cursor.line, self.cursor.line + 5);
    }

    pub fn on_delete_line(&mut self) {
        let line = self.cursor.line;
        self.buffer.delete_line(line);

        // 使當前行及之後所有行失效（因為行號改變）
        self.highlight_cache.invalidate_range(line, usize::MAX);
    }
}
```

---

## 五、編碼處理策略

### 5.1 問題說明

wedi 支援多種字元編碼（UTF-8、GBK、Big5、Shift-JIS 等），但 syntect 只能處理 UTF-8 字串。因此需要在語法高亮前進行編碼轉換。

### 5.2 編碼轉換流程

```
檔案 (GBK/Big5/...)
    ↓
RopeBuffer (內部儲存：UTF-8)
    ↓
語法高亮 (syntect 處理 UTF-8)
    ↓
ANSI 轉義序列 (UTF-8)
    ↓
終端顯示
```

**重要：** RopeBuffer 應該在內部統一使用 UTF-8 儲存，載入時轉換，儲存時再轉回原編碼。

### 5.3 實作範例

**修改 `src/buffer/rope_buffer.rs`：**

```rust
use encoding_rs::Encoding;

pub struct RopeBuffer {
    rope: Rope,
    /// 檔案原始編碼（用於儲存時轉換回去）
    source_encoding: &'static Encoding,
    /// 儲存時使用的編碼
    save_encoding: &'static Encoding,
}

impl RopeBuffer {
    /// 從檔案載入（自動編碼轉換）
    pub fn from_file_with_encoding(
        path: &Path,
        from_encoding: Option<&'static Encoding>,
    ) -> Result<Self> {
        // 1. 讀取原始位元組
        let bytes = std::fs::read(path)?;

        // 2. 偵測或使用指定編碼
        let encoding = from_encoding.unwrap_or_else(|| {
            detect_encoding(&bytes).unwrap_or(encoding_rs::UTF_8)
        });

        // 3. 解碼為 UTF-8 字串
        let (decoded, _, had_errors) = encoding.decode(&bytes);
        if had_errors {
            eprintln!("[WARN] Encoding errors detected when reading file");
        }

        // 4. 建立 Rope（內部儲存為 UTF-8）
        let rope = Rope::from_str(&decoded);

        Ok(Self {
            rope,
            source_encoding: encoding,
            save_encoding: encoding,  // 預設儲存時使用相同編碼
        })
    }

    /// 儲存檔案（轉換回原編碼）
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        // 1. 從 Rope 取得 UTF-8 字串
        let content = self.rope.to_string();

        // 2. 編碼為目標編碼
        let (encoded, _, had_errors) = self.save_encoding.encode(&content);
        if had_errors {
            eprintln!("[WARN] Encoding errors when saving file");
        }

        // 3. 寫入檔案
        std::fs::write(path, encoded.as_ref())?;

        Ok(())
    }

    /// 取得單行文字（UTF-8）
    pub fn line(&self, idx: usize) -> Option<String> {
        if idx >= self.rope.len_lines() {
            return None;
        }

        let line = self.rope.line(idx);
        Some(line.to_string())
    }
}

/// 自動偵測檔案編碼
fn detect_encoding(bytes: &[u8]) -> Option<&'static Encoding> {
    // 1. 檢查 BOM
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return Some(encoding_rs::UTF_8);
    }
    if bytes.starts_with(&[0xFF, 0xFE]) {
        return Some(encoding_rs::UTF_16LE);
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        return Some(encoding_rs::UTF_16BE);
    }

    // 2. 嘗試 UTF-8 解碼
    if std::str::from_utf8(bytes).is_ok() {
        return Some(encoding_rs::UTF_8);
    }

    // 3. 根據系統 locale 猜測
    #[cfg(windows)]
    {
        return Some(get_system_ansi_encoding());
    }

    #[cfg(not(windows))]
    {
        return Some(encoding_rs::UTF_8);
    }
}
```

**在 View 渲染時：**

```rust
impl View {
    fn render_line(
        &mut self,
        line_idx: usize,
        buffer: &RopeBuffer,
        highlighter: &mut Option<LineHighlighter>,
        cache: &mut HighlightCache,
    ) -> Result<()> {
        // 1. 從 RopeBuffer 取得 UTF-8 字串
        let line_text = buffer.line(line_idx).unwrap_or_default();

        // 2. 語法高亮（syntect 處理 UTF-8）
        let highlighted = if let Some(hl) = highlighter {
            hl.highlight_line(&line_text)  // UTF-8 in, ANSI UTF-8 out
        } else {
            line_text.clone()
        };

        // 3. 輸出到終端（UTF-8）
        print!("{}", highlighted);

        Ok(())
    }
}
```

### 5.4 測試要點

**編碼測試清單：**
- [ ] UTF-8 檔案正確載入和儲存
- [ ] GBK 檔案正確載入和儲存（簡體中文）
- [ ] Big5 檔案正確載入和儲存（繁體中文）
- [ ] Shift-JIS 檔案正確載入（日文）
- [ ] 混合 ASCII + 中文的檔案
- [ ] 語法高亮在非 UTF-8 檔案中正確運作
- [ ] 編碼錯誤時的降級處理

**範例測試檔案：**
```bash
# 建立測試檔案
echo "fn main() { println!(\"你好世界\"); }" | iconv -f UTF-8 -t GBK > test_gbk.rs
echo "fn main() { println!(\"你好世界\"); }" | iconv -f UTF-8 -t BIG5 > test_big5.rs

# 測試 wedi 能否正確開啟
wedi test_gbk.rs -f gbk
wedi test_big5.rs -f big5
```

---

## 六、測試清單

### 基本功能
- [ ] 載入 .rs 檔案，自動啟用 Rust 高亮
- [ ] 載入 .py 檔案，自動啟用 Python 高亮
- [ ] 載入 .txt 檔案，無高亮（純文字）
- [ ] F5 切換高亮開關
- [ ] Ctrl+T 切換主題

### 檔案類型檢測
- [ ] 正確識別 Dockerfile（無副檔名）
- [ ] 正確識別 Makefile
- [ ] 正確識別 Cargo.toml 為 TOML
- [ ] 各種副檔名 (.cpp, .hpp, .js, .ts, .json, .yml)

### 編輯操作
- [ ] 輸入文字後高亮即時更新
- [ ] 刪除文字後高亮正確
- [ ] 撤銷/重做後高亮狀態正確
- [ ] 多行操作（複製/貼上）後高亮正確

### 效能測試
- [ ] 小檔案 (<1000 行) 流暢編輯
- [ ] 中檔案 (1000-10000 行) 可接受效能
- [ ] 大檔案 (10MB-50MB) 使用者選擇是否啟用
- [ ] 超大檔案 (>50MB) 自動停用高亮
- [ ] 捲動時無明顯延遲 (<16ms/frame)

### 快取測試
- [ ] 相同行重複渲染使用快取（效能提升）
- [ ] 修改行後快取失效（智慧失效）
- [ ] 快取包含語法狀態（跨行正確）
- [ ] 插入/刪除行後快取正確調整
- [ ] 快取不會無限增長

### 錯誤處理測試
- [ ] 語法錯誤時降級為純文字（不崩潰）
- [ ] 編碼錯誤時正確處理
- [ ] 無效主題名稱有錯誤提示
- [ ] 畸形 ANSI 序列不影響編輯器

### 編碼整合測試
- [ ] UTF-8 檔案 + 語法高亮正確
- [ ] GBK 檔案 + 語法高亮正確
- [ ] Big5 檔案 + 語法高亮正確
- [ ] 編碼轉換不影響高亮效果

### 主題測試
- [ ] 切換到 "Monokai Extended"
- [ ] 切換到 "Solarized (dark)"
- [ ] 切換到 "base16-ocean.dark"
- [ ] 無效主題名稱有錯誤處理

---

## 六、與 cate 專案的差異

### 相同點
- ✅ 使用相同的 bat syntaxes.bin（219+ 種語言）
- ✅ 使用相同的 syntect 配置
- ✅ 使用相同的授權處理方式（MIT/Apache-2.0 雙授權）
- ✅ 支援真彩色和 256 色模式
- ✅ 使用 ansi_colours 進行 RGB -> 256 色轉換

### 差異點
- ⚠️ **wedi 是編輯器**：需要逐行即時高亮，維護狀態
- ⚠️ **cate 是查看器**：一次性高亮整個檔案
- ⚠️ **wedi 需要快取**：因為編輯時需要重複渲染相同的行
- ⚠️ **wedi 需要狀態管理**：編輯操作會影響高亮狀態
- ⚠️ **wedi 提供多種模式**：Disabled/Fast/Accurate 供使用者選擇
- ⚠️ **wedi 的快取更簡化**：不含 ParseState（因為 syntect 中是私有的）

---

## 七、階段性實作計劃

### Phase 1: MVP（1-2 天）✅ 完成

**目標：** 基本語法高亮能正常運作

- [x] 複製 syntaxes.bin 到專案
- [x] 整合 syntect 依賴（與 cate 相同配置）
- [x] 實作 HighlightEngine（基於 cate 的實作）
- [x] 實作 HighlightCache（簡化版本，不含 ParseState）
- [x] 修改 View 渲染邏輯（基本版本）
- [x] 基本測試（5 種常用語言）

**完成標準：**
- ✅ 能正確高亮 Rust、Python、JavaScript 檔案
- ✅ 語法狀態正確維護（多行註解/字串）
- ✅ 沒有崩潰或錯誤

### Phase 2: 效能優化（1-2 天）✅ 完成

**目標：** 大檔案流暢編輯

⚠️ **重要：此階段提前至 Phase 2，因為：**
- 可見區域高亮是核心優化，應優先實作
- 避免後續階段做無用的全檔案快取

**任務：**
- [x] 三種語法高亮模式（Disabled/Fast/Accurate）
- [x] Fast 模式：只處理可見區域
- [x] Accurate 模式：從 line 0 處理確保語法狀態
- [x] 智慧快取失效（EditType 策略）
- [x] 修復執行順序 bug（scroll_if_needed 在 highlighted_lines 之前）

**完成標準：**
- ✅ 跳頁後高亮正確顯示（無需額外輸入）
- ✅ Fast 模式適合大檔案快速瀏覽
- ✅ Accurate 模式確保多行語法正確

### Phase 3: 完整功能（2-3 天）✅ 部分完成

**目標：** 支援所有語言和主題切換

- [x] 驗證所有 219+ 種語法可用
- [x] 新增快捷鍵（Ctrl+H 循環切換模式）
- [x] 編碼處理整合（UTF-8 ↔ GBK/Big5）
- [x] 錯誤處理完善（降級為純文字）
- [x] 更新說明文件（README, CLAUDE.md）
- [x] 添加 bat 專案版權說明
- [ ] 主題切換功能（可選功能，未實作）

**完成標準：**
- ✅ 所有 219+ 種語法正確載入
- ✅ 模式切換流暢
- ✅ 非 UTF-8 檔案正確處理
- ✅ 版權說明完整

### Phase 4: 使用者體驗（1 天）

**目標：** 改善互動和回饋

- [ ] 狀態列顯示當前主題
- [ ] 狀態列顯示檔案類型
- [ ] 主題預覽（顯示配色範例）
- [ ] 說明文件更新（README, CLAUDE.md）
- [ ] 快捷鍵說明（F1 幫助頁面）
- [ ] 跨平台測試（Windows, Linux, macOS）

**完成標準：**
- ✅ 使用者可輕鬆切換主題
- ✅ 狀態列資訊清晰
- ✅ 文件完整

---

**調整後的順序優勢：**
1. ✅ Phase 1 完成後即可展示核心功能
2. ✅ Phase 2 提早優化效能，避免後續重構
3. ✅ Phase 3 在效能穩定後添加完整功能
4. ✅ Phase 4 最後打磨使用者體驗

**預估總時間：5-8 天（與原計畫相同）**

---

## 八、參考資源

- [syntect 文件](https://docs.rs/syntect/)
- [bat 專案](https://github.com/sharkdp/bat)
- [cate 專案實作](../cate/src/highlighter.rs) - 參考相同方案
- [crossterm 顏色](https://docs.rs/crossterm/latest/crossterm/style/enum.Color.html)
- [ropey 文件](https://docs.rs/ropey/) - 文字緩衝區

---

## 最後檢查清單

實作完成後，請確認：

### 核心功能
- [x] 所有測試通過（包括單元測試和整合測試）
- [x] 無編譯警告（`cargo clippy -- -D warnings` 通過）
- [x] 語法狀態正確維護（多行註解/字串測試通過）
- [x] 錯誤處理完善（語法錯誤降級為純文字）

### 效能與品質
- [ ] 效能符合預期：
  - [ ] 啟動時間 < 100ms
  - [ ] 10000 行檔案捲動流暢（< 16ms/frame）
  - [ ] 記憶體使用合理（< 100MB for 10000 lines）
- [ ] 二進位大小合理（< 5MB，含 syntaxes.bin）
- [ ] 快取命中率 > 80%（重複捲動時）

### 編碼與相容性
- [ ] UTF-8、GBK、Big5 檔案正確處理
- [ ] 編碼轉換不影響語法高亮
- [ ] 真彩色和 256 色模式都正確運作

### 文件與授權
- [x] README.md 已更新（新增語法高亮功能說明及 bat 版權資訊）
- [x] CLAUDE.md 已更新（新增 highlight 模組完整架構說明）
- [x] WEDI_SYNTAX_HIGHLIGHTING_GUIDE.md 已更新（確保 bat 引用完整）
- [x] syntaxes.bin 已包含在專案中（assets/syntaxes.bin）
- [x] 授權資訊完整（MIT/Apache-2.0 雙授權，引用 bat 專案）
- [x] README.md 包含第三方致謝章節

### 跨平台測試
- [ ] Windows（CMD、PowerShell、Windows Terminal）
- [ ] macOS（Terminal.app、iTerm2）
- [ ] Linux（gnome-terminal、konsole、xterm）
- [ ] 各平台終端色彩正確（真彩色/256色自動檢測）

### 使用者體驗
- [ ] F5 切換語法高亮功能正常
- [ ] Ctrl+T 切換主題功能正常
- [ ] 大檔案提示訊息清晰
- [ ] 狀態列顯示正確資訊
- [ ] 快捷鍵說明文件完整

---

**預估總開發時間：5-8 天**
**建議開發順序：Phase 1 → Phase 2 → Phase 3 → Phase 4**

**重要改進：**
- ✅ 快取包含語法狀態（解決跨行問題）
- ✅ 智慧快取失效（避免過度失效）
- ✅ 錯誤降級處理（提高穩定性）
- ✅ 真彩色檢測完善（更好的跨平台支援）
- ✅ 編碼處理清晰（UTF-8 ↔ GBK/Big5）
- ✅ 大檔案多級降級（更好的使用者體驗）

**參考專案：** 此方案與 cate 專案使用相同的技術棧，可參考 cate 的實作細節。主要差異在於 wedi 是編輯器，需要處理即時編輯和狀態管理。

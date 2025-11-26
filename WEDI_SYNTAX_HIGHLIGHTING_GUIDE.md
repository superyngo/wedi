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
- 支援 20+ 種常用語言
- 與現有架構無縫整合
- 效能優化（增量渲染、快取）

---

## 一、實作方案總覽

### 方案選擇：客製化整合 syntect

**理由：**
- ✅ 編輯器需要逐行增量高亮（非一次性渲染）
- ✅ 已有完整的 FileType 檢測框架
- ✅ 使用 crossterm，需要深度整合
- ✅ 可以利用現有的視窗渲染邏輯

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
└── theme.rs        - 主題管理（新建）
```

---

## 二、語法支援清單

### 推薦支援的語言（28 種）

基於 wedi 的目標用戶（系統管理員、開發者），以下是建議的語言清單：

#### **系統程式語言（9 種）**
- Rust (.rs) - 已在 FileType 中
- Python (.py) - 已在 FileType 中
- JavaScript (.js) - 已在 FileType 中
- TypeScript (.ts) - 已在 FileType 中
- Go (.go) - 已在 FileType 中
- C (.c, .h) - 已在 FileType 中
- C++ (.cpp, .hpp) - 已在 FileType 中
- Java (.java) - 已在 FileType 中
- C# (.cs)

#### **Shell 腳本（6 種）**
- Bash (.sh) - 已在 FileType 中
- PowerShell (.ps1)
- Batch (.bat, .cmd)
- Zsh (.zsh)
- Makefile
- Dockerfile

#### **標記/資料語言（8 種）**
- JSON (.json) - 已在 FileType 中
- YAML (.yml, .yaml) - 已在 FileType 中
- TOML (.toml)
- XML (.xml)
- HTML (.html) - 已在 FileType 中
- CSS (.css) - 已在 FileType 中
- Markdown (.md) - 已在 FileType 中
- INI (.ini)

#### **資料庫與查詢（2 種）**
- SQL (.sql)
- GraphQL (.gql)

#### **其他常用（3 種）**
- Git Config (.gitignore, .gitconfig)
- Log Files (.log)
- Plain Text (.txt)

**與現有 FileType enum 的對應：**
wedi 已經定義了 14+ 種 FileType，我們將擴充並映射到 syntect 語法。

---

## 三、實作步驟

### Step 1: 更新依賴配置

**檔案：`Cargo.toml`**

```toml
[package]
name = "wedi"
version = "0.2.0"  # 升版號
edition = "2021"

[dependencies]
crossterm = { version = "0.27", features = ["event-stream"] }
ropey = "1.6"
unicode-width = "0.1"
pico-args = "0.5"
encoding_rs = "0.8"
anyhow = "1.0"

# 新增：語法高亮
syntect = { version = "5.3.0", default-features = false, features = ["parsing"] }
once_cell = "1.19"

[target.'cfg(windows)'.dependencies]
winapi = { version = "0.3", features = ["winnls", "winuser"] }

[profile.release]
opt-level = "z"
lto = true
strip = true
codegen-units = 1
panic = "abort"

[features]
default = ["syntax-highlighting"]
syntax-highlighting = ["syntect", "once_cell"]
```

### Step 2: 擴充 FileType 檢測

**修改檔案：`src/highlight/detector.rs`**

```rust
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    // 程式語言
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
    C,
    Cpp,
    Java,
    CSharp,

    // 腳本
    Shell,
    PowerShell,
    Batch,
    Makefile,
    Dockerfile,

    // 標記語言
    Html,
    Css,
    Markdown,
    Json,
    Yaml,
    Toml,
    Xml,
    Ini,

    // 資料庫
    Sql,

    // 其他
    Text,
    Unknown,
}

impl FileType {
    /// 從檔案路徑檢測檔案類型
    pub fn from_path(path: &Path) -> Self {
        // 檢查特殊檔名
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            match name.to_lowercase().as_str() {
                "makefile" | "gnumakefile" => return Self::Makefile,
                "dockerfile" => return Self::Dockerfile,
                "cargo.toml" => return Self::Toml,
                "package.json" => return Self::Json,
                _ => {}
            }
        }

        // 從副檔名檢測
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            Self::from_extension(ext)
        } else {
            Self::Unknown
        }
    }

    /// 從副檔名檢測
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            // 程式語言
            "rs" => Self::Rust,
            "py" | "pyw" | "pyi" => Self::Python,
            "js" | "mjs" | "cjs" => Self::JavaScript,
            "ts" | "mts" | "cts" => Self::TypeScript,
            "go" => Self::Go,
            "c" | "h" => Self::C,
            "cpp" | "hpp" | "cc" | "cxx" | "hxx" | "c++" | "h++" => Self::Cpp,
            "java" => Self::Java,
            "cs" => Self::CSharp,

            // 腳本
            "sh" | "bash" | "zsh" => Self::Shell,
            "ps1" | "psm1" | "psd1" => Self::PowerShell,
            "bat" | "cmd" => Self::Batch,

            // 標記語言
            "html" | "htm" => Self::Html,
            "css" | "scss" | "sass" => Self::Css,
            "md" | "markdown" => Self::Markdown,
            "json" | "jsonc" => Self::Json,
            "yml" | "yaml" => Self::Yaml,
            "toml" => Self::Toml,
            "xml" => Self::Xml,
            "ini" | "conf" | "cfg" => Self::Ini,

            // 資料庫
            "sql" => Self::Sql,

            // 其他
            "txt" | "text" => Self::Text,
            _ => Self::Unknown,
        }
    }

    /// 取得對應的 syntect 語法名稱
    pub fn syntect_name(&self) -> Option<&'static str> {
        match self {
            Self::Rust => Some("Rust"),
            Self::Python => Some("Python"),
            Self::JavaScript => Some("JavaScript"),
            Self::TypeScript => Some("TypeScript"),
            Self::Go => Some("Go"),
            Self::C => Some("C"),
            Self::Cpp => Some("C++"),
            Self::Java => Some("Java"),
            Self::CSharp => Some("C#"),
            Self::Shell => Some("Bash"),
            Self::PowerShell => Some("PowerShell"),
            Self::Batch => Some("Batch File"),
            Self::Makefile => Some("Makefile"),
            Self::Dockerfile => Some("Dockerfile"),
            Self::Html => Some("HTML"),
            Self::Css => Some("CSS"),
            Self::Markdown => Some("Markdown"),
            Self::Json => Some("JSON"),
            Self::Yaml => Some("YAML"),
            Self::Toml => Some("TOML"),
            Self::Xml => Some("XML"),
            Self::Ini => Some("INI"),
            Self::Sql => Some("SQL"),
            Self::Text | Self::Unknown => None,
        }
    }

    /// 是否應該啟用語法高亮
    pub fn supports_highlighting(&self) -> bool {
        self.syntect_name().is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_detection() {
        assert_eq!(FileType::from_extension("rs"), FileType::Rust);
        assert_eq!(FileType::from_extension("py"), FileType::Python);
        assert_eq!(FileType::from_extension("cpp"), FileType::Cpp);
    }

    #[test]
    fn test_special_filenames() {
        assert_eq!(FileType::from_path(Path::new("Makefile")), FileType::Makefile);
        assert_eq!(FileType::from_path(Path::new("Dockerfile")), FileType::Dockerfile);
    }
}
```

### Step 3: 建立高亮引擎

**新建檔案：`src/highlight/engine.rs`**

```rust
use anyhow::{Result, Context};
use crossterm::style::Color;
use once_cell::sync::Lazy;
use syntect::parsing::{SyntaxSet, SyntaxReference};
use syntect::highlighting::{Theme, ThemeSet, Style, Highlighter as SyntectHighlighter};
use syntect::easy::HighlightLines;

use super::detector::FileType;

/// 全域語法集
static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);

/// 全域主題集
static THEME_SET: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

/// 語法高亮引擎
pub struct HighlightEngine {
    theme: Theme,
    current_file_type: Option<FileType>,
    syntax: Option<&'static SyntaxReference>,
}

impl HighlightEngine {
    /// 建立新的高亮引擎
    pub fn new(theme_name: Option<&str>) -> Result<Self> {
        let theme_name = theme_name.unwrap_or("base16-ocean.dark");
        let theme = THEME_SET.themes.get(theme_name)
            .context(format!("Theme '{}' not found", theme_name))?
            .clone();

        Ok(Self {
            theme,
            current_file_type: None,
            syntax: None,
        })
    }

    /// 設定當前檔案類型
    pub fn set_file_type(&mut self, file_type: FileType) {
        self.current_file_type = Some(file_type);

        // 查找對應的語法
        if let Some(syntax_name) = file_type.syntect_name() {
            self.syntax = SYNTAX_SET.find_syntax_by_name(syntax_name);
        } else {
            self.syntax = None;
        }
    }

    /// 建立新的高亮器（用於逐行高亮）
    pub fn create_highlighter(&self) -> Option<LineHighlighter> {
        self.syntax.map(|syntax| {
            LineHighlighter::new(syntax, &self.theme)
        })
    }

    /// 取得主題的預設前景色
    pub fn default_foreground(&self) -> Color {
        let settings = &self.theme.settings;
        if let Some(fg) = settings.foreground {
            syntect_to_crossterm_color(fg)
        } else {
            Color::White
        }
    }

    /// 取得主題的預設背景色
    pub fn default_background(&self) -> Color {
        let settings = &self.theme.settings;
        if let Some(bg) = settings.background {
            syntect_to_crossterm_color(bg)
        } else {
            Color::Black
        }
    }

    /// 是否已啟用語法高亮
    pub fn is_enabled(&self) -> bool {
        self.syntax.is_some()
    }

    /// 取得可用主題清單
    pub fn available_themes() -> Vec<String> {
        THEME_SET.themes.keys().cloned().collect()
    }
}

/// 逐行高亮器（維護跨行狀態）
pub struct LineHighlighter {
    inner: HighlightLines<'static>,
}

impl LineHighlighter {
    fn new(syntax: &'static SyntaxReference, theme: &Theme) -> Self {
        Self {
            inner: HighlightLines::new(syntax, theme),
        }
    }

    /// 高亮單行，返回帶顏色的文字片段
    pub fn highlight_line(&mut self, line: &str) -> Result<Vec<(Color, String)>> {
        let regions = self.inner
            .highlight_line(line, &SYNTAX_SET)
            .context("Failed to highlight line")?;

        Ok(regions
            .into_iter()
            .map(|(style, text)| {
                let color = syntect_to_crossterm_color(style.foreground);
                (color, text.to_string())
            })
            .collect())
    }
}

/// 將 syntect 顏色轉為 crossterm 顏色
fn syntect_to_crossterm_color(color: syntect::highlighting::Color) -> Color {
    Color::Rgb {
        r: color.r,
        g: color.g,
        b: color.b,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = HighlightEngine::new(None);
        assert!(engine.is_ok());
    }

    #[test]
    fn test_rust_highlighting() {
        let mut engine = HighlightEngine::new(None).unwrap();
        engine.set_file_type(FileType::Rust);
        assert!(engine.is_enabled());

        let mut highlighter = engine.create_highlighter().unwrap();
        let result = highlighter.highlight_line("fn main() {}");
        assert!(result.is_ok());
    }
}
```

### Step 4: 建立快取系統

**新建檔案：`src/highlight/cache.rs`**

```rust
use std::collections::HashMap;
use crossterm::style::Color;

/// 單行的高亮快取項目
#[derive(Clone, Debug)]
pub struct CachedLine {
    /// 原始文字內容
    pub text: String,
    /// 高亮後的片段：(顏色, 文字)
    pub segments: Vec<(Color, String)>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_basic() {
        let mut cache = HighlightCache::new();
        let cached = CachedLine {
            text: "test".to_string(),
            segments: vec![(Color::White, "test".to_string())],
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
            segments: vec![],
        };

        cache.insert(0, cached);
        assert!(cache.get(0).is_some());

        cache.invalidate(0);
        assert!(cache.get(0).is_none());
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
pub use engine::{HighlightEngine, LineHighlighter};
pub use cache::{HighlightCache, CachedLine};

/// 語法高亮設定
#[derive(Clone, Debug)]
pub struct HighlightConfig {
    /// 是否啟用語法高亮
    pub enabled: bool,
    /// 主題名稱
    pub theme: String,
}

impl Default for HighlightConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            theme: "base16-ocean.dark".to_string(),
        }
    }
}
```

### Step 6: 整合到 Editor

**修改檔案：`src/editor.rs`**

在 Editor 結構中加入高亮相關欄位：

```rust
use crate::highlight::{HighlightEngine, HighlightCache, HighlightConfig, FileType};

pub struct Editor {
    // ... 現有欄位

    // 新增：語法高亮
    highlight_engine: Option<HighlightEngine>,
    highlight_cache: HighlightCache,
    highlight_config: HighlightConfig,
    current_file_type: Option<FileType>,
}

impl Editor {
    pub fn new(/* ... */) -> Result<Self> {
        // ... 現有初始化

        // 初始化語法高亮
        let highlight_config = HighlightConfig::default();
        let highlight_engine = if highlight_config.enabled {
            HighlightEngine::new(Some(&highlight_config.theme)).ok()
        } else {
            None
        };

        Ok(Self {
            // ... 現有欄位
            highlight_engine,
            highlight_cache: HighlightCache::new(),
            highlight_config,
            current_file_type: None,
        })
    }

    /// 載入檔案後檢測檔案類型
    pub fn detect_and_set_file_type(&mut self, file_path: &std::path::Path) {
        let file_type = FileType::from_path(file_path);
        self.current_file_type = Some(file_type);

        if let Some(engine) = &mut self.highlight_engine {
            engine.set_file_type(file_type);
        }

        // 清除舊快取
        self.highlight_cache.clear();
    }

    /// 切換語法高亮開關
    pub fn toggle_syntax_highlighting(&mut self) -> Result<()> {
        if self.highlight_engine.is_some() {
            self.highlight_engine = None;
            self.highlight_config.enabled = false;
        } else {
            self.highlight_engine =
                Some(HighlightEngine::new(Some(&self.highlight_config.theme))?);
            self.highlight_config.enabled = true;

            // 重新設定檔案類型
            if let Some(file_type) = self.current_file_type {
                self.highlight_engine.as_mut().unwrap().set_file_type(file_type);
            }
        }

        // 清除快取以觸發重繪
        self.highlight_cache.clear();
        Ok(())
    }

    /// 更換主題
    pub fn set_theme(&mut self, theme_name: &str) -> Result<()> {
        self.highlight_config.theme = theme_name.to_string();
        self.highlight_engine = Some(HighlightEngine::new(Some(theme_name))?);

        // 重新設定檔案類型
        if let Some(file_type) = self.current_file_type {
            self.highlight_engine.as_mut().unwrap().set_file_type(file_type);
        }

        self.highlight_cache.clear();
        Ok(())
    }

    /// 取得語法高亮引擎的參考
    pub fn highlight_engine(&self) -> Option<&HighlightEngine> {
        self.highlight_engine.as_ref()
    }

    /// 取得快取的可變參考
    pub fn highlight_cache_mut(&mut self) -> &mut HighlightCache {
        &mut self.highlight_cache
    }

    /// 文字修改時使快取失效
    pub fn on_text_modified(&mut self, line_idx: usize) {
        // 使當前行及後續幾行失效（因為語法狀態可能影響後續行）
        self.highlight_cache.invalidate_range(line_idx, line_idx + 10);
    }
}
```

### Step 7: 整合到 View 渲染

**修改檔案：`src/view.rs`**

```rust
use crate::highlight::{HighlightEngine, HighlightCache, CachedLine};
use crossterm::style::{Color, SetForegroundColor, ResetColor};

impl View {
    /// 渲染單行（帶語法高亮）
    pub fn render_line_with_highlight(
        &self,
        line_idx: usize,
        line_text: &str,
        highlight_engine: Option<&HighlightEngine>,
        highlight_cache: &mut HighlightCache,
    ) -> Result<()> {
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();

        // 如果沒有高亮引擎或未啟用，直接渲染純文字
        if highlight_engine.is_none() || !highlight_engine.unwrap().is_enabled() {
            queue!(handle, Print(line_text))?;
            return Ok(());
        }

        let engine = highlight_engine.unwrap();

        // 檢查快取
        if highlight_cache.is_valid(line_idx, line_text) {
            // 使用快取
            if let Some(cached) = highlight_cache.get(line_idx) {
                for (color, text) in &cached.segments {
                    queue!(
                        handle,
                        SetForegroundColor(*color),
                        Print(text),
                    )?;
                }
                queue!(handle, ResetColor)?;
                return Ok(());
            }
        }

        // 需要重新高亮
        if let Some(mut highlighter) = engine.create_highlighter() {
            match highlighter.highlight_line(line_text) {
                Ok(segments) => {
                    // 渲染
                    for (color, text) in &segments {
                        queue!(
                            handle,
                            SetForegroundColor(*color),
                            Print(text),
                        )?;
                    }
                    queue!(handle, ResetColor)?;

                    // 加入快取
                    highlight_cache.insert(line_idx, CachedLine {
                        text: line_text.to_string(),
                        segments,
                    });
                }
                Err(_) => {
                    // 高亮失敗，渲染純文字
                    queue!(handle, Print(line_text))?;
                }
            }
        } else {
            // 無法建立高亮器，渲染純文字
            queue!(handle, Print(line_text))?;
        }

        Ok(())
    }

    /// 修改後的渲染迴圈（整合高亮）
    pub fn render(&mut self, editor: &Editor) -> Result<()> {
        let mut stdout = std::io::stdout();
        queue!(stdout, crossterm::terminal::Clear(ClearType::All))?;

        let start_line = self.scroll_offset;
        let end_line = (start_line + self.viewport_height).min(self.buffer.line_count());

        for line_idx in start_line..end_line {
            let line_text = self.buffer.line(line_idx);

            // 渲染行號（如果啟用）
            if self.show_line_numbers {
                self.render_line_number(line_idx)?;
            }

            // 渲染行內容（帶高亮）
            self.render_line_with_highlight(
                line_idx,
                line_text,
                editor.highlight_engine(),
                editor.highlight_cache_mut(),
            )?;

            queue!(stdout, Print("\r\n"))?;
        }

        stdout.flush()?;
        Ok(())
    }
}
```

### Step 8: 新增快捷鍵

**修改檔案：`src/input/keymap.rs`**

新增語法高亮相關命令：

```rust
use crossterm::event::{KeyCode, KeyModifiers};

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    // ... 現有命令

    // 新增：語法高亮
    ToggleSyntaxHighlight,   // F5 或 Ctrl+H
    NextTheme,               // Ctrl+T
    PrevTheme,
}

pub fn map_key_to_command(key_code: KeyCode, modifiers: KeyModifiers) -> Option<Command> {
    match (key_code, modifiers) {
        // ... 現有映射

        // 語法高亮快捷鍵
        (KeyCode::F(5), KeyModifiers::NONE) => Some(Command::ToggleSyntaxHighlight),
        (KeyCode::Char('h'), KeyModifiers::CONTROL) => Some(Command::ToggleSyntaxHighlight),
        (KeyCode::Char('t'), KeyModifiers::CONTROL) => Some(Command::NextTheme),

        _ => None,
    }
}
```

### Step 9: 處理命令

**修改檔案：`src/input/handler.rs`**

```rust
impl CommandHandler {
    pub fn execute(&mut self, command: Command, editor: &mut Editor) -> Result<()> {
        match command {
            // ... 現有命令處理

            Command::ToggleSyntaxHighlight => {
                editor.toggle_syntax_highlighting()?;
                self.set_status_message("Syntax highlighting toggled");
            }

            Command::NextTheme => {
                let themes = HighlightEngine::available_themes();
                if !themes.is_empty() {
                    let current_idx = themes
                        .iter()
                        .position(|t| t == &editor.highlight_config.theme)
                        .unwrap_or(0);
                    let next_idx = (current_idx + 1) % themes.len();
                    let next_theme = &themes[next_idx];

                    editor.set_theme(next_theme)?;
                    self.set_status_message(&format!("Theme: {}", next_theme));
                }
            }

            _ => {}
        }
        Ok(())
    }
}
```

### Step 10: 更新 main.rs

**修改檔案：`src/main.rs`**

在載入檔案後檢測檔案類型：

```rust
fn main() -> Result<()> {
    // ... 解析參數

    let mut editor = Editor::new(/* ... */)?;

    // 如果有檔案，載入並檢測類型
    if let Some(file_path) = args.file {
        editor.load_file(&file_path)?;
        editor.detect_and_set_file_type(&file_path);
    }

    // ... 進入主迴圈
    editor.run()?;

    Ok(())
}
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

### 4.2 大檔案自動降級

對於超大檔案（>10MB 或 >50000 行），自動停用語法高亮：

```rust
impl Editor {
    pub fn load_file(&mut self, path: &Path) -> Result<()> {
        // ... 載入檔案

        // 檢查檔案大小
        let file_size = std::fs::metadata(path)?.len();
        let line_count = self.buffer.line_count();

        // 大檔案自動停用高亮
        if file_size > 10 * 1024 * 1024 || line_count > 50000 {
            self.highlight_engine = None;
            self.highlight_config.enabled = false;
            self.status_message = Some(
                "Syntax highlighting disabled for large file".to_string()
            );
        }

        Ok(())
    }
}
```

### 4.3 增量快取失效

只使受影響的行失效，而不是整個快取：

```rust
impl Editor {
    pub fn on_insert_char(&mut self, ch: char) {
        self.buffer.insert_char(self.cursor.line, self.cursor.column, ch);

        // 只使當前行失效
        self.highlight_cache.invalidate(self.cursor.line);
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

## 五、測試清單

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
- [ ] 大檔案 (>50000 行) 自動停用高亮
- [ ] 捲動時無明顯延遲

### 快取測試
- [ ] 相同行重複渲染使用快取（效能提升）
- [ ] 修改行後快取失效
- [ ] 快取不會無限增長

### 主題測試
- [ ] 切換到 "Monokai Extended"
- [ ] 切換到 "Solarized (dark)"
- [ ] 切換到 "base16-ocean.dark"
- [ ] 無效主題名稱有錯誤處理

---

## 六、疑難排解

### 問題 1: 高亮渲染閃爍

**原因：** 每次按鍵都重繪整個螢幕

**解決方案：**
- 只重繪修改的行
- 使用 crossterm 的 cursor positioning
- 實作髒行追蹤（dirty line tracking）

### 問題 2: 大檔案卡頓

**原因：** 對整個檔案進行高亮

**解決方案：**
- 實作可見區域高亮（Step 4.1）
- 背景執行緒預計算
- 自動降級策略

### 問題 3: 語法狀態不一致

**原因：** 跨行語法（如多行註解）狀態管理錯誤

**解決方案：**
```rust
// 在 LineHighlighter 中維護正確的狀態
pub struct LineHighlighter {
    inner: HighlightLines<'static>,
    // 記錄上一行的結束狀態
    last_state: Option<syntect::parsing::ParseState>,
}
```

### 問題 4: 記憶體使用過高

**原因：** 快取過大

**解決方案：**
- 限制快取大小（已在 cache.rs 實作）
- 實作 LRU 策略
- 定期清理不可見行的快取

---

## 七、階段性實作計劃

### Phase 1: MVP（1-2 天）

**目標：** 基本語法高亮能正常運作

- [x] 整合 syntect 依賴
- [ ] 實作 HighlightEngine
- [ ] 實作 FileType 檢測
- [ ] 修改 View 渲染邏輯
- [ ] 支援 5 種語言（Rust, Python, JS, C, C++）
- [ ] 基本測試

### Phase 2: 完整功能（2-3 天）

**目標：** 支援所有常用語言和主題切換

- [ ] 擴充到 28 種語言支援
- [ ] 實作 HighlightCache
- [ ] 新增快捷鍵（F5, Ctrl+T）
- [ ] 主題切換功能
- [ ] 設定檔整合

### Phase 3: 效能優化（1-2 天）

**目標：** 大檔案流暢編輯

- [ ] 可見區域高亮
- [ ] 快取優化
- [ ] 大檔案自動降級
- [ ] 增量快取失效
- [ ] 效能基準測試

### Phase 4: 使用者體驗（1 天）

**目標：** 改善互動和回饋

- [ ] 狀態列顯示當前主題
- [ ] 狀態列顯示檔案類型
- [ ] 主題預覽（在狀態列顯示）
- [ ] 說明文件更新
- [ ] 快捷鍵說明

---

## 八、設定檔整合

**新建/修改檔案：`src/config.rs`**

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    // ... 現有設定

    // 語法高亮設定
    #[serde(default)]
    pub syntax_highlighting: SyntaxHighlightConfig,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SyntaxHighlightConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    #[serde(default = "default_theme")]
    pub theme: String,

    #[serde(default = "default_large_file_threshold")]
    pub large_file_threshold_mb: u64,
}

fn default_enabled() -> bool { true }
fn default_theme() -> String { "base16-ocean.dark".to_string() }
fn default_large_file_threshold_mb() -> u64 { 10 }

impl Default for SyntaxHighlightConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            theme: "base16-ocean.dark".to_string(),
            large_file_threshold_mb: 10,
        }
    }
}
```

**設定檔範例：`~/.wedirc` 或 `wedi.toml`**

```toml
[syntax_highlighting]
enabled = true
theme = "Monokai Extended"
large_file_threshold_mb = 10
```

---

## 九、文件更新

### README.md 更新

```markdown
## Features

- 基本文字編輯操作
- 撤銷/重做 (Ctrl+Z / Ctrl+Y)
- 複製/貼上 (Ctrl+C / Ctrl+V)
- 搜尋 (Ctrl+F)
- 行號顯示
- **語法高亮 (支援 28+ 種語言)** ← 新增
- **可自訂主題 (18+ 種內建主題)** ← 新增
- 多編碼支援 (UTF-8, GBK, Big5, Shift-JIS 等)
- 跨平台支援 (Windows, macOS, Linux)

## Keyboard Shortcuts

... (現有快捷鍵)

### Syntax Highlighting

- `F5` - Toggle syntax highlighting on/off
- `Ctrl+H` - Toggle syntax highlighting (alternative)
- `Ctrl+T` - Switch to next theme

## Supported Languages

Rust, Python, JavaScript, TypeScript, Go, C, C++, Java, C#,
Bash, PowerShell, Batch, Makefile, Dockerfile,
HTML, CSS, Markdown, JSON, YAML, TOML, XML, INI, SQL, and more.

## Configuration

Create `~/.wedirc` or `wedi.toml`:

```toml
[syntax_highlighting]
enabled = true
theme = "Monokai Extended"
large_file_threshold_mb = 10
```
```

---

## 十、效能基準測試

建立效能測試腳本：

**新建檔案：`benches/highlighting_bench.rs`**

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use wedi::highlight::{HighlightEngine, FileType};

fn bench_rust_highlighting(c: &mut Criterion) {
    let mut engine = HighlightEngine::new(None).unwrap();
    engine.set_file_type(FileType::Rust);
    let mut highlighter = engine.create_highlighter().unwrap();

    let code = "fn main() { println!(\"Hello, world!\"); }";

    c.bench_function("highlight_rust_line", |b| {
        b.iter(|| {
            highlighter.highlight_line(black_box(code))
        })
    });
}

criterion_group!(benches, bench_rust_highlighting);
criterion_main!(benches);
```

在 `Cargo.toml` 中加入：

```toml
[[bench]]
name = "highlighting_bench"
harness = false

[dev-dependencies]
criterion = "0.5"
```

執行基準測試：

```bash
cargo bench
```

---

## 十一、預估時程與資源

| 階段 | 任務 | 預估時間 | 難度 |
|------|------|---------|------|
| Phase 1 | MVP 實作 | 1-2 天 | 中 |
| Phase 2 | 完整功能 | 2-3 天 | 中 |
| Phase 3 | 效能優化 | 1-2 天 | 高 |
| Phase 4 | UX 改善 | 1 天 | 低 |
| **總計** | | **5-8 天** | |

---

## 十二、參考資源

- [syntect 文件](https://docs.rs/syntect/)
- [crossterm 顏色](https://docs.rs/crossterm/latest/crossterm/style/enum.Color.html)
- [bat 原始碼](https://github.com/sharkdp/bat)
- [ropey 文件](https://docs.rs/ropey/) - 文字緩衝區
- [Sublime Text 語法](https://www.sublimetext.com/docs/syntax.html)

---

## 附錄：主題推薦清單

**適合編輯器的主題（視覺友善）：**

**暗色主題：**
- base16-ocean.dark (推薦預設)
- Monokai Extended
- Dracula
- Nord
- OneHalfDark
- Solarized (dark)

**亮色主題：**
- Solarized (light)
- InspiredGitHub
- Monokai Extended Light
- OneHalfLight

---

## 最後檢查清單

實作完成後，請確認：

- [ ] 所有測試通過
- [ ] 無編譯警告
- [ ] 效能符合預期（<100ms 啟動，流暢編輯）
- [ ] 二進位大小合理（<5MB）
- [ ] 文件已更新（README, CHANGELOG）
- [ ] 設定檔支援
- [ ] 快捷鍵說明完整
- [ ] 錯誤處理完善
- [ ] 跨平台測試（Windows, Linux, macOS）

---

**預估總開發時間：5-8 天**
**建議開發順序：Phase 1 → Phase 2 → Phase 4 → Phase 3**

祝實作順利！如遇問題請參考疑難排解章節。

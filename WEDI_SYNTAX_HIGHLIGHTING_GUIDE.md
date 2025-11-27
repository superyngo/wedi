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
- 使用路徑：`D:\Users\user\Documents\rust\bat\assets\syntaxes.bin`
- 授權：MIT License / Apache License 2.0（雙授權）
- 包含 219 種語法定義
- 原始來源：Sublime Text packages (MIT License)

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
crossterm = { version = "0.27", features = ["event-stream"] }
ropey = "1.6"
unicode-width = "0.1"
pico-args = "0.5"
encoding_rs = "0.8"
anyhow = "1.0"

# 新增：語法高亮
syntect = { version = "5.3.0", default-features = false, features = ["parsing", "regex-onig", "default-themes"] }
once_cell = "1.19"
bincode = "1.3"
flate2 = { version = "1.0", optional = true }  # 如果 syntaxes.bin 是壓縮的
ansi_colours = "1.2"  # RGB -> 256 色轉換
serde = "1.0"

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
syntax-highlighting = ["syntect", "once_cell", "bincode", "ansi_colours", "serde"]
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

/// 語法集是否壓縮（與 bat 保持一致）
const COMPRESS_SYNTAXES: bool = false;

/// 全域語法集（延遲載入，使用 bat 的載入方式）
static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(|| {
    load_from_binary(SERIALIZED_SYNTAX_SET, COMPRESS_SYNTAXES)
        .expect("Failed to load embedded syntax set")
});

/// 全域主題集（使用 syntect 內建主題）
static THEME_SET: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

/// 從二進位資料載入（與 bat 的 from_binary 相同邏輯）
fn load_from_binary<T>(data: &[u8], compressed: bool) -> Result<T>
where
    T: for<'de> serde::Deserialize<'de>,
{
    if compressed {
        #[cfg(feature = "flate2")]
        {
            bincode::deserialize_from(flate2::read::ZlibDecoder::new(data))
                .context("Failed to decompress and deserialize")
        }
        #[cfg(not(feature = "flate2"))]
        {
            anyhow::bail!("Compressed syntax sets require flate2 feature")
        }
    } else {
        bincode::deserialize(data).context("Failed to deserialize")
    }
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
    pub fn highlight_line(&mut self, line: &str) -> Result<String> {
        let ranges: Vec<(Style, &str)> = self
            .inner
            .highlight_line(line, &SYNTAX_SET)
            .context("Failed to highlight line")?;

        let escaped = if self.true_color {
            as_24_bit_terminal_escaped(&ranges[..], false)
        } else {
            self.as_8bit_terminal_escaped(&ranges[..])
        };

        Ok(escaped)
    }

    /// 將 syntect 顏色轉為 8-bit ANSI 色碼（相容模式）
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
pub fn supports_true_color() -> bool {
    std::env::var("COLORTERM")
        .map(|v| v == "truecolor" || v == "24bit")
        .unwrap_or(false)
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
        assert!(result.is_ok());
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
#[derive(Clone, Debug)]
pub struct CachedLine {
    /// 原始文字內容（用於驗證）
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

### Step 6: 建立第三方授權文件

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

## 六、與 cate 專案的差異

### 相同點
- ✅ 使用相同的 bat syntaxes.bin（219 種語言）
- ✅ 使用相同的 syntect 配置
- ✅ 使用相同的授權處理方式
- ✅ 支援真彩色和 256 色模式

### 差異點
- ⚠️ **wedi 是編輯器**：需要逐行即時高亮，維護狀態
- ⚠️ **cate 是查看器**：一次性高亮整個檔案
- ⚠️ **wedi 需要快取**：因為編輯時需要重複渲染相同的行
- ⚠️ **wedi 需要狀態管理**：編輯操作會影響高亮狀態

---

## 七、階段性實作計劃

### Phase 1: MVP（1-2 天）

**目標：** 基本語法高亮能正常運作

- [x] 複製 syntaxes.bin 到專案
- [ ] 整合 syntect 依賴（與 cate 相同配置）
- [ ] 實作 HighlightEngine（基於 cate 的實作）
- [ ] 修改 View 渲染邏輯
- [ ] 基本測試（5 種常用語言）

### Phase 2: 完整功能（2-3 天）

**目標：** 支援所有語言和主題切換

- [ ] 驗證所有 219 種語法可用
- [ ] 實作 HighlightCache
- [ ] 新增快捷鍵（F5, Ctrl+T）
- [ ] 主題切換功能
- [ ] 建立 THIRD-PARTY-LICENSES.md

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
- [ ] 主題預覽
- [ ] 說明文件更新
- [ ] 快捷鍵說明

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

- [ ] 所有測試通過
- [ ] 無編譯警告
- [ ] 效能符合預期（<100ms 啟動，流暢編輯）
- [ ] 二進位大小合理（<5MB）
- [ ] 文件已更新（README, THIRD-PARTY-LICENSES.md）
- [ ] syntaxes.bin 已包含在專案中
- [ ] 授權資訊完整
- [ ] 跨平台測試（Windows, Linux, macOS）

---

**預估總開發時間：5-8 天**
**建議開發順序：Phase 1 → Phase 2 → Phase 4 → Phase 3**

**重要：** 此方案與 cate 專案使用相同的技術棧，可參考 cate 的實作細節。主要差異在於 wedi 是編輯器，需要處理即時編輯和狀態管理。

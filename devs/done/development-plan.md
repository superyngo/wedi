# wedi 開發計畫

> **狀態更新**: Phase 1 和 Phase 2 MVP 核心功能已完成！✅  
> **完成日期**: 2025 年 11 月 8 日  
> **詳細進度**: 請參閱 [PROGRESS.md](./PROGRESS.md)

## 專案概述

開發一款跨平台極簡輕量 CLI 文本編輯器,使用 Rust 語言實現,對標 nano/micro,專注於系統管理員在服務器端快速編輯配置文件的需求。

## 開發時程

**預估總開發時間**: 8-10 週
**建議團隊規模**: 1-2 名開發者

## Phase 1: 專案基礎建設 (Week 1) ✅ **已完成**

### 1.1 專案初始化 ✅

- [x] 建立 Cargo 專案結構
- [x] 配置 `Cargo.toml` 依賴項
- [ ] 設定 CI/CD 流程 (GitHub Actions) - 待後續
- [x] 建立開發文檔框架
- [ ] 設定 Rust 編碼規範 (rustfmt, clippy) - 待後續

### 1.2 核心依賴選型 ✅

```toml
[dependencies]
crossterm = "0.27"      # 終端操作
clap = "4.5"            # CLI 參數解析
ropey = "1.6"           # 文本緩衝區
arboard = "3.3"         # 剪貼板
unicode-width = "0.1"   # Unicode 字符寬度計算
log = "0.4"             # 日誌
env_logger = "0.11"     # 日誌實現

[dev-dependencies]
assert_cmd = "2.0"      # CLI 測試
predicates = "3.0"      # 測試斷言
tempfile = "3.8"        # 臨時文件測試
```

### 1.3 架構設計 ✅

```
src/
├── main.rs              # 程式入口、CLI 參數處理 ✅
├── editor.rs            # Editor 主結構體、事件循環 ✅
├── buffer/
│   ├── mod.rs          # 文本緩衝區管理 ✅
│   ├── rope_buffer.rs  # Rope 數據結構封裝 ✅
│   └── history.rs      # 撤銷/重做歷史管理 🚧
├── cursor.rs            # 光標位置管理 ✅
├── view.rs              # 視窗顯示邏輯
├── terminal.rs          # 終端抽象層
├── input/
│   ├── mod.rs          # 輸入處理主模塊
│   ├── keymap.rs       # 快捷鍵映射
│   └── handler.rs      # 輸入事件處理器
├── clipboard.rs         # 剪貼板操作
├── search.rs            # 搜索功能
├── comment.rs           # 註解處理
├── highlight/
│   ├── mod.rs          # 語法高亮主模塊
│   └── detector.rs     # 文件類型檢測
├── config.rs            # 配置管理
└── utils/
    ├── mod.rs
    ├── line_wrapper.rs  # 自動換行邏輯
    └── logger.rs        # 日誌工具
```

### 1.4 交付物

- 完整的專案骨架
- 編譯通過的空白框架
- README.md 基本文檔
- 開發環境配置說明

---

## Phase 2: MVP 核心功能 (Week 2-3)

### 2.1 終端初始化與清理 (2 天)

**目標**: 建立穩定的終端控制基礎

**實現要點**:

```rust
// terminal.rs
pub struct Terminal {
    stdout: Stdout,
    original_size: (u16, u16),
}

impl Terminal {
    pub fn new() -> Result<Self>;
    pub fn enter_raw_mode() -> Result<()>;
    pub fn exit_raw_mode() -> Result<()>;
    pub fn clear_screen() -> Result<()>;
    pub fn size() -> Result<(u16, u16)>;
    pub fn flush() -> Result<()>;
}
```

**測試要點**:

- 進入/退出 raw mode 正常
- Ctrl+C 正確清理資源
- Panic 時正常恢復終端狀態

### 2.2 文本緩衝區 (3 天)

**目標**: 實現高效的文本存儲和操作

**實現要點**:

```rust
// buffer/rope_buffer.rs
pub struct RopeBuffer {
    rope: Rope,
    file_path: Option<PathBuf>,
    modified: bool,
}

impl RopeBuffer {
    pub fn new() -> Self;
    pub fn from_file(path: &Path) -> Result<Self>;
    pub fn insert_char(&mut self, pos: usize, ch: char);
    pub fn delete_char(&mut self, pos: usize);
    pub fn delete_range(&mut self, start: usize, end: usize);  // 刪除選中範圍
    pub fn delete_line(&mut self, row: usize);                 // 刪除整行(Ctrl+D)
    pub fn line_count(&self) -> usize;
    pub fn line(&self, idx: usize) -> Option<RopeSlice>;
    pub fn save(&mut self) -> Result<()>;
}
```

**測試要點**:

- UTF-8 字符正確處理
- 大文件加載測試 (至少 10MB)
- 插入/刪除操作性能測試

### 2.3 光標管理 (2 天)

**目標**: 精確的光標定位和移動

**實現要點**:

```rust
// cursor.rs
pub struct Cursor {
    pub row: usize,        // 邏輯行號 (0-based)
    pub col: usize,        // 邏輯列號 (0-based)
    pub desired_col: usize, // 上下移動時保持的列
}

impl Cursor {
    pub fn move_up(&mut self, buffer: &RopeBuffer);
    pub fn move_down(&mut self, buffer: &RopeBuffer);
    pub fn move_left(&mut self, buffer: &RopeBuffer);
    pub fn move_right(&mut self, buffer: &RopeBuffer);
    pub fn move_to_line_start(&mut self);
    pub fn move_to_line_end(&mut self, buffer: &RopeBuffer);
}
```

**測試要點**:

- 邊界條件 (文件開頭/結尾)
- 空行處理
- Unicode 字符寬度計算

### 2.4 視窗顯示 (3 天)

**目標**: 正確渲染文本到終端

**實現要點**:

```rust
// view.rs
pub struct View {
    pub offset_row: usize,  // 視窗頂部顯示的行號
    pub show_line_numbers: bool,
    screen_rows: usize,
    screen_cols: usize,
}

impl View {
    pub fn render(&self, buffer: &RopeBuffer, cursor: &Cursor) -> Result<()>;
    pub fn scroll_if_needed(&mut self, cursor: &Cursor);
    fn render_status_bar(&self, buffer: &RopeBuffer) -> String;
    fn render_line(&self, line: RopeSlice, line_num: usize) -> String;
}
```

**實現細節**:

- 狀態欄顯示: `檔名 | Line: X/Y | Modified: * | Ctrl+S: Save | Ctrl+Q: Quit`
- 行號寬度自適應 (根據總行數)
- 視窗滾動邏輯

**測試要點**:

- 不同終端尺寸適配
- 滾動流暢性
- 狀態欄資訊正確性

### 2.5 基本輸入處理 (3 天)

**目標**: 響應鍵盤輸入

**實現要點**:

```rust
// input/handler.rs
pub enum Command {
    Insert(char),
    Delete,
    Backspace,
    DeleteLine,        // Ctrl+D
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    MoveHome,
    MoveEnd,
    PageUp,
    PageDown,
    Copy,              // Ctrl+C (智能複製)
    Cut,               // Ctrl+X (智能剪切)
    Paste,             // Ctrl+V
    Save,
    Quit,
    Undo,
    Redo,
    Find,
    ToggleLineNumbers,
    ToggleComment,
    SelectAll,
    GoToLine,
    // 選擇相關
    StartSelection,
    ExtendSelection(Direction),
    ClearSelection,
}

pub enum Direction {
    Up, Down, Left, Right, Home, End, PageUp, PageDown
}

pub fn handle_key_event(event: KeyEvent, has_selection: bool) -> Option<Command>;
```

**快捷鍵行為總結**:

| 快捷鍵           | 有選擇時               | 無選擇時       | 備註             |
| ---------------- | ---------------------- | -------------- | ---------------- |
| `Ctrl+C`         | 複製選中文本           | 複製當前整行   | 包含換行符       |
| `Ctrl+X`         | 剪切選中文本           | 剪切當前整行   | 包含換行符       |
| `Ctrl+V`         | 粘貼並替換選中內容     | 粘貼到當前位置 | -                |
| `Ctrl+D`         | 刪除選中文本           | 刪除當前整行   | -                |
| `Ctrl+/`         | 切換選中行的註解       | 切換當前行註解 | 多行批次操作     |
| 任何字符輸入     | 替換選中內容後取消選擇 | 正常插入       | **自動取消選擇** |
| Backspace/Delete | 刪除選中內容後取消選擇 | 刪除單個字符   | **自動取消選擇** |

> **重要**: 選擇模式只在持續按住 Shift 時維持,任何非 Shift 組合的輸入操作都會自動取消選擇狀態

**支持操作**:

- 字符輸入 (可見字符)
- Backspace / Delete
- 方向鍵移動
- Home / End
- Ctrl+S 保存
- Ctrl+Q 退出
- Ctrl+C / Ctrl+X / Ctrl+D (整行操作)

**測試要點**:

- 特殊字符輸入 (Tab, Enter)
- 快捷鍵正確識別
- 無效輸入忽略

### 2.6 主事件循環 (2 天)

**目標**: 整合所有組件

**實現要點**:

```rust
// editor.rs
pub struct Editor {
    buffer: RopeBuffer,
    cursor: Cursor,
    view: View,
    terminal: Terminal,
    should_quit: bool,
}

impl Editor {
    pub fn run(&mut self) -> Result<()> {
        loop {
            self.view.render(&self.buffer, &self.cursor)?;

            if let Some(command) = self.read_command()? {
                self.handle_command(command)?;
            }

            if self.should_quit {
                break;
            }
        }
        Ok(())
    }
}
```

### 2.7 MVP 交付物

- 可以打開文件並顯示
- 光標移動流暢
- 文本輸入/刪除正常
- 保存文件成功
- 正常退出

**驗收測試**:

```bash
# 測試場景 1: 打開現有文件
wedi test.txt

# 測試場景 2: 創建新文件
wedi newfile.txt

# 測試場景 3: 編輯並保存
# 1. 輸入文字
# 2. Ctrl+S 保存
# 3. Ctrl+Q 退出
# 4. 重新打開驗證內容

# 測試場景 4: 大文件
wedi large.log  # 10MB+ 文件
```

---

## Phase 3: 進階編輯功能 (Week 4-6)

### 3.1 選擇模式 (4 天)

**實現要點**:

```rust
// cursor.rs 擴展
pub struct Selection {
    pub start: (usize, usize),  // (row, col)
    pub end: (usize, usize),
}

impl Cursor {
    pub fn start_selection(&mut self);
    pub fn clear_selection(&mut self);
    pub fn get_selected_text(&self, buffer: &RopeBuffer) -> Option<String>;
}
```

**支持操作**:

- Shift + 方向鍵選擇
- Shift + Home/End 選擇到行首/尾
- Shift + Page Up/Down 選擇整頁
- Ctrl+A 全選

**選擇模式行為**:

1. **自動取消選擇**:
   - 只要沒有繼續按著 Shift,任何輸入(包括字符輸入、Backspace、Delete 等)都會自動取消並清除所有選擇
   - 輸入會替換選中內容並取消選擇狀態
2. **多行註解切換**:
   - 選擇模式下按 Ctrl+/ 會對所有選中行進行註解切換
   - 如果選中行全部已註解,則取消註解
   - 如果選中行部分或全部未註解,則全部添加註解

**實現要點**:

```rust
impl Editor {
    fn handle_input_with_selection(&mut self, ch: char) {
        if self.cursor.has_selection() {
            // 刪除選中內容
            self.delete_selection();
            // 清除選擇狀態
            self.cursor.clear_selection();
        }
        // 插入字符
        self.insert_char(ch);
    }
}
```

**視覺反饋**: 選中文本反色顯示

**測試要點**:

- Shift+方向鍵選擇正確
- 選擇範圍視覺反饋清晰
- 輸入字符立即取消選擇
- Backspace/Delete 取消選擇
- 多行選擇時 Ctrl+/ 批次註解
- 選擇狀態下方向鍵(不按 Shift)取消選擇

### 3.2 剪貼板操作 (2 天)

**實現要點**:

```rust
// clipboard.rs
pub struct Clipboard {
    backend: Box<dyn ClipboardProvider>,
}

impl Clipboard {
    pub fn new() -> Result<Self>;
    pub fn set_text(&mut self, text: &str) -> Result<()>;
    pub fn get_text(&mut self) -> Result<String>;
}
```

**支持操作**:

**選擇模式下**:

- Ctrl+C 複製選中文本
- Ctrl+X 剪切選中文本
- Ctrl+V 粘貼(會替換選中內容)

**非選擇模式下**:

- Ctrl+C 複製當前整行(包含換行符)
- Ctrl+X 剪切當前整行(包含換行符)
- Ctrl+V 粘貼到當前位置
- Ctrl+D 刪除當前整行

**實現細節**:

```rust
impl Editor {
    fn handle_copy(&mut self) {
        let text = if self.cursor.has_selection() {
            // 選擇模式: 複製選中文本
            self.cursor.get_selected_text(&self.buffer)
        } else {
            // 非選擇模式: 複製整行
            self.buffer.line(self.cursor.row).map(|line| {
                format!("{}\n", line)
            })
        };

        if let Some(text) = text {
            self.clipboard.set_text(&text).ok();
        }
    }

    fn handle_cut(&mut self) {
        if self.cursor.has_selection() {
            // 選擇模式: 剪切選中文本
            let text = self.cursor.get_selected_text(&self.buffer);
            self.clipboard.set_text(&text).ok();
            self.delete_selection();
        } else {
            // 非選擇模式: 剪切整行
            let line = self.buffer.line(self.cursor.row);
            if let Some(line_text) = line {
                self.clipboard.set_text(&format!("{}\n", line_text)).ok();
                self.buffer.delete_line(self.cursor.row);
            }
        }
    }

    fn handle_delete_line(&mut self) {
        // Ctrl+D: 刪除當前整行
        self.buffer.delete_line(self.cursor.row);
        // 光標保持在同一行號(如果還有後續行)或移到上一行
        if self.cursor.row >= self.buffer.line_count() {
            self.cursor.row = self.cursor.row.saturating_sub(1);
        }
    }
}
```

**測試要點**:

- 跨平台剪貼板正常工作
- 與系統剪貼板互通
- 大段文本複製粘貼
- **非選擇模式下整行操作正確**
- **選擇與非選擇模式行為區分清晰**

### 3.3 撤銷/重做系統 (5 天)

**實現要點**:

```rust
// buffer/history.rs
pub enum Action {
    Insert { pos: usize, text: String },
    Delete { pos: usize, text: String },
    // 批次操作
    Batch(Vec<Action>),
}

pub struct History {
    undos: Vec<Action>,
    redos: Vec<Action>,
    max_history: usize,
}

impl History {
    pub fn record(&mut self, action: Action);
    pub fn undo(&mut self) -> Option<Action>;
    pub fn redo(&mut self) -> Option<Action>;
}
```

**優化點**:

- 連續輸入合併為單個操作
- 歷史記錄限制 (預設 1000 條)
- 內存使用控制

### 3.4 搜索功能 (4 天)

**實現要點**:

```rust
// search.rs
pub struct Search {
    query: String,
    matches: Vec<(usize, usize)>,  // (row, col)
    current_match: usize,
}

impl Search {
    pub fn new(query: &str, buffer: &RopeBuffer) -> Self;
    pub fn next_match(&mut self, cursor: &mut Cursor);
    pub fn prev_match(&mut self, cursor: &mut Cursor);
    pub fn match_count(&self) -> usize;
}
```

**UI 流程**:

1. Ctrl+F 進入搜索模式
2. 顯示輸入框: `Search: [____]`
3. 即時高亮匹配結果
4. Enter 跳轉下一個, Shift+Enter 上一個
5. Esc 退出搜索

**功能**:

- 大小寫不敏感搜索
- 顯示匹配數量
- 高亮當前匹配項

### 3.5 行號切換 (1 天)

**實現要點**:

```rust
// view.rs 擴展
impl View {
    pub fn toggle_line_numbers(&mut self) {
        self.show_line_numbers = !self.show_line_numbers;
        self.recalculate_layout();
    }
}
```

**細節**:

- Ctrl+L 切換
- 行號寬度動態調整
- 保持光標視覺位置不變

### 3.6 跳轉到行 (2 天)

**實現要點**:

```rust
// input/handler.rs 擴展
pub enum Command {
    // ...
    GoToLine(usize),
}
```

**UI 流程**:

1. Ctrl+G 進入跳轉模式
2. 顯示提示: `Go to line: [____]`
3. 輸入行號 (只允許數字)
4. Enter 確認跳轉
5. 錯誤處理: 超出範圍提示

### 3.7 Phase 3 交付物

- 完整的文本選擇功能
- 剪貼板無縫集成
- 可靠的撤銷/重做
- 實用的搜索功能

**驗收測試**:

```bash
# 測試撤銷/重做
1. 輸入多行文字
2. Ctrl+Z 撤銷到空白
3. Ctrl+Y 重做恢復
4. 驗證內容完整

# 測試搜索
1. 打開有重複內容的文件
2. Ctrl+F 搜索關鍵字
3. 驗證高亮和跳轉
4. 確認匹配計數正確

# 測試剪貼板與選擇
## 場景 1: 選擇模式操作
1. Shift+方向鍵選擇多行文本
2. Ctrl+C 複製選中文本
3. 移動到其他位置 Ctrl+V 粘貼
4. 驗證粘貼內容正確

## 場景 2: 非選擇模式整行操作
1. 光標定位到某行(不選擇)
2. Ctrl+C 複製整行
3. 移動到其他位置 Ctrl+V 粘貼
4. 驗證整行(含換行符)被複製

## 場景 3: 整行刪除
1. 光標定位到某行
2. Ctrl+D 刪除整行
3. 驗證該行被刪除,後續行上移

## 場景 4: 選擇自動取消
1. Shift+方向鍵選擇文本
2. 直接輸入字符(不按 Shift)
3. 驗證選中內容被替換,選擇狀態取消

## 場景 5: 多行註解切換
1. Shift+方向鍵選擇多行代碼
2. Ctrl+/ 切換註解
3. 驗證所有選中行都被註解
4. 再次 Ctrl+/ 驗證取消註解

## 場景 6: 單行註解切換
1. 光標定位到某行(不選擇)
2. Ctrl+/ 切換註解
3. 驗證該行註解狀態切換

# 測試與系統剪貼板互通
1. 在 wedi 中複製文本
2. 在其他應用(瀏覽器/記事本)粘貼驗證
3. 從其他應用複製
4. Ctrl+V 粘貼到 wedi 驗證
```

---

## Phase 4: 註解與語法高亮 (Week 7-8)

### 4.1 文件類型檢測 (2 天)

**實現要點**:

```rust
// highlight/detector.rs
pub enum FileType {
    Rust,
    Python,
    JavaScript,
    Go,
    Shell,
    Config,   // .conf, .ini, .toml, .yaml
    Plain,
}

impl FileType {
    pub fn from_path(path: &Path) -> Self;
    pub fn comment_syntax(&self) -> CommentSyntax;
}

pub struct CommentSyntax {
    pub line_comment: Option<&'static str>,  // "//" or "#"
    pub block_comment: Option<(&'static str, &'static str)>,  // ("/*", "*/")
}
```

**支持文件類型** (基於副檔名):

- `.rs` → Rust (`//`, `/* */`)
- `.py` → Python (`#`)
- `.js, .ts` → JavaScript (`//`, `/* */`)
- `.go` → Go (`//`, `/* */`)
- `.sh` → Shell (`#`)
- `.toml, .yaml, .conf` → Config (`#`)

### 4.2 註解切換功能 (3 天)

**實現要點**:

```rust
// comment.rs
pub struct CommentToggler {
    file_type: FileType,
}

impl CommentToggler {
    pub fn toggle_line(&self, buffer: &mut RopeBuffer, row: usize);
    pub fn toggle_selection(&self, buffer: &mut RopeBuffer, selection: &Selection);
}
```

**邏輯**:

- 單行: 檢測行首是否有註解符,有則移除,無則添加
- 多行選擇: 全部註解則取消,否則全部添加註解
- 保持縮進對齊

**示例**:

```rust
// Before (Ctrl+/)
fn main() {
    println!("Hello");
}

// After
// fn main() {
//     println!("Hello");
// }
```

### 4.3 簡單語法高亮 (4 天)

**實現要點**:

```rust
// highlight/mod.rs
pub struct Highlighter {
    file_type: FileType,
}

impl Highlighter {
    pub fn highlight_line(&self, line: &str) -> Vec<Span>;
}

pub struct Span {
    pub text: String,
    pub style: Style,
}

pub enum Style {
    Normal,
    Comment,
    Keyword,
    String,
    Number,
}
```

**高亮規則** (簡化版):

- **註解**: 行註解/區塊註解 → 灰色
- **關鍵字**: 語言保留字 → 藍色/紫色
- **字串**: 雙引號/單引號內容 → 綠色
- **數字**: 數字字面量 → 黃色

**實現建議**:

- 使用正則表達式匹配
- 優先實現註解高亮 (最重要)
- 關鍵字用 HashSet 查找
- 考慮性能: 只高亮可見行

**替代方案**:
如果時間緊張,可以只實現**註解高亮**,其他語法高亮標記為 Future Work

### 4.4 Phase 4 交付物

- 自動識別常見文件類型
- Ctrl+/ 快速註解切換
- 註解部分明顯高亮

---

## Phase 5: 優化與發布 (Week 9-10)

### 5.1 性能優化 (3 天)

**優化方向**:

1. **渲染優化**

   - 差分渲染: 只重繪變化的行
   - 雙緩衝: 避免閃爍
   - 延遲渲染: 輸入時降低渲染頻率

2. **大文件處理**

   - 延遲加載: 按需加載行
   - 虛擬滾動: 只渲染可見區域
   - 內存映射: 超大文件使用 mmap

3. **啟動速度**
   - 減少依賴編譯
   - 優化初始化流程

**基準測試**:

```bash
# 啟動時間 < 100ms
hyperfine 'wedi large.txt'

# 編輯響應 < 16ms (60fps)
# 大文件滾動流暢 (10MB+)
```

### 5.2 錯誤處理 (2 天)

**完善場景**:

- 文件不存在 → 創建新文件提示
- 無讀取權限 → 只讀模式
- 無寫入權限 → 保存失敗提示
- 磁盤空間不足 → 警告
- 非 UTF-8 文件 → 拒絕打開或轉換

**用戶友好提示**:

```
Error: Permission denied
Hint: Try running with sudo or check file permissions
```

### 5.3 配置系統 (2 天)

**配置文件**: `~/.wedi/config.toml`

```toml
[display]
show_line_numbers = true
tab_width = 4
theme = "default"

[editor]
auto_indent = true
max_undo_history = 1000

[keybindings]
# 預留自定義快捷鍵擴展
```

**實現**:

```rust
// config.rs
#[derive(Deserialize)]
pub struct Config {
    pub display: DisplayConfig,
    pub editor: EditorConfig,
}

impl Config {
    pub fn load() -> Result<Self>;
    pub fn default() -> Self;
}
```

### 5.4 調試模式 (1 天)

**實現**:

```rust
// main.rs
if args.debug {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();
}

// 使用
log::debug!("Cursor position: {:?}", cursor);
log::error!("Failed to save file: {}", e);
```

**輸出到**: `~/.wedi/debug.log`

### 5.5 文檔完善 (2 天)

**README.md**:

- 功能介紹
- 安裝說明 (各平台)
- 快捷鍵列表
- 常見問題

**User Guide**:

- 基本使用教學
- 進階功能說明
- 故障排除

**開發文檔**:

- 架構設計說明
- 貢獻指南
- API 文檔 (rustdoc)

### 5.6 跨平台測試 (3 天)

**測試矩陣**:
| OS | 終端 | 測試內容 |
|---|---|---|
| Windows 10/11 | PowerShell, CMD, Windows Terminal | 基本功能, 剪貼板, 快捷鍵 |
| macOS 12+ | Terminal.app, iTerm2 | 基本功能, 剪貼板 |
| Linux (Ubuntu) | gnome-terminal, xterm | 基本功能, SSH 遠程 |

**自動化測試**:

```rust
// tests/integration_test.rs
#[test]
fn test_open_and_edit() {
    let mut cmd = Command::cargo_bin("wedi").unwrap();
    cmd.arg("test.txt")
       .assert()
       .success();
}
```

### 5.7 編譯與發布 (2 天)

**編譯配置**:

```toml
[profile.release]
opt-level = 'z'        # 優化體積
lto = true             # 鏈接時優化
codegen-units = 1      # 單編譯單元
strip = true           # 移除符號表
```

**多平台編譯**:

```bash
# Windows
cargo build --release --target x86_64-pc-windows-msvc

# macOS
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# Linux
cargo build --release --target x86_64-unknown-linux-musl
```

**打包**:

- 建立 GitHub Release
- 提供各平台二進制檔案
- 撰寫 Release Notes
- 生成 SHA256 校驗和

### 5.8 Phase 5 交付物

- 優化後的穩定版本
- 完整文檔
- 跨平台測試報告
- v0.1.0 Release

---

## 測試策略

### 單元測試

```rust
// 每個模塊都應有單元測試
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_move_down() {
        // ...
    }
}
```

**覆蓋率目標**: > 70%

### 集成測試

```bash
tests/
├── cli_test.rs          # CLI 參數測試
├── editing_test.rs      # 編輯功能測試
└── file_ops_test.rs     # 文件操作測試
```

### 手動測試清單

- [ ] 打開各類型文件
- [ ] 輸入特殊字符 (emoji, CJK)
- [ ] 極端文件大小 (0 bytes, 100MB)
- [ ] 網絡文件系統
- [ ] 終端尺寸調整
- [ ] 長時間運行穩定性

---

## 風險管理

### 技術風險

| 風險             | 影響 | 緩解措施                          |
| ---------------- | ---- | --------------------------------- |
| 跨平台兼容性問題 | 高   | 早期在各平台測試,使用成熟的 crate |
| 性能不符預期     | 中   | 預留優化時間,建立基準測試         |
| 剪貼板整合失敗   | 中   | 提供 fallback 方案(內部剪貼板)    |
| 語法高亮複雜度   | 低   | 簡化實現或降級到僅註解高亮        |

### 時程風險

**應對措施**:

- MVP 優先: 確保核心功能完成
- 功能降級: 語法高亮 → 僅註解高亮
- 並行開發: 文檔與編碼同步進行

---

## 成功指標

### 功能指標

- [ ] 支持基本文本編輯操作
- [ ] 跨平台運行無阻礙
- [ ] 啟動時間 < 100ms
- [ ] 支持 100MB 文件流暢編輯
- [ ] 撤銷/重做正常工作
- [ ] 剪貼板與系統互通

### 質量指標

- [ ] 無致命 Bug
- [ ] 代碼覆蓋率 > 70%
- [ ] 通過全平台測試
- [ ] 文檔完整清晰

### 用戶體驗指標

- [ ] 操作響應 < 16ms
- [ ] 快捷鍵符合直覺
- [ ] 錯誤提示友好
- [ ] 學習成本低 (< 5min 上手)

---

## 後續規劃 (v0.2.0+)

### 可能的增強功能

- [ ] 完整語法高亮 (使用 syntect)
- [ ] 多文件編輯 (tabs)
- [ ] 分屏顯示
- [ ] 正則表達式搜索
- [ ] 搜索替換功能
- [ ] 宏錄製/重放
- [ ] 插件系統
- [ ] 主題自定義
- [ ] LSP 整合 (代碼補全)

### 社群建設

- [ ] GitHub Discussions 開放
- [ ] 收集用戶反饋
- [ ] 建立貢獻者文檔
- [ ] 設計 Roadmap 投票機制

---

## 參考資源

### 學習資料

- [crossterm 文檔](https://docs.rs/crossterm/)
- [ropey 文檔](https://docs.rs/ropey/)
- [Building a Text Editor in Rust](https://www.flenker.blog/hecto/)
- [The Craft of Text Editing](http://www.finseth.com/craft/)

### 對標產品分析

- **nano**: 簡單但功能有限
- **micro**: 功能豐富但體積較大
- **wedi 定位**: 介於兩者之間,專注快速編輯

### 工具

- `hyperfine` - 性能基準測試
- `cargo-criterion` - 微基準測試
- `cargo-flamegraph` - 性能分析
- `cargo-bloat` - 分析二進制體積

---

## 檢查點 (Milestones)

### M1: 專案就緒 (Week 1 結束)

✓ 專案結構建立
✓ 依賴配置完成
✓ CI/CD 運作

### M2: MVP 可用 (Week 3 結束)

✓ 可打開文件並編輯
✓ 基本導航功能
✓ 保存/退出正常

### M3: 核心完整 (Week 6 結束)

✓ 選擇/剪貼板功能
✓ 撤銷/重做穩定
✓ 搜索功能可用

### M4: 功能完備 (Week 8 結束)

✓ 註解切換
✓ 語法高亮
✓ 所有需求功能實現

### M5: 發布就緒 (Week 10 結束)

✓ 性能優化完成
✓ 跨平台測試通過
✓ 文檔完整
✓ v0.1.0 Release

---

## 每日站會建議

**時間**: 每天 15 分鐘

**討論內容**:

1. 昨日完成了什麼?
2. 今日計畫做什麼?
3. 遇到什麼阻礙?

**每週回顧** (Friday):

- 本週進度檢視
- 調整下週計畫
- 風險識別

---

## 附錄: 快速啟動清單

### 開發環境設置

```bash
# 1. 安裝 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. 安裝工具
cargo install cargo-watch
cargo install cargo-edit

# 3. Clone 專案
git clone <repo>
cd wedi

# 4. 首次編譯
cargo build

# 5. 開發模式運行
cargo watch -x run

# 6. 測試
cargo test
```

### 開發工作流

```bash
# 修改代碼
vim src/editor.rs

# 運行測試
cargo test

# 檢查格式
cargo fmt --check

# 檢查 lint
cargo clippy

# 運行程式
cargo run -- test.txt

# 提交
git add .
git commit -m "feat: implement cursor movement"
git push
```

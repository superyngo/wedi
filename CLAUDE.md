# wedi - Claude 專案概覽

## 專案簡介

**wedi** 是一個用 Rust 編寫的輕量級終端文本編輯器，支援語法高亮、多語言註解、搜尋、撤銷/重做等豐富功能。專案採用模組化架構，分為核心庫和可嵌入 widget 兩部分。

## 專案架構

### 整體結構

```
wedi/
├── src/                    # 主應用程式
│   ├── main.rs            # 程式入口點，處理命令行參數
│   ├── editor.rs          # 編輯器主循環與事件處理
│   ├── dialog.rs          # 對話框（搜尋、跳行等）
│   └── help.rs            # 快捷鍵幫助顯示
│
├── crates/
│   ├── wedi-core/         # 核心編輯器邏輯 (可獨立使用)
│   └── wedi-widget/       # TUI Widget (可嵌入其他應用)
│
└── assets/
    └── syntaxes.bin       # 語法定義 (219+ 語言)
```

### Crate 職責劃分

#### 1. **wedi-core** - 核心編輯器邏輯

提供編輯器的底層功能，可被其他專案重用。

**模組結構：**
```
wedi-core/
├── buffer/              # 文本緩衝區
│   ├── rope_buffer.rs   # 基於 ropey 的文本處理
│   └── history.rs       # 撤銷/重做歷史管理
├── cursor.rs            # 游標位置管理
├── view.rs              # 視圖渲染與滾動
├── keymap/              # 快捷鍵與命令
│   ├── command.rs       # 編輯器命令定義
│   └── bindings.rs      # 按鍵綁定邏輯
├── search.rs            # 搜尋功能
├── comment.rs           # 多語言註解支援
├── clipboard.rs         # 剪貼簿集成
├── highlight/           # 語法高亮 (可選功能)
│   ├── engine.rs        # 高亮引擎
│   └── cache.rs         # 高亮緩存
├── terminal.rs          # 終端控制抽象
└── utils/               # 工具函數
    ├── ansi_slice.rs    # ANSI 文本處理
    └── line_wrapper.rs  # 行換行邏輯
```

**核心類型：**
- `RopeBuffer` - 高效文本緩衝區（支援大文件）
- `Cursor` - 游標管理（行/列位置）
- `View` - 視圖渲染（滾動、行號、語法高亮）
- `Keymap` - 可自訂快捷鍵系統
- `Command` - 編輯器命令枚舉
- `Search` - 搜尋狀態與匹配
- `HighlightEngine` - 語法高亮引擎（syntect）

**特性 (Cargo Features):**
- `syntax-highlighting` - 啟用語法高亮功能

#### 2. **wedi-widget** - 可嵌入 Widget

提供可在其他 TUI 應用中嵌入的編輯器元件。

**模組結構：**
```
wedi-widget/
├── config.rs            # 編輯器配置
├── layout/              # 佈局管理
│   ├── screen.rs        # 螢幕佈局
│   └── line_layout.rs   # 行佈局
└── lib.rs               # 公開 API 導出
```

**公開 API：**
- `EditorConfig` - 配置選項（行號、Tab寬度、主題）
- `ScreenLayout` - 螢幕尺寸與滾動管理
- `LineLayout` - 行佈局處理
- 重新導出 `wedi-core` 的所有核心類型

#### 3. **wedi (主應用)** - 獨立編輯器

整合核心功能，提供完整的命令行編輯器。

**主要文件：**
- `main.rs` - 命令行參數解析、編碼設定、主題選擇
- `editor.rs` - 編輯器主循環、按鍵事件處理、狀態管理
- `dialog.rs` - 搜尋對話框、跳行對話框
- `help.rs` - 快捷鍵幫助畫面

## 程式邏輯流程

### 1. 啟動流程

```
main.rs
  ├─ 解析命令行參數 (clap)
  ├─ 載入編碼設定 (--from-encoding, --to-encoding)
  ├─ 載入主題設定 (--theme)
  ├─ 初始化 Terminal (crossterm)
  ├─ 建立 RopeBuffer
  │   ├─ 從檔案載入 (from_file_with_encoding)
  │   └─ 自動偵測編碼 (UTF-8, GBK, Big5 等)
  ├─ 初始化語法高亮引擎 (syntect)
  └─ 進入 editor::run() 主循環
```

### 2. 編輯器主循環 (editor.rs)

```
loop {
  ├─ 渲染畫面 (View::render)
  │   ├─ 計算視覺行佈局 (LineLayout)
  │   ├─ 應用語法高亮 (HighlightEngine)
  │   ├─ 渲染行號、選擇區域
  │   └─ 渲染狀態欄
  │
  ├─ 讀取輸入事件 (Terminal::read_input)
  │   ├─ 按鍵事件 (KeyEvent)
  │   └─ 貼上事件 (Bracketed Paste)
  │
  ├─ 轉換為命令 (Keymap::get_command)
  │
  ├─ 執行命令 (execute_command)
  │   ├─ 文本編輯 (Insert, Delete, Backspace)
  │   ├─ 游標移動 (Move*, Jump*)
  │   ├─ 選擇操作 (Select*, Copy, Cut, Paste)
  │   ├─ 搜尋 (Find, FindNext, FindPrev)
  │   ├─ 註解切換 (ToggleComment)
  │   ├─ 撤銷/重做 (Undo, Redo)
  │   └─ 儲存/退出 (Save, Quit)
  │
  └─ 更新視圖 (View::scroll_if_needed)
}
```

### 3. 文本編輯流程

```
使用者輸入 → Keymap → Command → RopeBuffer
                                      ├─ 修改 Rope 結構
                                      ├─ 記錄到 History
                                      ├─ 標記為 modified
                                      └─ 使 View cache 失效

儲存時：
RopeBuffer::save()
  ├─ 讀取 Rope 內容
  ├─ 轉換編碼 (encoding-rs)
  └─ 寫入檔案
```

### 4. 語法高亮流程

```
大文件策略 (>500 行):
  ├─ 只處理可見區域 ± 100 行緩衝
  ├─ 使用 HighlightCache 快取結果
  └─ 滾動時增量更新

小文件策略 (≤500 行):
  ├─ 一次性處理整個文件
  └─ 快取所有行的高亮結果

渲染時：
View::render()
  ├─ 檢查 HighlightCache
  ├─ 命中 → 直接使用 ANSI 字串
  └─ 未命中 → HighlightEngine::highlight() → 快取
```

### 5. 搜尋流程

```
Ctrl+F → 顯示搜尋對話框
  ├─ 使用者輸入關鍵字
  ├─ Search::set_query()
  ├─ Search::find_matches()
  │   └─ 遍歷所有行，記錄匹配位置
  ├─ 進入搜尋模式
  └─ Ctrl+N/Ctrl+P → 跳轉匹配項
```

### 6. 撤銷/重做流程

```
History 結構:
  ├─ actions: Vec<Action>
  ├─ current: usize
  └─ Action { Insert | Delete { pos, text } }

Undo (Ctrl+Z):
  ├─ 取得 current 之前的 Action
  ├─ 反向操作 (Insert ↔ Delete)
  ├─ current -= 1
  └─ 返回游標位置

Redo (Ctrl+Y):
  ├─ 取得 current 的 Action
  ├─ 正向操作
  ├─ current += 1
  └─ 返回游標位置
```

## 技術特色

### 1. 高效文本處理
- **ropey** - Rope 資料結構，O(log n) 插入/刪除
- 支援大文件編輯（GB 級）
- 增量式語法高亮

### 2. 跨平台支援
- **crossterm** - 統一的終端 API
- Windows、Linux、macOS 一致體驗
- 自動編碼偵測與轉換

### 3. 中文友善
- Unicode 寬度計算 (unicode-width)
- 正確處理 CJK 字元顯示
- 支援 GBK、Big5 等編碼

### 4. 智慧語法高亮
- 219+ 語言支援 (bat 專案語法定義)
- 自動檔案類型偵測
- 7 種內建主題
- 快取優化，大文件流暢渲染

### 5. 靈活的快捷鍵系統
- 可自訂綁定 (Keymap)
- 選擇模式覆寫 (bind_selection)
- Shift/Ctrl+S 雙模式支援（相容不同終端）

## 依賴關係

### 核心依賴
- `ropey` - 文本緩衝區
- `crossterm` - 終端控制
- `arboard` - 剪貼簿
- `unicode-width` - Unicode 寬度計算
- `encoding_rs` - 編碼轉換

### 語法高亮 (可選)
- `syntect` - 語法高亮引擎
- `bat` 的 syntaxes.bin - 語法定義

### CLI 工具
- `clap` - 命令行解析
- `anyhow` - 錯誤處理

## 開發指南

### 編譯與測試
```bash
# 開發編譯
cargo build

# 發布編譯 (優化)
cargo build --release

# 執行測試
cargo test --workspace

# 生成文檔
cargo doc --open

# 檢查 widget
cargo check --package wedi-widget
```

### 新增功能建議流程
1. **核心功能** → 加入 `wedi-core`
2. **配置選項** → 加入 `EditorConfig` 和命令行參數
3. **快捷鍵** → 加入 `Command` 枚舉和 `Keymap` 綁定
4. **UI 整合** → 在 `editor.rs` 實作命令執行
5. **文檔** → 更新 CHANGELOG 和 README

### 版本發布流程
1. 更新 `Cargo.toml` 版本號（三個 crate）
2. 整理 `CHANGELOG.md` 的 `[Unreleased]` 區段
3. 提交變更並打 tag
4. GitHub Actions 自動建置跨平台 binary
5. 發布 GitHub Release

## 未來優化方向

### 短期
- [ ] 增強 widget 文檔與使用範例
- [ ] 新增更多主題選項
- [ ] 改進大文件效能

### 長期
- [ ] LSP (Language Server Protocol) 支援
- [ ] 多游標編輯
- [ ] 分割視窗
- [ ] 外掛系統

## 相關連結

- **專案倉庫**: https://github.com/superyngo/wedi
- **文檔**: https://docs.rs/wedi-widget
- **問題回報**: https://github.com/superyngo/wedi/issues

---

*本文件由 Claude 協助維護，記錄專案核心邏輯與架構設計，方便開發者快速理解專案全貌。*

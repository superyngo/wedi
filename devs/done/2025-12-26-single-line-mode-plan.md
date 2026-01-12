# 單行/多行顯示切換開發計畫

## 概述

整合行號顯示與換行模式，透過 `Ctrl+L` 一起切換：
- **顯示行號** → 多行換行模式（wrap mode）
- **關閉行號** → 單行水平滾動模式（no-wrap mode）

## 現有架構分析

### 相關檔案

| 檔案 | 用途 |
|------|------|
| `src/view.rs` | 視圖渲染、行佈局、Selection 高亮 |
| `src/cursor.rs` | 游標移動邏輯 |
| `src/editor.rs` | 命令處理、Selection 管理 |
| `src/input/handler.rs` | 按鍵映射（Ctrl+L） |

### 核心資料結構

```rust
// view.rs
pub struct View {
    pub offset_row: usize,      // 垂直偏移
    pub show_line_numbers: bool, // 行號顯示開關
    pub screen_rows: usize,
    pub screen_cols: usize,
    line_layout_cache: Vec<Option<LineLayout>>,
}

pub struct LineLayout {
    pub visual_lines: Vec<String>,    // 換行後的視覺行
    pub visual_height: usize,
    pub logical_to_visual: Vec<usize>, // 邏輯列→視覺列映射
}

// cursor.rs
pub struct Cursor {
    pub row: usize,
    pub col: usize,
    pub visual_line_index: usize,  // 當前視覺行索引
    pub desired_visual_col: usize,
}
```

---

## 開發階段

### Phase 1: 基礎架構 (預計 1-2 小時)

#### 1.1 新增模式控制

**檔案：`src/view.rs`**

```rust
pub struct View {
    pub offset_row: usize,
    pub offset_col: usize,       // 新增：水平偏移（單行模式用）
    pub show_line_numbers: bool,
    pub wrap_mode: bool,         // 新增：換行模式
    // ...
}

impl View {
    pub fn new(terminal: &Terminal) -> Self {
        Self {
            offset_row: 0,
            offset_col: 0,        // 初始化
            show_line_numbers: true,
            wrap_mode: true,      // 預設啟用換行
            // ...
        }
    }

    pub fn toggle_line_numbers(&mut self) {
        self.show_line_numbers = !self.show_line_numbers;
        self.wrap_mode = self.show_line_numbers;  // 連動切換
        self.offset_col = 0;  // 重置水平偏移
        self.invalidate_cache();
    }
}
```

#### 1.2 修改 LineLayout 生成

**檔案：`src/view.rs`**

```rust
impl LineLayout {
    pub fn new(buffer: &RopeBuffer, row: usize, available_width: usize, wrap: bool) -> Option<Self> {
        // ... 現有邏輯 ...
        
        let visual_lines = if wrap {
            wrap_line(&displayed_line, available_width)
        } else {
            vec![displayed_line]  // 單行模式：不切分
        };
        
        // ...
    }
}
```

#### 1.3 任務清單

- [ ] 新增 `offset_col` 欄位
- [ ] 新增 `wrap_mode` 欄位
- [ ] 修改 `toggle_line_numbers()` 連動切換
- [ ] 修改 `LineLayout::new()` 支援 wrap 參數
- [ ] 更新所有 `LineLayout::new()` 呼叫點

---

### Phase 2: 水平滾動 (預計 2-3 小時)

#### 2.1 水平滾動邏輯

**檔案：`src/view.rs`**

```rust
const HORIZONTAL_SCROLL_MARGIN: usize = 5;  // 邊界預留

impl View {
    /// 水平滾動（單行模式專用）
    pub fn scroll_horizontal_if_needed(&mut self, cursor: &Cursor, buffer: &RopeBuffer) {
        if self.wrap_mode {
            self.offset_col = 0;
            return;
        }

        let available_width = self.get_available_width(buffer);
        
        // 計算游標的視覺列
        let line = buffer.line(cursor.row).map(|s| s.to_string()).unwrap_or_default();
        let line = line.trim_end_matches(['\n', '\r']);
        let cursor_visual_col = self.logical_col_to_visual_col(line, cursor.col);

        // 游標超出右邊界
        if cursor_visual_col >= self.offset_col + available_width - HORIZONTAL_SCROLL_MARGIN {
            self.offset_col = cursor_visual_col.saturating_sub(available_width - HORIZONTAL_SCROLL_MARGIN - 1);
        }
        
        // 游標超出左邊界
        if cursor_visual_col < self.offset_col + HORIZONTAL_SCROLL_MARGIN {
            self.offset_col = cursor_visual_col.saturating_sub(HORIZONTAL_SCROLL_MARGIN);
        }
    }
}
```

#### 2.2 渲染時截取可見區間

**檔案：`src/view.rs` - `render()` 方法**

```rust
// 單行模式：截取可見部分
let display_text = if self.wrap_mode {
    visual_line.clone()
} else {
    self.slice_visible_text(visual_line, self.offset_col, available_width)
};

/// 截取可見文字（處理中文寬度）
fn slice_visible_text(&self, text: &str, start_col: usize, width: usize) -> String {
    let mut result = String::new();
    let mut current_col = 0;
    
    for ch in text.chars() {
        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(1);
        
        // 跳過 offset 之前的字符
        if current_col + ch_width <= start_col {
            current_col += ch_width;
            continue;
        }
        
        // 超出可見範圍則停止
        if current_col >= start_col + width {
            break;
        }
        
        result.push(ch);
        current_col += ch_width;
    }
    
    result
}
```

#### 2.3 任務清單

- [ ] 實作 `scroll_horizontal_if_needed()`
- [ ] 在 `scroll_if_needed()` 中呼叫水平滾動
- [ ] 實作 `slice_visible_text()` 工具函數
- [ ] 修改 `render()` 使用截取後的文字
- [ ] 修改 `get_cursor_visual_position()` 減去 `offset_col`

---

### Phase 3: Cursor 移動適配 (預計 1-2 小時)

#### 3.1 簡化單行模式的上下移動

**檔案：`src/cursor.rs`**

```rust
impl Cursor {
    pub fn move_up(&mut self, buffer: &RopeBuffer, view: &View) {
        if view.wrap_mode && self.visual_line_index > 0 {
            // 多行模式：在視覺行間移動
            self.visual_line_index -= 1;
            self.update_logical_col_from_visual(buffer, view);
        } else if self.row > 0 {
            // 單行模式或已在第一個視覺行：移動到上一邏輯行
            self.row -= 1;
            if view.wrap_mode {
                let visual_lines = view.calculate_visual_lines_for_row(buffer, self.row);
                self.visual_line_index = visual_lines.len().saturating_sub(1);
            } else {
                self.visual_line_index = 0;  // 單行模式永遠 = 0
            }
            self.update_logical_col_from_visual(buffer, view);
        }
    }

    pub fn move_down(&mut self, buffer: &RopeBuffer, view: &View) {
        if view.wrap_mode {
            let visual_lines = view.calculate_visual_lines_for_row(buffer, self.row);
            if self.visual_line_index + 1 < visual_lines.len() {
                self.visual_line_index += 1;
                self.update_logical_col_from_visual(buffer, view);
                return;
            }
        }
        
        // 移動到下一邏輯行
        if self.row + 1 < buffer.line_count() {
            self.row += 1;
            self.visual_line_index = 0;
            self.update_logical_col_from_visual(buffer, view);
        }
    }
}
```

#### 3.2 任務清單

- [ ] 修改 `move_up()` 支援單行模式
- [ ] 修改 `move_down()` 支援單行模式
- [ ] 確認 `move_left/right()` 行為正確
- [ ] 測試 Home/End 鍵行為

---

### Phase 4: Selection 渲染適配 (預計 2-3 小時)

#### 4.1 單行模式 Selection 渲染

**檔案：`src/view.rs` - `render()` 方法**

Selection 的邏輯座標計算不變，只需修改渲染時的可見範圍判斷：

```rust
// 單行模式下的 Selection 渲染
if !self.wrap_mode {
    for (idx, &ch) in chars.iter().enumerate() {
        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(1);
        
        // 跳過 offset_col 之前的字符
        if current_visual_pos + ch_width <= self.offset_col {
            current_visual_pos += ch_width;
            continue;
        }
        
        // 超出可見範圍則停止
        if current_visual_pos >= self.offset_col + available_width {
            break;
        }
        
        // Selection 判斷（使用原始視覺座標，不受 offset 影響）
        let is_selected = /* 現有邏輯不變 */;
        
        if is_selected {
            queue!(stdout, style::SetAttribute(Attribute::Reverse))?;
        }
        queue!(stdout, style::Print(ch))?;
        if is_selected {
            queue!(stdout, style::SetAttribute(Attribute::NoReverse))?;
        }
        
        current_visual_pos += ch_width;
    }
}
```

#### 4.2 任務清單

- [ ] 修改 Selection 渲染邏輯支援 offset_col
- [ ] 確保跨行 Selection 正確顯示
- [ ] 測試 Shift+方向鍵 選取
- [ ] 測試 Ctrl+A 全選

---

### Phase 5: Syntax Highlighting 適配 (預計 3-4 小時)

#### 5.1 策略選擇

**方案 A：單行模式降級為純文字** ⭐ 推薦先實作
- 實作簡單，風險低
- 可在後續版本優化

```rust
// view.rs render()
#[cfg(feature = "syntax-highlighting")]
let use_syntax_highlight = selection.is_none()
    && self.wrap_mode  // 新增：單行模式不使用語法高亮
    && visual_idx == 0
    && highlighted_lines.and_then(|h| h.get(&file_row)).is_some();
```

**方案 B：完整支援語法高亮**（進階）
- 需要正確切割 ANSI escape codes
- 使用 `ansi-cut` 或類似函式庫
- 複雜度高，建議後續迭代

#### 5.2 任務清單

- [ ] Phase 5a：實作降級方案（純文字）
- [ ] Phase 5b（選擇性）：研究 ANSI 切割方案
- [ ] Phase 5b（選擇性）：實作完整語法高亮支援

---

## 測試計畫

### 功能測試

| 測試項目 | 預期行為 |
|----------|----------|
| Ctrl+L 切換 | 行號、換行模式同時切換 |
| 單行模式游標右移 | 超出邊界時自動水平滾動 |
| 單行模式游標左移 | 回到邊界時自動水平滾動 |
| 單行模式上下移動 | 直接移動邏輯行，不跨視覺行 |
| 單行模式 Selection | 正確高亮可見範圍內的選取 |
| 長行編輯 | 輸入/刪除時視圖正確更新 |

### 邊界測試

- [ ] 空檔案
- [ ] 單行超長檔案
- [ ] 含有 Tab 的行
- [ ] 含有中文/Emoji 的行
- [ ] Selection 跨越不可見區域

---

## 風險與緩解

| 風險 | 影響 | 緩解措施 |
|------|------|----------|
| 水平滾動性能 | 長行渲染變慢 | 只渲染可見區域 |
| ANSI 切割錯誤 | 顏色錯亂 | 降級為純文字 |
| 座標計算錯誤 | 游標位置錯誤 | 增加 debug 資訊 |

---

## 時程估計

| 階段 | 預計時間 | 累計 |
|------|----------|------|
| Phase 1: 基礎架構 | 1-2 小時 | 2 小時 |
| Phase 2: 水平滾動 | 2-3 小時 | 5 小時 |
| Phase 3: Cursor 適配 | 1-2 小時 | 7 小時 |
| Phase 4: Selection 適配 | 2-3 小時 | 10 小時 |
| Phase 5a: 語法高亮降級 | 0.5 小時 | 10.5 小時 |
| 測試與除錯 | 2-3 小時 | 13.5 小時 |

**總計：約 1.5-2 個工作天**

---

## 後續優化（可選）

1. **單行模式語法高亮**：實作 ANSI escape code 切割
2. **行號區顯示水平位置指示**：如 `+50→` 表示偏移 50 列
3. **快捷鍵獨立切換**：分離行號與換行模式的快捷鍵
4. **設定檔支援**：記住使用者偏好的預設模式

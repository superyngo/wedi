# 單行/多行顯示切換實作總結

## 實作完成日期
2025-12-26

## 概述

成功實作了單行/多行顯示切換功能，透過 `Ctrl+L` 可以同時切換：
- **顯示行號 + 多行換行模式（wrap mode）**
- **關閉行號 + 單行水平滾動模式（no-wrap mode）**

## 實作內容

### Phase 1: 基礎架構 ✓

#### 修改的結構體 (src/view.rs)

1. **View 結構體新增欄位**：
   ```rust
   pub struct View {
       pub offset_row: usize,
       pub offset_col: usize,      // 新增：水平偏移（單行模式用）
       pub show_line_numbers: bool,
       pub wrap_mode: bool,         // 新增：換行模式
       pub screen_rows: usize,
       pub screen_cols: usize,
       line_layout_cache: Vec<Option<LineLayout>>,
   }
   ```

2. **LineLayout::new() 修改**：
   - 新增 `wrap` 參數控制是否換行
   - 單行模式下不切分文字，直接保留完整行

3. **toggle_line_numbers() 連動切換**：
   ```rust
   pub fn toggle_line_numbers(&mut self) {
       self.show_line_numbers = !self.show_line_numbers;
       self.wrap_mode = self.show_line_numbers;
       self.offset_col = 0;
       self.invalidate_cache();
   }
   ```

### Phase 2: 水平滾動 ✓

#### 新增常量 (src/view.rs)
```rust
const HORIZONTAL_SCROLL_MARGIN: usize = 5;
```

### Phase 5: Syntax Highlighting 適配 ✓

#### Phase 5b: 完整支援語法高亮 ✓ (2025-12-26 新增)

新增 `src/utils/ansi_slice.rs` 模組，實作 ANSI escape codes 切割：

1. **slice_ansi_text() 函數**：
   ```rust
   /// 切割帶有 ANSI escape codes 的文字
   pub fn slice_ansi_text(text: &str, start_col: usize, width: usize) -> String
   ```
   - 正確解析 ANSI CSI 序列
   - 追蹤活躍的樣式狀態
   - 在切割邊界正確輸出/繼承樣式
   - 確保結尾有重置碼

2. **支援的 ANSI 格式**：
   - 真彩色: `\x1b[38;2;R;G;Bm`
   - 256 色: `\x1b[38;5;Nm`
   - 基本色: `\x1b[30-37m`
   - 重置: `\x1b[0m`

3. **輔助函數**：
   - `collect_escape_sequence()` - 收集完整的 escape sequence
   - `ansi_visual_width()` - 計算帶 ANSI 的字串視覺寬度

## 修改的檔案

1. **src/view.rs**：
   - 新增 2 個欄位
   - 新增 1 個常量
   - 新增 2 個方法
   - 修改 8 個方法
   - 更新 4 處 LineLayout::new() 調用點
   - 整合 slice_ansi_text() 用於語法高亮

2. **src/cursor.rs**：
   - 修改 2 個方法（move_up, move_down）

3. **src/utils/ansi_slice.rs** (新增)：
   - ANSI escape codes 解析器
   - slice_ansi_text() 切割函數
   - 257 行代碼 + 測試

4. **src/utils/mod.rs**：
   - 導出 slice_ansi_text 函數

## 功能特性

### 已實作功能

✅ Ctrl+L 切換顯示模式
✅ 單行模式水平滾動
✅ 游標移動自動觸發水平滾動
✅ Selection 正確顯示（考慮水平偏移）
✅ **單行模式完整語法高亮支援** (Phase 5b)
✅ 中文/Unicode 字符正確處理
✅ Tab 字符正確處理

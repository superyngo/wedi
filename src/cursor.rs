use crate::buffer::RopeBuffer;
use crate::utils::visual_width;
use crate::view::View;

#[derive(Debug, Clone, Copy)]
pub struct Cursor {
    pub row: usize,                // 邏輯行號 (0-based)
    pub col: usize,                // 邏輯列號 (0-based)
    pub visual_line_index: usize,  // 在當前邏輯行的第幾個視覺行 (0-based)
    pub desired_visual_col: usize, // 期望的視覺列位置（用於上下移動）
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            row: 0,
            col: 0,
            visual_line_index: 0,
            desired_visual_col: 0,
        }
    }

    pub fn move_up(&mut self, buffer: &RopeBuffer, view: &View) {
        if self.visual_line_index > 0 {
            // 在同一邏輯行內向上移動到上一個視覺行
            self.visual_line_index -= 1;
            self.update_logical_col_from_visual(buffer, view);
        } else {
            // 移動到上一個邏輯行
            if self.row > 0 {
                self.row -= 1;
                // 移動到該邏輯行的最後一個視覺行
                let visual_lines = view.calculate_visual_lines_for_row(buffer, self.row);
                self.visual_line_index = visual_lines.len().saturating_sub(1);
                self.update_logical_col_from_visual(buffer, view);
            }
        }
    }

    pub fn move_down(&mut self, buffer: &RopeBuffer, view: &View) {
        let visual_lines = view.calculate_visual_lines_for_row(buffer, self.row);

        if self.visual_line_index + 1 < visual_lines.len() {
            // 在同一邏輯行內向下移動到下一個視覺行
            self.visual_line_index += 1;
            self.update_logical_col_from_visual(buffer, view);
        } else {
            // 移動到下一個邏輯行
            if self.row + 1 < buffer.line_count() {
                self.row += 1;
                self.visual_line_index = 0;
                self.update_logical_col_from_visual(buffer, view);
            }
        }
    }

    pub fn move_left(&mut self, buffer: &RopeBuffer, view: &View) {
        if self.col > 0 {
            self.col -= 1;
            self.update_visual_from_logical(buffer, view);
        } else if self.row > 0 {
            // 移動到上一行末尾
            self.row -= 1;
            self.col = self.line_len(buffer, self.row);
            self.update_visual_from_logical(buffer, view);
        }
        self.sync_desired_visual_col(buffer, view);
    }

    pub fn move_right(&mut self, buffer: &RopeBuffer, view: &View) {
        let line_len = self.line_len(buffer, self.row);
        if self.col < line_len {
            self.col += 1;
            self.update_visual_from_logical(buffer, view);
        } else if self.row + 1 < buffer.line_count() {
            // 移動到下一行開頭
            self.row += 1;
            self.col = 0;
            self.visual_line_index = 0;
            self.desired_visual_col = 0;
        }
        self.sync_desired_visual_col(buffer, view);
    }

    pub fn move_to_line_start(&mut self) {
        self.col = 0;
        self.visual_line_index = 0;
        self.desired_visual_col = 0;
    }

    pub fn move_to_line_end(&mut self, buffer: &RopeBuffer, view: &View) {
        self.col = self.line_len(buffer, self.row);
        self.update_visual_from_logical(buffer, view);
        self.sync_desired_visual_col(buffer, view);
    }

    pub fn move_page_up(&mut self, buffer: &RopeBuffer, view: &View, effective_rows: usize) {
        let mut target_row = self.row;
        let mut visual_count = 0;

        // 向上累積視覺行直到達到約一個螢幕
        while target_row > 0 && visual_count < effective_rows {
            target_row -= 1;
            let vlines = view.calculate_visual_lines_for_row(buffer, target_row);
            visual_count += vlines.len();
        }

        self.row = target_row;
        self.visual_line_index = 0;
        self.update_logical_col_from_visual(buffer, view);
    }

    pub fn move_page_down(&mut self, buffer: &RopeBuffer, view: &View, effective_rows: usize) {
        let max_row = buffer.line_count().saturating_sub(1);
        let mut target_row = self.row;
        let mut visual_count = 0;

        // 向下累積視覺行直到達到約一個螢幕
        while target_row < max_row && visual_count < effective_rows {
            let vlines = view.calculate_visual_lines_for_row(buffer, target_row);
            visual_count += vlines.len();
            target_row += 1;
        }

        self.row = target_row.min(max_row);
        self.visual_line_index = 0;
        self.update_logical_col_from_visual(buffer, view);
    }

    #[allow(dead_code)]
    pub fn move_to_line(&mut self, buffer: &RopeBuffer, view: &View, line: usize) {
        self.row = line.min(buffer.line_count().saturating_sub(1));
        self.visual_line_index = 0;
        self.update_logical_col_from_visual(buffer, view);
    }

    /// 獲取光標在文本中的絕對字符位置
    pub fn char_position(&self, buffer: &RopeBuffer) -> usize {
        buffer.line_to_char(self.row) + self.col
    }

    /// 設置光標位置並同步視覺狀態
    /// 這是統一的光標位置設置方法，確保邏輯和視覺狀態一致
    pub fn set_position(&mut self, buffer: &RopeBuffer, view: &View, row: usize, col: usize) {
        self.row = row;
        self.col = col;
        self.update_visual_from_logical(buffer, view);
        self.sync_desired_visual_col(buffer, view);
    }

    /// 重置到行首（用於換行等操作）
    pub fn reset_to_line_start(&mut self) {
        self.col = 0;
        self.visual_line_index = 0;
        self.desired_visual_col = 0;
    }

    /// 從視覺座標更新邏輯列位置
    fn update_logical_col_from_visual(&mut self, buffer: &RopeBuffer, view: &View) {
        let visual_col = self.desired_visual_col;
        self.col = view.visual_to_logical_col(buffer, self.row, self.visual_line_index, visual_col);

        // 確保不超出行長度
        let line_len = self.line_len(buffer, self.row);
        self.col = self.col.min(line_len);
    }

    /// 從邏輯座標更新視覺座標
    fn update_visual_from_logical(&mut self, buffer: &RopeBuffer, view: &View) {
        let visual_lines = view.calculate_visual_lines_for_row(buffer, self.row);

        if let Some(line) = buffer.line(self.row) {
            let line_str = line.to_string();
            let visual_col = view.logical_col_to_visual_col(&line_str, self.col);

            // 找出光標在哪個視覺行
            let mut accumulated = 0;
            for (idx, vline) in visual_lines.iter().enumerate() {
                let vline_len = visual_width(vline);
                if visual_col < accumulated + vline_len || idx == visual_lines.len() - 1 {
                    self.visual_line_index = idx;
                    break;
                }
                accumulated += vline_len;
            }
        } else {
            self.visual_line_index = 0;
        }
    }

    /// 同步期望視覺列位置
    fn sync_desired_visual_col(&mut self, buffer: &RopeBuffer, view: &View) {
        if let Some(line) = buffer.line(self.row) {
            let line_str = line.to_string();
            let visual_col = view.logical_col_to_visual_col(&line_str, self.col);

            // 計算在當前視覺行內的列位置
            let visual_lines = view.calculate_visual_lines_for_row(buffer, self.row);
            let mut accumulated = 0;
            for i in 0..self.visual_line_index {
                if i < visual_lines.len() {
                    accumulated += visual_width(&visual_lines[i]);
                }
            }

            self.desired_visual_col = visual_col - accumulated;
        }
    }

    /// 獲取指定行的長度（不包含換行符）
    fn line_len(&self, buffer: &RopeBuffer, row: usize) -> usize {
        if let Some(line) = buffer.line(row) {
            let text = line.to_string();
            let text = text.trim_end_matches(['\n', '\r']);
            text.chars().count()
        } else {
            0
        }
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new()
    }
}

use crate::buffer::RopeBuffer;

#[derive(Debug, Clone, Copy)]
pub struct Cursor {
    pub row: usize,        // 邏輯行號 (0-based)
    pub col: usize,        // 邏輯列號 (0-based)
    pub desired_col: usize, // 上下移動時保持的列
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            row: 0,
            col: 0,
            desired_col: 0,
        }
    }

    pub fn move_up(&mut self, buffer: &RopeBuffer) {
        if self.row > 0 {
            self.row -= 1;
            self.adjust_col_to_desired(buffer);
        }
    }

    pub fn move_down(&mut self, buffer: &RopeBuffer) {
        if self.row + 1 < buffer.line_count() {
            self.row += 1;
            self.adjust_col_to_desired(buffer);
        }
    }

    pub fn move_left(&mut self, buffer: &RopeBuffer) {
        if self.col > 0 {
            self.col -= 1;
        } else if self.row > 0 {
            // 移動到上一行末尾
            self.row -= 1;
            self.col = self.line_len(buffer, self.row);
        }
        self.desired_col = self.col;
    }

    pub fn move_right(&mut self, buffer: &RopeBuffer) {
        let line_len = self.line_len(buffer, self.row);
        if self.col < line_len {
            self.col += 1;
        } else if self.row + 1 < buffer.line_count() {
            // 移動到下一行開頭
            self.row += 1;
            self.col = 0;
        }
        self.desired_col = self.col;
    }

    pub fn move_to_line_start(&mut self) {
        self.col = 0;
        self.desired_col = 0;
    }

    pub fn move_to_line_end(&mut self, buffer: &RopeBuffer) {
        self.col = self.line_len(buffer, self.row);
        self.desired_col = self.col;
    }

    pub fn move_page_up(&mut self, buffer: &RopeBuffer, page_size: usize) {
        if self.row >= page_size {
            self.row -= page_size;
        } else {
            self.row = 0;
        }
        self.adjust_col_to_desired(buffer);
    }

    pub fn move_page_down(&mut self, buffer: &RopeBuffer, page_size: usize) {
        let max_row = buffer.line_count().saturating_sub(1);
        if self.row + page_size < max_row {
            self.row += page_size;
        } else {
            self.row = max_row;
        }
        self.adjust_col_to_desired(buffer);
    }

    pub fn move_to_line(&mut self, buffer: &RopeBuffer, line: usize) {
        self.row = line.min(buffer.line_count().saturating_sub(1));
        self.adjust_col_to_desired(buffer);
    }

    /// 獲取光標在文本中的絕對字符位置
    pub fn char_position(&self, buffer: &RopeBuffer) -> usize {
        buffer.line_to_char(self.row) + self.col
    }

    /// 調整列位置到期望的列，確保不超出行長度
    fn adjust_col_to_desired(&mut self, buffer: &RopeBuffer) {
        let line_len = self.line_len(buffer, self.row);
        self.col = self.desired_col.min(line_len);
    }

    /// 獲取指定行的長度（不包含換行符）
    fn line_len(&self, buffer: &RopeBuffer, row: usize) -> usize {
        if let Some(line) = buffer.line(row) {
            let text = line.to_string();
            // 移除末尾的換行符
            let text = text.trim_end_matches(|c| c == '\n' || c == '\r');
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
